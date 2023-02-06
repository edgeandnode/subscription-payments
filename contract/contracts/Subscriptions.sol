// SPDX-License-Identifier: MIT

/* TODO: turn this into a more coherent set of docs.
- This contract is designed to allow users of the Graph Protocol to pay gateways
for their services with limited risk of losing tokens.
- This contract makes no assumptions about how the subscription price per block
is interpreted by the gateway.
*/

/* TODO:
- Make sure we can support multi-sigs, etc.
- Implement resubscribe (update subscription) operation to reduce user friction.
*/

pragma solidity ^0.8.17;
pragma abicoder v2;

import '@openzeppelin/contracts/token/ERC20/IERC20.sol';
import '@openzeppelin/contracts/utils/math/Math.sol';
import '@openzeppelin/contracts/utils/math/SignedMath.sol';

contract Subscriptions {
    // A Subscription represents a lockup of `rate` tokens per block for the
    // half-open block range [start, end).
    struct Subscription {
        uint64 start;
        uint64 end;
        uint128 rate;
    }
    // An epoch defines the end of a span of blocks, the length of which is
    // defined by `epochBlocks`. These exist to facilitate a relatively
    // efficient `collect` implementation while allowing users to recover
    // unlocked tokens at a block granularity.
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

    // The ERC-20 token held by this contract
    IERC20 public token;
    // The owner of the contract, which has the authority to call collect
    address public owner;
    // The block length of each epoch
    uint64 public epochBlocks;
    // Mapping of users to their most recent subscription
    mapping(address => Subscription) public _subscriptions;
    // Mapping of epoch numbers to their payloads
    mapping(uint256 => Epoch) public _epochs;
    // The epoch cursor position
    uint64 public _uncollectedEpoch;
    // The epoch cursor value
    int128 public _collectPerEpoch;

    constructor(address token_, uint64 epochBlocks_) {
        token = IERC20(token_);
        owner = msg.sender;
        epochBlocks = epochBlocks_;
        _uncollectedEpoch = uint64(block.number) / epochBlocks_;

        emit Init(token_);
    }

    // Convert block number to epoch number, rounding up to the next epoch
    // boundry.
    function blockToEpoch(uint256 b) public view returns (uint256) {
        return Math.max(1, (b / epochBlocks) + Math.min(1, (b % epochBlocks) + 1));
    }

    /**
     * Locked tokens for a subscription are collectable by the contract owner
     * and cannot be recovered by the user.
     * Defined as `rate * max(0, min(block, end) - start)`
     * @param _subStart the start block of the active subscription
     * @param _subEnd the end block of the active subscription
     * @param _subRate the active subscription rate
     * @return lockedTokens the amount of locked tokens in the active subscription
     */
    function locked(uint64 _subStart, uint64 _subEnd, uint128 _subRate) public view returns (uint128) {
        uint256 len = uint256(
            SignedMath.max(
                0,
                int256(Math.min(block.number, _subEnd)) - int64(_subStart)
            )
        );
        return _subRate * uint128(len);
    }

    /**
     * Locked tokens for a subscription are collectable by the contract owner
     * and cannot be recovered by the user.
     * Defined as `rate * max(0, min(block, end) - start)`
     * @param _user address of the active subscription owner
     * @return lockedTokens the amount of locked tokens in the active subscription
     */
    function locked(address _user) public view returns (uint128) {
        Subscription storage sub = _subscriptions[_user];
        
        return locked(sub.start, sub.end, sub.rate);
    }

    /**
     * Unlocked tokens for a subscription are not collectable by the contract
     * owner and can be recovered by the user.
     * Defined as `rate * max(0, end - max(block, start))`
     * @param _subStart the start block of the active subscription
     * @param _subEnd the end block of the active subscription
     * @param _subRate the active subscription rate
     * @return unlockedTokens amount of unlocked tokens recoverable by the user
     */
    function unlocked(uint64 _subStart, uint64 _subEnd, uint128 _subRate) public view returns (uint128) {
        uint256 len = uint256(
            SignedMath.max(
                0,
                int256(int64(_subEnd)) -
                    int256(Math.max(block.number, _subStart))
            )
        );
        return _subRate * uint128(len);
    }

    /**
     * Unlocked tokens for a subscription are not collectable by the contract
     * owner and can be recovered by the user.
     * Defined as `rate * max(0, end - max(block, start))`
     * @param _user address of the active subscription owner
     * @return unlockedTokens amount of unlocked tokens recoverable by the user
     */
    function unlocked(address _user) public view returns (uint128) {
        Subscription storage sub = _subscriptions[_user];

        return unlocked(sub.start, sub.end, sub.rate);
    }

    /**
     * Collect a subset of the locked tokens held by this contract.
     * @param _block block used to calculate the epoch to collect the subset of locked tokens
     */
    function collect(uint256 _block) public {
        require(msg.sender == owner, 'must be called by owner');

        uint256 currentEpoch = blockToEpoch(_block);
        int128 total = 0;
        while (_uncollectedEpoch < currentEpoch) {
            Epoch storage epoch = _epochs[_uncollectedEpoch];
            _collectPerEpoch += epoch.delta;
            total += _collectPerEpoch + epoch.extra;
            delete _epochs[_uncollectedEpoch];

            unchecked { ++_uncollectedEpoch; }
        }

        bool success = token.transfer(owner, uint128(total));
        require(success, 'IERC20 token transfer failed');
    }

    function collect() public {
        collect(block.number);
    }

    function setEpochs(uint64 start, uint64 end, int128 rate) private {
        /*
        Example subscription layout using
            epochBlocks = 6
            sub = {start: 2, end: 9, rate: 1}

        blocks: |0 |1 |2 |3 |4 |5 |6 |7 |8 |9 |10|11|
                                      ^ currentBlock
                       ^start               ^end
        epochs: |                1|                2|
                               e1^               e2^
        */

        uint256 e = blockToEpoch(block.number);
        uint256 e1 = blockToEpoch(start);
        if (e <= e1) {
            _epochs[e1].delta += rate * int64(epochBlocks);
            _epochs[e1].extra -= rate * int64(start - ((uint64(e1) - 1) * epochBlocks));
        }
        uint256 e2 = blockToEpoch(end);
        if (e <= e2) {
            _epochs[e2].delta -= rate * int64(epochBlocks);
            _epochs[e2].extra += rate * int64(end - ((uint64(e2) - 1) * epochBlocks));
        }
    }

    // Set the subscription for a user.
    function subscribe(
        address user,
        uint64 start,
        uint64 end,
        uint128 rate
    ) public {
        // This can be called by any account for a given user, because it
        // requires that the active subscription start is less than the current
        // block. Otherwise, this contract might not be able to recover the
        // unlocked tokens for the active subscription we're overwriting.

        require(user != address(0), 'user is null');
        require(user != address(this), 'invalid user');
        start = uint64(Math.max(start, block.number));
        require(start < end, 'start must be less than end');
        // user can bypass this check incase their subscription gets griefed
        require(
            _subscriptions[user].end <= uint64(block.number) || user == msg.sender,
            'active subscription must have ended'
        );

        _subscriptions[user] = Subscription({
            start: start,
            end: end,
            rate: rate
        });
        setEpochs(start, end, int128(rate));

        uint128 subTotal = rate * (end - start);
        bool success = token.transferFrom(msg.sender, address(this), subTotal);
        require(success, 'IERC20 token transfer failed');

        emit Subscribe(user, start, end, rate);
    }

    // Remove a user's subscription.
    function unsubscribe() public {
        address user = msg.sender;
        require(user != address(0), 'user is null');

        Subscription storage sub = _subscriptions[user];
        require(sub.start != 0, 'no active subscription');

        // check if subscription has expired: sub.end <= block.number
        uint64 currentBlock = uint64(block.number);
        require(sub.end > currentBlock, 'Subscription has expired');

        uint128 tokenAmount = unlocked(sub.start, sub.end, sub.rate);

        if ((sub.start <= currentBlock) && (currentBlock < sub.end)) {
            setEpochs(sub.start, sub.end, -int128(sub.rate));
            setEpochs(sub.start, currentBlock, int128(sub.rate));
            _subscriptions[user].end = currentBlock;
        } else if (currentBlock < sub.start) {
            setEpochs(sub.start, sub.end, -int128(sub.rate));
            delete _subscriptions[user];
        }

        bool success = token.transfer(user, tokenAmount);
        require(success, 'IERC20 token transfer failed');

        emit Unsubscribe(user);
    }

    // Extend a user's subscription.
    function extend(address user, uint64 end) public {
        require(user != address(0), 'user is null');
        uint64 currentBlock = uint64(block.number);
        Subscription storage sub = _subscriptions[user];
        require(
            (sub.start <= currentBlock) && (currentBlock < sub.end),
            'current subscription must be active'
        );
        require(
            sub.end < end,
            'end must be after that of the current subscription'
        );

        setEpochs(sub.start, sub.end, -int128(sub.rate));
        setEpochs(sub.start, end, int128(sub.rate));

        uint64 currentEnd = sub.end;
        _subscriptions[user].end = end;

        uint128 addition = sub.rate * (end - currentEnd);
        bool success = token.transferFrom(msg.sender, address(this), addition);
        require(success, 'IERC20 token transfer failed');
        

        emit Extend(user, end);
    }
}
