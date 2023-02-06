import { hexValue } from "@ethersproject/bytes";
import { BigNumber } from "ethers";
import { network } from "hardhat";

export const latestBlockNumber = async () => BigNumber.from(await network.provider.send(
  'eth_blockNumber',
  []
));

export const nextBlockNumber = async () => (await latestBlockNumber()).add(1);

export const maxBN = (a: BigNumber, b: BigNumber) => a.gt(b) ? a : b;
export const minBN = (a: BigNumber, b: BigNumber) => a.lt(b) ? a : b;

export const mineNBlocks = async (n: number) => await network.provider.send('hardhat_mine', [
  hexValue(BigNumber.from(n)),
]);

export const setAutoMine = async (auto: boolean) => await network.provider.send('evm_setAutomine', [auto]);