import {Wallet, ethers} from 'ethers';
import {task, types} from 'hardhat/config';
import {HardhatRuntimeEnvironment} from 'hardhat/types';

import {deploySubscriptions, deployRegistry, deployFiatToken} from '../utils/deploy';
import { setupUSDC } from '../test/usdc';

task('deploy', 'Deploy the subscription contract (use L2 network!)')
  .addParam('token', 'Address of the ERC20 token')
  .addOptionalParam('epochSeconds', 'Epoch length in seconds.', 3, types.int)
  .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
    const accounts = await hre.ethers.getSigners();

    if (accounts.length === 0) {
      throw new Error(
        'No accounts available, set PRIVATE_KEY or MNEMONIC env variables'
      );
    }
    console.log(
      'Deploying subscriptions contract with the account:',
      accounts[0].address
    );

    await deploySubscriptions(
      [taskArgs.token, taskArgs.epochSeconds],
      accounts[0] as unknown as Wallet
    );
  });

task('deploy:registry', 'Deploy the registry contract (use L2 network!)')
  .addOptionalParam('owner', 'Address of the contract owner')
  .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
    const accounts = await hre.ethers.getSigners();

    if (accounts.length === 0) {
      throw new Error(
        'No accounts available, set PRIVATE_KEY or MNEMONIC env variables'
      );
    }
    console.log(
      'Deploying registry contract with the account:',
      accounts[0].address
    );

    const registry = await deployRegistry(accounts[0] as unknown as Wallet);

    if (ethers.utils.isAddress(taskArgs.owner)) {
      console.log(`Transferring ownership to ${taskArgs.owner}`);
      await registry.connect(accounts[0]).transferOwnership(taskArgs.owner);
    }
  });

task('deploy:usdc', 'Deploy the USDC contract (use L2 network!)')
  .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
    const accounts = await hre.ethers.getSigners();

    if (accounts.length === 0) {
      throw new Error(
        'No accounts available, set PRIVATE_KEY or MNEMONIC env variables'
      );
    }
    console.log(
      'Deploying USDC contract with the account:',
      accounts[0].address
    );

    const usdc = await deployFiatToken([], accounts[0] as unknown as Wallet);
    console.log(`USDC deployed at ${usdc.address}`);
    console.log(`Configuring contract...`);
    await setupUSDC(usdc, { address: accounts[0].address, signer: accounts[0] });
  });

