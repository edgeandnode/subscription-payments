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
  defaultNetwork: 'hardhat',
  networks: {
    hardhat: {
      chainId: 1337,
      mining: {
        auto: false,
        interval: 0,
        mempool: {
          order: 'fifo',
        },
      },
    },
    localhost: {
      url: 'http://localhost:8545',
      chainId: 1337,
      mining: {
        auto: false,
        interval: 0,
        mempool: {
          order: 'fifo',
        },
      },
    },
  },
  solidity: {
    version: '0.8.17',
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
};
