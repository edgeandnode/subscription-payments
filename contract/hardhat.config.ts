import {HardhatUserConfig, task} from 'hardhat/config';
import '@nomiclabs/hardhat-ethers';
import '@nomiclabs/hardhat-waffle';
import '@typechain/hardhat';
import './tasks/deploy';

task('accounts', 'Print a list of accounts', async (_, hre) => {
  const accounts = await hre.ethers.getSigners();
  for (const account of accounts) {
    console.log(account.address);
  }
});

interface NetworkConfig {
  network: string;
  chainId: number;
  url?: string;
  gas?: number | 'auto';
  gasPrice?: number | 'auto';
}

const networkConfigs: NetworkConfig[] = [
  {
    network: 'arbitrum-one',
    chainId: 42161,
    url: 'https://arb1.arbitrum.io/rpc',
  },
  {
    network: 'arbitrum-goerli',
    chainId: 421613,
    url: 'https://goerli-rollup.arbitrum.io/rpc',
  },
];

function getAccountsKeys() {
  if (process.env.MNEMONIC) return {mnemonic: process.env.MNEMONIC};
  if (process.env.PRIVATE_KEY) return [process.env.PRIVATE_KEY];
  return 'remote';
}

function getProviderURL(network: string) {
  return `https://${network}.infura.io/v3/${process.env.INFURA_KEY}`;
}

function setupNetworkConfig(config: HardhatUserConfig) {
  if (config.networks == null) {
    config.networks = {};
  }
  for (const netConfig of networkConfigs) {
    config.networks[netConfig.network] = {
      chainId: netConfig.chainId,
      url: netConfig.url ? netConfig.url : getProviderURL(netConfig.network),
      gas: netConfig.gas || 'auto',
      gasPrice: netConfig.gasPrice || 'auto',
      accounts: getAccountsKeys(),
    };
  }
}

const config: HardhatUserConfig = {
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
  typechain: {
    outDir: 'types',
    target: 'ethers-v5',
  },
};

setupNetworkConfig(config);

export default config;
