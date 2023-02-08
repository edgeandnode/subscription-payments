// SPDX-License-Identifier: MIT

pragma solidity ^0.8.17;

import '@openzeppelin/contracts/access/Ownable.sol';
import '@openzeppelin/contracts/token/ERC20/IERC20.sol';
import '@openzeppelin/contracts/utils/math/Math.sol';
import '@openzeppelin/contracts/utils/math/SignedMath.sol';

/// @title Graph subscriptions contract. This contract is designed to allow users of the Graph
/// Protocol to pay gateways for their services with limited risk of losing tokens.
/// @notice This contract makes no assumptions about how the subscription rate is interpreted by the
/// gateway.
contract Subscriptions is Ownable {
    /// @notice A Subscription represents a lockup of `rate` tokens per second for the half-open
    /// timestamp range [start, end).
    struct Subscription {
        uint64 start;
        uint64 end;
        uint128 rate;
    }
    /// @notice An epoch defines the end of a span of blocks, the length of which is defined by
    /// `epochSeconds`. These exist to facilitate a relatively efficient `collect` implementation
    /// while allowing users to recover unlocked tokens at a block granularity.
    struct Epoch {
        int128 delta;
        int128 extra;
    }

    event Init(address token);
    event Subscribe(
        address indexed user,
        uint64 start,
        uint64 end,
        uint128 rate
    );
    event Unsubscribe(address indexed user);
    event Extend(address indexed user, uint64 end);

    /// @notice ERC-20 token held by this contract.
    IERC20 public immutable token;
    /// @notice Duration of each epoch in seconds.
    uint64 public immutable epochSeconds;
    /// @notice Mapping of users to their most recent subscription.
    mapping(address => Subscription) public subscriptions;
    /// @notice Mapping of epoch numbers to their payloads.
    mapping(uint256 => Epoch) public epochs;
    /// @notice Epoch cursor position.
    uint256 public uncollectedEpoch;
    /// @notice Epoch cursor value.
    int128 public collectPerEpoch;
    /// @notice Mapping of user to set of authorized signer.
    mapping(address => mapping(address => bool)) public authorizedSigners;

    /// @param _token The ERC-20 token held by this contract
    /// @param _epochSeconds The Duration of each epoch in seconds.
    constructor(address _token, uint64 _epochSeconds) {
        token = IERC20(_token);
        epochSeconds = _epochSeconds;
        uncollectedEpoch = block.timestamp / _epochSeconds;

        emit Init(_token);
    }

    /// @param _user Subscription owner.
    /// @param _signer Address to be authorized to sign messages on the owners behalf.
    function addAuthorizedSigner(address _user, address _signer) public {
        require(_user != _signer, 'user is always an authorized signer');
        authorizedSigners[_user][_signer] = true;
    }

    /// @param _user Subscription owner.
    /// @param _signer Address to become unauthorized to sign messages on the owners behalf.
    function removeAuthorizedSigner(address _user, address _signer) public {
        require(_user != _signer, 'user is always an authorized signer');
        authorizedSigners[_user][_signer] = false;
    }

    /// @param _user Subscription owner.
    /// @param _signer Address potentially authorized to sign messages on the owners behalf.
    /// @return isAuthorized True if the given signer is set as an authorized signer for the given
    /// user, false otherwise.
    function checkAuthorizedSigner(address _user, address _signer) public view returns (bool) {
        if (_user == _signer) {
            return true;
        }
        return authorizedSigners[_user][_signer];
    }

    /// @notice Collect a subset of the locked tokens held by this contract.
    function collect() public {
        collect(0);
    }

    /// @notice Collect a subset of the locked tokens held by this contract.
    /// @param _offset epochs before the current epoch to end collection. This should be zero unless
    /// this call would otherwise be expected to run out of gas.
    function collect(uint256 _offset) public onlyOwner {
        uint256 endEpoch = currentEpoch() - _offset;
        int128 total = 0;
        while (uncollectedEpoch < endEpoch) {
            Epoch storage epoch = epochs[uncollectedEpoch];
            collectPerEpoch += epoch.delta;
            total += collectPerEpoch + epoch.extra;
            delete epochs[uncollectedEpoch];

            unchecked { ++uncollectedEpoch; }
        }

        bool success = token.transfer(owner(), uint128(total));
        require(success, 'IERC20 token transfer failed');
    }

    /// @param user Owner for the new subscription.
    /// @param start Start timestamp for the new subscription.
    /// @param end End timestamp for the new subscription.
    /// @param rate Rate for the new subscription.
    function subscribe(address user, uint64 start, uint64 end, uint128 rate) public {
        // This can be called by any account for a given user, because it requires that the active
        // subscription start before the current block. Otherwise, this contract might not be able
        // to recover the unlocked tokens for the active subscription we're overwriting.

        require(user != address(0), 'user is null');
        require(user != address(this), 'invalid user');
        start = uint64(Math.max(start, block.timestamp));
        require(start < end, 'start must be less than end');

        // Only the user can overwrite an active subscription.
        if (subscriptions[user].end > block.timestamp) {
            require(user == msg.sender, 'active subscription must have ended');
            unsubscribe();
        }

        subscriptions[user] = Subscription({ start: start, end: end, rate: rate });
        setEpochs(start, end, int128(rate));

        uint128 subTotal = rate * (end - start);
        bool success = token.transferFrom(msg.sender, address(this), subTotal);
        require(success, 'IERC20 token transfer failed');

        emit Subscribe(user, start, end, rate);
    }

    /// @notice Remove the sender's subscription. Unlocked tokens will be transfered to the sender.
    function unsubscribe() public {
        address user = msg.sender;
        require(user != address(0), 'user is null');

        Subscription storage sub = subscriptions[user];
        require(sub.start != 0, 'no active subscription');

        uint64 _now = uint64(block.timestamp);
        require(sub.end > _now, 'Subscription has expired');

        uint128 tokenAmount = unlocked(sub.start, sub.end, sub.rate);

        if ((sub.start <= _now) && (_now < sub.end)) {
            setEpochs(sub.start, sub.end, -int128(sub.rate));
            setEpochs(sub.start, _now, int128(sub.rate));
            subscriptions[user].end = _now;
        } else if (_now < sub.start) {
            setEpochs(sub.start, sub.end, -int128(sub.rate));
            delete subscriptions[user];
        }

        bool success = token.transfer(user, tokenAmount);
        require(success, 'IERC20 token transfer failed');

        emit Unsubscribe(user);
    }

    /// @param user Owner of the subscription will be extended.
    /// @param end New end timestamp for the user's subscription.
    function extendSubscription(address user, uint64 end) public {
        require(user != address(0), 'user is null');
        Subscription storage sub = subscriptions[user];
        require(
            (sub.start <= block.timestamp) && (block.timestamp < sub.end),
            'current subscription must be active'
        );
        require(sub.end < end, 'end must be after that of the current subscription');

        setEpochs(sub.start, sub.end, -int128(sub.rate));
        setEpochs(sub.start, end, int128(sub.rate));

        uint64 currentEnd = sub.end;
        subscriptions[user].end = end;

        uint128 addition = sub.rate * (end - currentEnd);
        bool success = token.transferFrom(msg.sender, address(this), addition);
        require(success, 'IERC20 token transfer failed');

        emit Extend(user, end);
    }

    /// @param _timestamp Block timestamp, in seconds.
    /// @return epoch Epoch number, rouded up to the next epoch Boundary.
    function timestampToEpoch(uint256 _timestamp) public view returns (uint256) {
        return (_timestamp / epochSeconds) + 1;
    }

    /// @return epoch Current epoch number, rouded up to the next epoch Boundary.
    function currentEpoch() public view returns (uint256) {
        return timestampToEpoch(block.number);
    }

    /// @param _subStart Start timestamp of the active subscription.
    /// @param _subEnd End timestamp of the active subscription.
    /// @param _subRate Active subscription rate.
    /// @return lockedTokens Amount of locked tokens for the given subscription, which are
    /// collectable by the contract owner and are not recoverable by the user.
    /// @dev Defined as `rate * max(0, min(now, end) - start)`.
    function locked(uint64 _subStart, uint64 _subEnd, uint128 _subRate) public view returns (uint128) {
        uint256 len = uint256(
            SignedMath.max(0, int256(Math.min(block.timestamp, _subEnd)) - int64(_subStart))
        );
        return _subRate * uint128(len);
    }

    /// @param _user Address of the active subscription owner.
    /// @return lockedTokens Amount of locked tokens for the given subscription, which are
    /// collectable by the contract owner and are not recoverable by the user.
    /// @dev Defined as `rate * max(0, min(now, end) - start)`.
    function locked(address _user) public view returns (uint128) {
        Subscription storage sub = subscriptions[_user];
        return locked(sub.start, sub.end, sub.rate);
    }

    /// @param _subStart Start timestamp of the active subscription.
    /// @param _subEnd End timestamp of the active subscription.
    /// @param _subRate Active subscription rate.
    /// @return unlockedTokens Amount of unlocked tokens, which are recoverable by the user, and are
    /// not collectable by the contract owner.
    /// @dev Defined as `rate * max(0, end - max(now, start))`.
    function unlocked(uint64 _subStart, uint64 _subEnd, uint128 _subRate) public view returns (uint128) {
        uint256 len = uint256(
            SignedMath.max(0, int256(int64(_subEnd)) - int256(Math.max(block.timestamp, _subStart)))
        );
        return _subRate * uint128(len);
    }

    /// @param _user Address of the active subscription owner.
    /// @return unlockedTokens Amount of unlocked tokens, which are recoverable by the user, and are
    /// not collectable by the contract owner.
    /// @dev Defined as `rate * max(0, end - max(now, start))`.
    function unlocked(address _user) public view returns (uint128) {
        Subscription storage sub = subscriptions[_user];
        return unlocked(sub.start, sub.end, sub.rate);
    }

    function setEpochs(uint64 start, uint64 end, int128 rate) private {
        /*
        Example subscription layout using
            epochSeconds = 6
            sub = {start: 2, end: 9, rate: 1}

        blocks: |0 |1 |2 |3 |4 |5 |6 |7 |8 |9 |10|11|
                                      ^ currentBlock
                       ^start               ^end
        epochs: |                1|                2|
                               e1^               e2^
        */

        uint256 e = currentEpoch();
        uint256 e1 = timestampToEpoch(start);
        if (e <= e1) {
            epochs[e1].delta += rate * int64(epochSeconds);
            epochs[e1].extra -= rate * int64(start - (uint64(e1 - 1) * epochSeconds));
        }
        uint256 e2 = timestampToEpoch(end);
        if (e <= e2) {
            epochs[e2].delta -= rate * int64(epochSeconds);
            epochs[e2].extra += rate * int64(end - (uint64(e2 - 1) * epochSeconds));
        }
    }
}
