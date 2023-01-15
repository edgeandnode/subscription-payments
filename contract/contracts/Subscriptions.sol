// SPDX-License-Identifier: MIT

/* TODO: turn this into a more coherent set of docs

- This contract is designed to allow users of the Graph Protocol to pay gateways
for their services with limited risk of losing funds.

- This contract makes no assumptions about how the subscription price per block
is interpreted by the gateway.

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

    function clamp(
        uint64 x,
        uint64 a,
        uint64 b
    ) internal pure returns (uint64) {
        return uint64(int64(max(int64(a), min(int64(b), int64(x)))));
    }
}

contract Subscriptions {
    struct Subscription {
        uint64 startBlock;
        uint64 endBlock;
        uint128 pricePerBlock;
    }

    event Subscribe(
        address indexed subscriber,
        uint64 startBlock,
        uint64 endBlock,
        uint128 pricePerBlock
    );
    event Unsubscribe(address indexed subscriber);
    event Extend(address indexed subscriber, uint64 endBlock);

    IERC20 public token;
    address public owner;
    uint128 private _uncollected;
    mapping(address => Subscription) private _subscriptions;
    // TODO: use epochs for more efficient collection.
    address[] private _subscribers;

    constructor(address tokenAddress) {
        token = IERC20(tokenAddress);
        owner = msg.sender;
    }

    function subscription(
        address subscriber
    ) public view returns (Subscription memory) {
        return _subscriptions[subscriber];
    }

    function locked(Subscription storage sub) private view returns (uint128) {
        uint64 currentBlock = uint64(block.number);
        int256 len = Prelude.max(
            0,
            Prelude.min(int64(currentBlock), int64(sub.endBlock)) -
                int64(sub.startBlock)
        );
        return sub.pricePerBlock * uint128(uint256(len));
    }

    function unlocked(Subscription storage sub) private view returns (uint128) {
        uint64 currentBlock = uint64(block.number);
        int256 len = Prelude.max(
            0,
            int64(sub.endBlock) -
                Prelude.max(int64(currentBlock), int64(sub.startBlock))
        );
        return sub.pricePerBlock * uint128(uint256(len));
    }

    function collect() public {
        require(msg.sender == owner, 'must be called by owner');

        uint64 currentBlock = uint64(block.number);
        uint i = 0;
        while (i < _subscribers.length) {
            Subscription storage sub = _subscriptions[_subscribers[i]];
            _uncollected += locked(sub);
            if (sub.endBlock <= currentBlock) {
                delete _subscriptions[_subscribers[i]];
                uint last = _subscribers.length - 1;
                _subscribers[i] = _subscribers[last];
                _subscribers.pop();
            } else {
                _subscriptions[_subscribers[i]] = Subscription({
                    startBlock: Prelude.clamp(
                        currentBlock,
                        sub.startBlock,
                        sub.endBlock
                    ),
                    endBlock: sub.endBlock,
                    pricePerBlock: sub.pricePerBlock
                });
                i += 1;
            }
        }

        token.transfer(owner, _uncollected);
        _uncollected = 0;
    }

    function subscribe(
        address subscriber,
        uint64 startBlock,
        uint64 endBlock,
        uint128 pricePerBlock
    ) public {
        // This can be called by any account for a given subscriber, because it
        // requires that the subscription's startBlock is less than the current
        // block.

        require(subscriber != address(0), 'subscriber is null');
        startBlock = uint64(
            uint256(Prelude.max(int64(startBlock), int64(uint64(block.number))))
        );
        require(startBlock < endBlock, 'startBlock must be less than endBlock');
        require(
            _subscriptions[subscriber].endBlock <= uint64(block.number),
            'active subscription must have ended'
        );

        uint128 subTotal = pricePerBlock * (endBlock - startBlock);
        token.transferFrom(msg.sender, address(this), subTotal);

        Subscription storage prev = _subscriptions[subscriber];
        _uncollected += prev.pricePerBlock * (prev.endBlock - prev.startBlock);

        _subscriptions[subscriber] = Subscription({
            startBlock: startBlock,
            endBlock: endBlock,
            pricePerBlock: pricePerBlock
        });
        _subscribers.push(subscriber);

        emit Subscribe(subscriber, startBlock, endBlock, pricePerBlock);
    }

    function unsubscribe() public {
        address subscriber = msg.sender;
        Subscription storage sub = _subscriptions[subscriber];

        token.transfer(subscriber, unlocked(sub));
        _uncollected += locked(sub);
        delete _subscriptions[subscriber];

        emit Unsubscribe(subscriber);
    }

    function extend(address subscriber, uint64 endBlock) public {
        require(subscriber != address(0), 'subscriber is null');
        uint64 currentBlock = uint64(block.number);
        Subscription storage sub = _subscriptions[subscriber];
        require(
            (sub.startBlock <= currentBlock) && (currentBlock < sub.endBlock),
            'current subscription must be active'
        );
        require(
            sub.endBlock < endBlock,
            'endBlock must be after that of the current subscription'
        );

        uint128 addition = sub.pricePerBlock * (endBlock - sub.endBlock);
        token.transferFrom(msg.sender, address(this), addition);

        _subscriptions[subscriber].endBlock = endBlock;

        emit Extend(subscriber, endBlock);
    }
}
