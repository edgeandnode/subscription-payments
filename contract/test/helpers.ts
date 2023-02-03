import { BigNumber } from "ethers";
import { network } from "hardhat";

export const latestBlockNumber = async () => BigNumber.from(await network.provider.send(
  'eth_blockNumber',
  []
));

export const nextBlockNumber = async () => (await latestBlockNumber()).add(1);

export const maxBN = (a: BigNumber, b: BigNumber) => a.gt(b) ? a : b;
export const minBN = (a: BigNumber, b: BigNumber) => a.lt(b) ? a : b;