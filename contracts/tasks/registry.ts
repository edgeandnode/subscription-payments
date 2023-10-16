import {Wallet, ethers} from 'ethers';
import {task, types} from 'hardhat/config';
import {HardhatRuntimeEnvironment} from 'hardhat/types';
import {loadArtifact} from '../utils/artifacts';
import {Registry} from '../types/contracts/Registry';

import addresses from '../addresses.json';

task('registry:insert', 'Insert an entry to the registry')
  .addParam('entryId', 'Entry ID to insert', undefined, types.int)
  .addParam('subscriptions', 'Comma separated list of subscription contract addresses to insert', undefined, types.string)
  .addParam('metadataHash', 'Metadata hash to insert', undefined, types.string)
  .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
    const accounts = await hre.ethers.getSigners();

    if (accounts.length === 0) {
      throw new Error(
        'No accounts available, set PRIVATE_KEY or MNEMONIC env variables'
      );
    }
    console.log(
      'Using the account:',
      accounts[0].address
    );

    const chainId = (hre.network.config.chainId as number).toString();
    const registryAddress = (addresses as any)[chainId]['Registry'];

    const artifact = loadArtifact('Registry');
    const registry = new ethers.Contract(
      registryAddress,
      artifact.abi,
      hre.ethers.provider
    ) as Registry;

    console.log(`Registry contract address: ${registry.address}`);

    const tx = await registry.connect(accounts[0]).insertEntry(taskArgs.entryId, {
      subscriptions: taskArgs.subscriptions.split(','),
      metadataHash: taskArgs.metadataHash,
    })

    const receipt = await tx.wait();
    console.log(`Transaction successful: ${receipt.transactionHash}`);

  });
