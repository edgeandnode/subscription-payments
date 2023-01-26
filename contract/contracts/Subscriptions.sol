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

library Prelude {
    function min(int256 a, int256 b) internal pure returns (int256) {
        return a <= b ? a : b;
    }

    function max(int256 a, int256 b) internal pure returns (int256) {
        return a >= b ? a : b;
    }
}

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
    mapping(address => Subscription) private _subscriptions;
    // Mapping of epoch numbers to their payloads
    mapping(uint64 => Epoch) private _epochs;
    // The epoch cursor position
    uint64 private _uncollectedEpoch;
    // The epoch cursor value
    int128 private _collectPerEpoch;

    constructor(address token_, uint64 epochBlocks_) {
        token = IERC20(token_);
        owner = msg.sender;
        epochBlocks = epochBlocks_;
        _uncollectedEpoch = uint64(block.number) / epochBlocks_;

        emit Init(token_);
    }

    // Convert block number to epoch number, rounding up to the next epoch
    // boundry.
    function blockToEpoch(uint64 b) private view returns (uint64) {
        int256 value = Prelude.max(
            1,
            int64(b / epochBlocks) + Prelude.min(1, int64(b % epochBlocks))
        );
        return uint64(int64(value));
    }

    // Get the user's most recent subscription.
    function subscription(
        address user
    ) public view returns (Subscription memory) {
        return _subscriptions[user];
    }

    // Locked tokens for a subscription are collectable by the contract owner
    // and cannot be recovered by the user.
    // Defined as `rate * max(0, min(block, end) - start)`
    function locked(Subscription storage sub) private view returns (uint128) {
        uint64 currentBlock = uint64(block.number);
        int256 len = Prelude.max(
            0,
            Prelude.min(int64(currentBlock), int64(sub.end)) - int64(sub.start)
        );
        return sub.rate * uint128(uint256(len));
    }

    // Unlocked tokens for a subscription are not collectable by the contract
    // owner and can be recovered by the user.
    // Defined as `rate * max(0, end - max(block, start))`
    function unlocked(Subscription storage sub) private view returns (uint128) {
        uint64 currentBlock = uint64(block.number);
        int256 len = Prelude.max(
            0,
            int64(sub.end) - Prelude.max(int64(currentBlock), int64(sub.start))
        );
        return sub.rate * uint128(uint256(len));
    }

    // Collect a subset of the locked tokens held by this contract.
    function collect() public {
        require(msg.sender == owner, 'must be called by owner');

        uint64 currentEpoch = blockToEpoch(uint64(block.number));
        int128 total = 0;
        for (; _uncollectedEpoch < currentEpoch; _uncollectedEpoch++) {
            Epoch storage epoch = _epochs[_uncollectedEpoch];
            _collectPerEpoch += epoch.delta;
            total += _collectPerEpoch + epoch.extra;
            delete _epochs[_uncollectedEpoch];
        }

        token.transfer(owner, uint128(total));
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

        uint64 e = blockToEpoch(uint64(block.number));
        uint64 e1 = blockToEpoch(start);
        if (e <= e1) {
            _epochs[e1].delta += rate * int64(epochBlocks);
            _epochs[e1].extra -= rate * int64(start - ((e1 - 1) * epochBlocks));
        }
        uint64 e2 = blockToEpoch(end);
        if (e <= e2) {
            _epochs[e1].delta -= rate * int64(epochBlocks);
            _epochs[e1].extra += rate * int64(end - ((e2 - 1) * epochBlocks));
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
        start = uint64(
            uint256(Prelude.max(int64(start), int64(uint64(block.number))))
        );
        require(start < end, 'start must be less than end');
        require(
            _subscriptions[user].end <= uint64(block.number),
            'active subscription must have ended'
        );

        uint128 subTotal = rate * (end - start);
        token.transferFrom(msg.sender, address(this), subTotal);

        _subscriptions[user] = Subscription({
            start: start,
            end: end,
            rate: rate
        });
        setEpochs(start, end, int128(rate));

        emit Subscribe(user, start, end, rate);
    }

    // Remove a user's subscription.
    function unsubscribe() public {
        address user = msg.sender;
        require(user != address(0), 'user is null');

        Subscription storage sub = _subscriptions[user];
        uint64 currentBlock = uint64(block.number);

        token.transfer(user, unlocked(sub));

        if ((sub.start <= currentBlock) && (currentBlock < sub.end)) {
            setEpochs(sub.start, sub.end, -int128(sub.rate));
            setEpochs(sub.start, currentBlock, int128(sub.rate));
            _subscriptions[user].end = currentBlock;
        } else if (currentBlock < sub.start) {
            setEpochs(sub.start, sub.end, -int128(sub.rate));
            delete _subscriptions[user];
        } else {
            // sub.end <= currentBlock
            delete _subscriptions[user];
        }

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

        uint128 addition = sub.rate * (end - sub.end);
        token.transferFrom(msg.sender, address(this), addition);

        setEpochs(sub.start, sub.end, -int128(sub.rate));
        setEpochs(sub.start, end, int128(sub.rate));

        _subscriptions[user].end = end;

        emit Extend(user, end);
    }
}
