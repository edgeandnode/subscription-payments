// SPDX-License-Identifier: MIT

// Reference: https://ethereum.org/en/developers/docs/evm/opcodes/

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
        require(msg.sender == owner);

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
        require(subscriber != address(0));
        startBlock = uint64(
            uint256(Prelude.max(int64(startBlock), int64(uint64(block.number))))
        );
        require(startBlock < endBlock);
        require(_subscriptions[subscriber].endBlock <= startBlock);

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
        uint64 endBlock = uint64(block.number);
        Subscription storage sub = _subscriptions[subscriber];
        require(sub.endBlock < endBlock);

        token.transfer(subscriber, unlocked(sub));
        _uncollected += locked(sub);
        delete _subscriptions[subscriber];

        emit Unsubscribe(subscriber);
    }
}
