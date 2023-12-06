import {HardhatUserConfig, task} from 'hardhat/config';
import '@nomiclabs/hardhat-ethers';
import '@typechain/hardhat';
import '@nomiclabs/hardhat-etherscan';
import './tasks/deploy';
import './tasks/registry';

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
  {
    network: 'arbitrum-sepolia',
    chainId: 421614,
    url: 'https://sepolia-rollup.arbitrum.io/rpcblock',
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
        auto: true,
        mempool: {
          order: 'fifo',
        },
      },
    },
  },
  solidity: {
    compilers: [
      {
        version: '0.6.12',
        settings: {
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
      },
      {
        version: '0.8.19',
        settings: {
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
      },
    ],
  },
  typechain: {
    outDir: 'types',
    target: 'ethers-v5',
  },
  etherscan: {
    apiKey: {
      arbitrumOne: process.env.ARBISCAN_API_KEY!,
      arbitrumGoerli: process.env.ARBISCAN_API_KEY!,
      arbitrumSepolia: process.env.ARBISCAN_API_KEY!,
    },
    customChains: [
      {
        network: 'arbitrumSepolia',
        chainId: 421614,
        urls: {
          apiURL: 'https://api-sepolia.arbiscan.io/api',
          browserURL: 'https://sepolia.arbiscan.io',
        },
      },
    ],
  },
};

setupNetworkConfig(config);

export default config;
