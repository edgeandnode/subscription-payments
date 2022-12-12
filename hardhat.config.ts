import {task} from 'hardhat/config';
import '@nomiclabs/hardhat-ethers';
import '@nomiclabs/hardhat-waffle';

task('accounts', 'Print a list of accounts', async (_, hre) => {
  const accounts = await hre.ethers.getSigners();
  for (const account of accounts) {
    console.log(account.address);
  }
});

export default {
  solidity: '0.8.17',
  networks: {
    hardhat: {
      mining: {
        auto: false,
        interval: 0,
        mempool: {
          order: 'fifo',
        },
      },
    },
  },
};
