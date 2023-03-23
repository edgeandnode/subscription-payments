import {Address, Bytes, BigInt, ethereum} from '@graphprotocol/graph-ts';

let defaultAddress = Address.fromString(
  '0xA16081F360e3847006dB660bae1c6d1b2e17eC2A'
);
let defaultAddressBytes = defaultAddress as Bytes;
let defaultBigInt = BigInt.fromI32(1);

function firstBlock(): ethereum.Block {
  return new ethereum.Block(
    defaultAddressBytes,
    defaultAddressBytes,
    defaultAddressBytes,
    defaultAddress,
    defaultAddressBytes,
    defaultAddressBytes,
    defaultAddressBytes,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt
  );
}

function nextBlock(parent: ethereum.Block): ethereum.Block {
  const blockNumber = parent.number.plus(BigInt.fromU32(1));
  const gasUsed = defaultBigInt;
  const gasLimit = defaultBigInt;
  const timestamp = parent.timestamp.plus(BigInt.fromU32(1000));

  return new ethereum.Block(
    defaultAddressBytes,
    parent.hash,
    parent.hash,
    defaultAddress,
    defaultAddressBytes,
    defaultAddressBytes,
    defaultAddressBytes,
    blockNumber,
    gasUsed,
    gasLimit,
    timestamp,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt,
    defaultBigInt
  );
}

class CurrentBlock {
  public parent: ethereum.Block = firstBlock();
  constructor(public current: ethereum.Block = firstBlock()) {}
  next(): void {
    this.parent = this.current;
    this.current = nextBlock(this.current);
  }
  reset(): void {
    this.current = firstBlock();
  }
}

export const mockBlock = new CurrentBlock();
