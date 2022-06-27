// SPDX-License-Identifier: MIT

// Reference: https://ethereum.org/en/developers/docs/evm/opcodes/

pragma solidity ^0.8.14;
pragma abicoder v2;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Subscriptions {
    struct Subscription {
        uint64 firstBlock;
        uint64 lastBlock;
        uint128 pricePerBlock;
    }
    struct Epoch {
        uint32 bin;
        int32 delta;
    }

    event Subscribe(address indexed subscriber, uint64 lastBlock, uint128 pricePerBlock);
    event Unsubscribe(address indexed subscriber);

    address public owner;
    IERC20 public token;
    uint128 public minValue;
    uint128 public pricePerBlock;
    uint128 public unlockedTokens;
    uint8 private _epochShift;
    uint64 private _firstEpoch;
    uint128 private _incomePerBlock;
    mapping (address => Subscription[]) private _subscriptions;
    mapping (uint64 => Epoch[4]) private _epochs;

    constructor(address tokenAddress, uint128 _minValue, uint8 epochShift, uint128 _pricePerBlock) {
        owner = msg.sender;
        token = IERC20(tokenAddress);
        minValue = _minValue;
        _epochShift = epochShift;
        pricePerBlock = _pricePerBlock;
        _firstEpoch = _blockToEpoch(uint64(block.number));
    }

    function _blockToEpoch(uint64 _block) private view returns (uint64) {
        return _block >> _epochShift;
    }

    function _blockToEpochRemainder(uint64 _block) private view returns (uint64) {
        return _block & ~(uint64(0xffffffffffffffff) << _epochShift);
    }

    function _epochToBlock(uint64 _epoch) private view returns (uint64) {
        return _epoch << _epochShift;
    }

    function _addEpochValue(uint64 epoch, uint128 value) private {
        uint32 packed = uint32(value / minValue);
        _epochs[epoch / 4][epoch % 4].bin += packed;
    }

    function _addEpochDelta(uint64 epoch, int128 delta) private {
        int32 packed = int32(delta / int128(minValue));
        _epochs[epoch / 4][epoch % 4].delta += packed;
    }

    function isSubscribed(address subscriber) public view returns (bool) {
        Subscription[] storage subscriptions = _subscriptions[subscriber];
        for (uint256 i = 1; i <= subscriptions.length; i++) {
            Subscription memory sub = subscriptions[subscriptions.length - i];
            if ((sub.firstBlock <= block.number) && (block.number <= sub.lastBlock)) {
                return true;
            }
        }
        return false;
    }

    function setPricePerBlock(uint128 _pricePerBlock) public {
        require(tx.origin == owner);
        require(pricePerBlock >= minValue);
        pricePerBlock = _pricePerBlock;
    }

    function subscribe(address subscriber, uint64 lastBlock) public {
        require(subscriber != address(0));
        uint64 firstBlock = uint64(block.number);
        require(lastBlock > firstBlock, "subscription must end after it begins");
        Subscription memory sub = Subscription(firstBlock, lastBlock, pricePerBlock);
        _subscriptions[subscriber].push(sub);
        _updateEpochs(firstBlock, lastBlock);
        token.transferFrom(msg.sender, address(this), (lastBlock - firstBlock + 1) * pricePerBlock);
        emit Subscribe(subscriber, lastBlock, pricePerBlock);
    }

    function _updateEpochs(uint64 firstBlock, uint64 lastBlock) private {
        uint64 firstEpoch = _blockToEpoch(firstBlock);
        uint64 lastEpoch = _blockToEpoch(lastBlock);
        if (firstEpoch == lastEpoch) {
            _addEpochValue(firstEpoch, pricePerBlock * (lastBlock - firstBlock + 1));
            return;
        }
        if (firstBlock > _epochToBlock(firstEpoch)) {
            _addEpochValue(firstEpoch, pricePerBlock * (_epochToBlock(firstEpoch + 1) - firstBlock));
            firstEpoch += 1;
        }
        if (lastBlock < (_epochToBlock(lastEpoch + 1) - 1)) {
            _addEpochValue(lastEpoch, pricePerBlock * (lastBlock - _epochToBlock(lastEpoch) + 1));
            lastEpoch -= 1;
        }
        if (firstEpoch > lastEpoch) {
            return;
        }
        if (firstEpoch == lastEpoch) {
            _addEpochValue(firstEpoch, pricePerBlock << _epochShift);
            return;
        }
        _addEpochDelta(firstEpoch, int128(pricePerBlock));
        _addEpochDelta(lastEpoch + 1, -int128(pricePerBlock));
    }

    function unsubscribe(address subscriber) public {
        require(subscriber != address(0));
        uint64 firstBlock = uint64(block.number);
        assert(false); // TODO
        emit Unsubscribe(subscriber);
    }

    function collect() public {
        uint64 currentEpoch = _blockToEpoch(uint64(block.number));
        for (uint64 i = _firstEpoch; i < currentEpoch; i++) {
            Epoch storage epoch = _epochs[i / 4][i % 4];
            uint128 value = uint128(epoch.bin) * minValue;
            unlockedTokens += value;
            int128 delta = epoch.delta * int128(minValue);
            _incomePerBlock = uint128(int128(_incomePerBlock) + delta);
            unlockedTokens += _incomePerBlock << _epochShift;
            if ((i % 4) == 3) {
                delete _epochs[i / 4];
            }
        }
        _firstEpoch = currentEpoch;
        // TODO: Limit withdrawal to `unlockedTokens`
    }
}
