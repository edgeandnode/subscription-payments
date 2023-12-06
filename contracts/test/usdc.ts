import { BigNumber, ethers } from "ethers";
import { FiatTokenV2_1 } from "../types";

export const setupUSDC = async (
  stableToken: FiatTokenV2_1,
  deployer: {
    signer: ethers.Signer;
    address: string;
  }
) => {
  await stableToken
    .connect(deployer.signer)
    .initialize(
      'USD Coin',
      'USDC',
      'USD',
      6,
      deployer.address,
      deployer.address,
      deployer.address,
      deployer.address
    );

  await stableToken
    .connect(deployer.signer)
    .configureMinter(deployer.address, BigNumber.from('10000000000'));
  await stableToken
    .connect(deployer.signer)
    .mint(deployer.address, BigNumber.from('10000000000'));
};