// SPDX-License-Identifier: MIT

pragma solidity ^0.8.17;

import '@openzeppelin/contracts/access/Ownable.sol';
import '@openzeppelin/contracts/token/ERC20/IERC20.sol';
import '@openzeppelin/contracts/utils/math/Math.sol';
import '@openzeppelin/contracts/utils/math/SignedMath.sol';

/// @title Graph subscriptions contract.
/// @notice This contract is designed to allow users of the Graph Protocol to pay gateways for their services with limited risk of losing tokens.
/// It also allows registering authorized signers with the gateway that can create subscription tickets on behalf of the user.
/// This contract makes no assumptions about how the subscription rate is interpreted by the
/// gateway.
contract Subscriptions is Ownable {
    // -- State --
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
    /// @notice Mapping of user to set of authorized signers.
    mapping(address => mapping(address => bool)) public authorizedSigners;
    /// @notice Mapping of user to pending subscription.
    mapping(address => Subscription) public pendingSubscriptions;
    /// @notice Address of the recurring payments contract.
    address public recurringPayments;

    // -- Events --
    event Init(address token, uint64 epochSeconds, address recurringPayments);
    event Subscribe(
        address indexed user,
        uint256 indexed epoch,
        uint64 start,
        uint64 end,
        uint128 rate
    );
    event Unsubscribe(address indexed user, uint256 indexed epoch);
    event Extend(
        address indexed user,
        uint64 oldEnd,
        uint64 newEnd,
        uint256 amount
    );
    event PendingSubscriptionCreated(
        address indexed user,
        uint256 indexed epoch,
        uint64 start,
        uint64 end,
        uint128 rate
    );
    event AuthorizedSignerAdded(
        address indexed subscriptionOwner,
        address indexed authorizedSigner
    );
    event AuthorizedSignerRemoved(
        address indexed subscriptionOwner,
        address indexed authorizedSigner
    );
    event TokensCollected(
        address indexed owner,
        uint256 amount,
        uint256 indexed startEpoch,
        uint256 indexed endEpoch
    );
    event RecurringPaymentsUpdated(address indexed recurringPayments);

    modifier onlyRecurringPayments() {
        require(
            msg.sender == recurringPayments,
            'caller is not the recurring payments contract'
        );
        _;
    }

    // -- Functions --
    /// @param _token The ERC-20 token held by this contract
    /// @param _epochSeconds The Duration of each epoch in seconds.
    /// @dev Contract ownership must be transfered to the gateway after deployment.
    constructor(
        address _token,
        uint64 _epochSeconds,
        address _recurringPayments
    ) {
        token = IERC20(_token);
        epochSeconds = _epochSeconds;
        uncollectedEpoch = block.timestamp / _epochSeconds;
        _setRecurringPayments(_recurringPayments);

        emit Init(_token, _epochSeconds, _recurringPayments);
    }

    /// @notice Create a subscription for the sender.
    /// Will override an active subscription if one exists.
    /// @dev Setting a start time in the past will clamp it to the current block timestamp.
    /// This protects users from paying for a subscription during a period of time they were
    /// not able to use it.
    /// @param start Start timestamp for the new subscription.
    /// @param end End timestamp for the new subscription.
    /// @param rate Rate for the new subscription.
    function subscribe(uint64 start, uint64 end, uint128 rate) public {
        start = uint64(Math.max(start, block.timestamp));
        _subscribe(msg.sender, start, end, rate);
    }

    /// @notice Remove the sender's subscription. Unlocked tokens will be transfered to the sender.
    function unsubscribe() public {
        _unsubscribe(msg.sender);
    }

    /// @notice Collect a subset of the locked tokens held by this contract.
    function collect() public onlyOwner {
        collect(0);
    }

    /// @notice Collect a subset of the locked tokens held by this contract.
    /// @param _offset epochs before the current epoch to end collection. This should be zero unless
    /// this call would otherwise be expected to run out of gas.
    function collect(uint256 _offset) public onlyOwner {
        address owner = owner();
        uint256 startEpoch = uncollectedEpoch;
        uint256 endEpoch = currentEpoch() - _offset;

        int128 total = 0;
        uint256 _uncollectedEpoch = uncollectedEpoch;
        while (_uncollectedEpoch < endEpoch) {
            Epoch storage epoch = epochs[_uncollectedEpoch];
            collectPerEpoch += epoch.delta;
            total += collectPerEpoch + epoch.extra;
            delete epochs[_uncollectedEpoch];

            unchecked {
                ++_uncollectedEpoch;
            }
        }
        uncollectedEpoch = _uncollectedEpoch;

        // This should never happen but we need to check due to the int > uint cast below
        require(total >= 0, 'total must be non-negative');
        uint256 amount = uint128(total);

        bool success = token.transfer(owner, amount);
        require(success, 'IERC20 token transfer failed');

        emit TokensCollected(owner, amount, startEpoch, endEpoch);
    }

    /// @notice Creates a subscription template without requiring funds. Expected to be used with
    /// `fulfil`.
    /// @dev Setting a start time in the past will clamp it to the current block timestamp when fulfilled.
    /// This protects users from paying for a subscription during a period of time they were
    /// not able to use it.
    /// @param start Start timestamp for the pending subscription.
    /// @param end End timestamp for the pending subscription.
    /// @param rate Rate for the pending subscription.
    function setPendingSubscription(
        uint64 start,
        uint64 end,
        uint128 rate
    ) public {
        address user = msg.sender;
        pendingSubscriptions[user] = Subscription({
            start: start,
            end: end,
            rate: rate
        });
        uint256 epoch = currentEpoch();
        emit PendingSubscriptionCreated(user, epoch, start, end, rate);
    }

    /// @notice Fulfil method for the payment fulfilment service
    /// @param _to Owner of the new subscription.
    /// @notice Equivalent to calling `subscribe` with the previous `setPendingSubscription`
    /// arguments for the same user.
    function fulfil(address _to, uint256 _amount) public {
        Subscription storage pendingSub = pendingSubscriptions[_to];
        require(
            pendingSub.start != 0 && pendingSub.end != 0,
            'No pending subscription'
        );

        uint64 subStart = uint64(Math.max(pendingSub.start, block.timestamp));
        require(subStart < pendingSub.end, 'Pending subscription has expired');
        uint256 subAmount = pendingSub.rate * (pendingSub.end - subStart);
        require(
            _amount >= subAmount,
            'Insufficient funds to create subscription'
        );

        // Create the subscription using the pending subscription details
        _subscribe(_to, subStart, pendingSub.end, pendingSub.rate);
        delete pendingSubscriptions[_to];

        // Send any extra tokens back to the user
        uint256 extra = _amount - subAmount;

        if (extra > 0) {
            bool pullSuccess = token.transferFrom(
                msg.sender,
                address(this),
                extra
            );
            require(pullSuccess, 'IERC20 token transfer failed');

            bool transferSuccess = token.transfer(_to, extra);
            require(transferSuccess, 'IERC20 token transfer failed');
        }
    }

    /// @param _signer Address to be authorized to sign messages on the sender's behalf.
    function addAuthorizedSigner(address _signer) public {
        address user = msg.sender;
        require(user != _signer, 'user is always an authorized signer');
        authorizedSigners[user][_signer] = true;

        emit AuthorizedSignerAdded(user, _signer);
    }

    /// @param _signer Address to become unauthorized to sign messages on the sender's behalf.
    function removeAuthorizedSigner(address _signer) public {
        address user = msg.sender;
        require(user != _signer, 'user is always an authorized signer');
        delete authorizedSigners[user][_signer];

        emit AuthorizedSignerRemoved(user, _signer);
    }

    /// @notice Create a subscription for a user.
    /// Will override an active subscription if one exists.
    /// @dev The function's name and signature, `create`, are used to comply with the `IPayment`
    /// interface for recurring payments.
    /// @dev Note that this function does not protect user against a start time in the past.
    /// @param user Subscription owner.
    /// @param data Encoded start, end and rate for the new subscription.
    function create(
        address user,
        bytes calldata data
    ) public onlyRecurringPayments {
        (uint64 start, uint64 end, uint128 rate) = abi.decode(
            data,
            (uint64, uint64, uint128)
        );
        _subscribe(user, start, end, rate);
    }

    /// @notice Extends a subscription's end time.
    /// The time the subscription will be extended by is calculated as `amount / rate`, where
    /// `rate` is the existing subscription rate and `amount` is the new amount of tokens provided.
    /// @dev The function's name, `addTo`, is used to comply with the `IPayment` interface for recurring payments.
    /// @param user Subscription owner.
    /// @param amount Total amount to be added to the subscription.
    function addTo(address user, uint256 amount) public {
        require(amount > 0, 'amount must be positive');
        require(user != address(0), 'user is null');

        Subscription memory sub = subscriptions[user];
        require(sub.start != 0, 'no subscription found');
        require(sub.rate != 0, 'cannot extend a zero rate subscription');

        uint64 oldEnd = sub.end;
        uint64 newEnd = oldEnd + uint64(amount / sub.rate);
        require(block.timestamp < newEnd, 'new end cannot be in the past');

        _setEpochs(sub.start, sub.end, -int128(sub.rate));
        _setEpochs(sub.start, newEnd, int128(sub.rate));

        subscriptions[user].end = newEnd;

        bool success = token.transferFrom(msg.sender, address(this), amount);
        require(success, 'IERC20 token transfer failed');

        emit Extend(user, oldEnd, newEnd, amount);
    }

    function setRecurringPayments(address _recurringPayments) public onlyOwner {
        _setRecurringPayments(_recurringPayments);
    }

    /// @param _user Subscription owner.
    /// @param _signer Address authorized to sign messages on the owners behalf.
    /// @return isAuthorized True if the given signer is set as an authorized signer for the given
    /// user, false otherwise.
    function checkAuthorizedSigner(
        address _user,
        address _signer
    ) public view returns (bool) {
        if (_user == _signer) {
            return true;
        }
        return authorizedSigners[_user][_signer];
    }

    /// @param _timestamp Block timestamp, in seconds.
    /// @return epoch Epoch number, rouded up to the next epoch Boundary.
    function timestampToEpoch(
        uint256 _timestamp
    ) public view returns (uint256) {
        return (_timestamp / epochSeconds) + 1;
    }

    /// @return epoch Current epoch number, rouded up to the next epoch Boundary.
    function currentEpoch() public view returns (uint256) {
        return timestampToEpoch(block.timestamp);
    }

    /// @dev Defined as `rate * max(0, min(now, end) - start)`.
    /// @param _subStart Start timestamp of the active subscription.
    /// @param _subEnd End timestamp of the active subscription.
    /// @param _subRate Active subscription rate.
    /// @return lockedTokens Amount of locked tokens for the given subscription, which are
    /// collectable by the contract owner and are not recoverable by the user.
    function locked(
        uint64 _subStart,
        uint64 _subEnd,
        uint128 _subRate
    ) public view returns (uint128) {
        uint256 len = uint256(
            SignedMath.max(
                0,
                int256(Math.min(block.timestamp, _subEnd)) - int64(_subStart)
            )
        );
        return _subRate * uint128(len);
    }

    /// @dev Defined as `rate * max(0, min(now, end) - start)`.
    /// @param _user Address of the active subscription owner.
    /// @return lockedTokens Amount of locked tokens for the given subscription, which are
    /// collectable by the contract owner and are not recoverable by the user.
    function locked(address _user) public view returns (uint128) {
        Subscription storage sub = subscriptions[_user];
        return locked(sub.start, sub.end, sub.rate);
    }

    /// @dev Defined as `rate * max(0, end - max(now, start))`.
    /// @param _subStart Start timestamp of the active subscription.
    /// @param _subEnd End timestamp of the active subscription.
    /// @param _subRate Active subscription rate.
    /// @return unlockedTokens Amount of unlocked tokens, which are recoverable by the user, and are
    /// not collectable by the contract owner.
    function unlocked(
        uint64 _subStart,
        uint64 _subEnd,
        uint128 _subRate
    ) public view returns (uint128) {
        uint256 len = uint256(
            SignedMath.max(
                0,
                int256(int64(_subEnd)) -
                    int256(Math.max(block.timestamp, _subStart))
            )
        );
        return _subRate * uint128(len);
    }

    /// @dev Defined as `rate * max(0, end - max(now, start))`.
    /// @param _user Address of the active subscription owner.
    /// @return unlockedTokens Amount of unlocked tokens, which are recoverable by the user, and are
    /// not collectable by the contract owner.
    function unlocked(address _user) public view returns (uint128) {
        Subscription storage sub = subscriptions[_user];
        return unlocked(sub.start, sub.end, sub.rate);
    }

    /// @notice Sets the recurring payments contract address.
    /// @param _recurringPayments Address of the recurring payments contract.
    function _setRecurringPayments(address _recurringPayments) private {
        require(
            _recurringPayments != address(0),
            'recurringPayments cannot be zero address'
        );
        recurringPayments = _recurringPayments;
        emit RecurringPaymentsUpdated(_recurringPayments);
    }

    /// @notice Create a subscription for a user
    /// Will override an active subscription if one exists.
    /// @dev Note that setting a start time in the past is allowed. If this behavior is not desired,
    /// the caller can clamp the start time to the current block timestamp.
    /// @param user Owner for the new subscription.
    /// @param start Start timestamp for the new subscription.
    /// @param end End timestamp for the new subscription.
    /// @param rate Rate for the new subscription.
    function _subscribe(
        address user,
        uint64 start,
        uint64 end,
        uint128 rate
    ) private {
        require(user != address(0), 'user is null');
        require(user != address(this), 'invalid user');
        require(start < end, 'start must be less than end');

        // This avoids unexpected behavior from truncation, especially in `locked` and `unlocked`.
        require(end <= uint64(type(int64).max), 'end too large');

        // Overwrite an active subscription if there is one
        if (subscriptions[user].end > block.timestamp) {
            // Note: This could potentially lead to a reentrancy vulnerability, since `_unsubscribe`
            // may call `token.transfer` here prior to contract state changes below. Consider the
            // following scenario:
            //   - The user has an active subscription, and `_unsubscribe` is called here.
            //   - Tokens are transfered to the user (for a refund), giving an opportunity for
            //     reentrancy.
            //   - This reentrancy occurs before `subscriptions[user]` is modified, and the new
            //     epoch state gets updated.
            // However, this would cause the attacker to lose money, as their old subscription data
            // is overwritten with the new, with no chance to retrieve the funds for the old.
            _unsubscribe(user);
        }

        subscriptions[user] = Subscription({
            start: start,
            end: end,
            rate: rate
        });
        _setEpochs(start, end, int128(rate));

        uint256 subTotal = rate * (end - start);
        bool success = token.transferFrom(msg.sender, address(this), subTotal);
        require(success, 'IERC20 token transfer failed');

        uint256 epoch = currentEpoch();
        emit Subscribe(user, epoch, start, end, rate);
    }

    /// @notice Remove the user's subscription. Unlocked tokens will be transfered to the user.
    /// @param user Owner of the subscription to be removed.
    function _unsubscribe(address user) private {
        Subscription storage sub = subscriptions[user];
        require(sub.start != 0, 'no active subscription');

        uint64 _now = uint64(block.timestamp);
        require(sub.end > _now, 'Subscription has expired');

        uint128 tokenAmount = unlocked(sub.start, sub.end, sub.rate);

        _setEpochs(sub.start, sub.end, -int128(sub.rate));
        if (sub.start <= _now) {
            _setEpochs(sub.start, _now, int128(sub.rate));
            subscriptions[user].end = _now;
        } else {
            delete subscriptions[user];
        }

        bool success = token.transfer(user, tokenAmount);
        require(success, 'IERC20 token transfer failed');

        uint256 epoch = currentEpoch();
        emit Unsubscribe(user, epoch);
    }

    function _setEpochs(uint64 start, uint64 end, int128 rate) private {
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
            epochs[e1].extra -=
                rate *
                int64(start - (uint64(e1 - 1) * epochSeconds));
        }
        uint256 e2 = timestampToEpoch(end);
        if (e <= e2) {
            epochs[e2].delta -= rate * int64(epochSeconds);
            epochs[e2].extra +=
                rate *
                int64(end - (uint64(e2 - 1) * epochSeconds));
        }
    }
}
