import { Wallet } from 'ethers'
import { task, types } from 'hardhat/config'
import { HardhatRuntimeEnvironment } from 'hardhat/types'

import { deploySubscriptions } from '../utils/deploy'

task('deploy', 'Deploy the subscription contract (use L2 network!)')
  .addParam('token', 'Address of the ERC20 token')
  .addOptionalParam('epochBlocks', 'Block length of each epoch.', 3, types.int)
  .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
    const accounts = await hre.ethers.getSigners()

    if (accounts.length === 0) {
      throw new Error('No accounts available, set PRIVATE_KEY or MNEMONIC env variables')
    }
    console.log('Deploying subscriptions contract with the account:', accounts[0].address);
    
    await deploySubscriptions(
      [taskArgs.token, taskArgs.epochBlocks],
      accounts[0] as unknown as Wallet,
    )
  })