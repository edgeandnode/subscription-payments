import hre from 'hardhat'
import { utils, BigNumber, Signer } from 'ethers'
import { formatUnits } from 'ethers/lib/utils'
import { EthereumProvider } from 'hardhat/types'

const { parseUnits } = utils

export const toBN = (value: string | number): BigNumber => BigNumber.from(value)
export const toGRT = (value: string | number): BigNumber => {
  return parseUnits(typeof value === 'number' ? value.toString() : value, '18')
}
export const formatGRT = (value: BigNumber): string => formatUnits(value, '18')
export const floorBN = (value: BigNumber) => BigNumber.from(Math.floor(value.toNumber()))
export const provider = (): EthereumProvider => hre.network.provider

export interface Account {
  readonly signer: Signer
  readonly address: string
}

export const getAccounts = async (): Promise<Account[]> => {
  const accounts = []
  const signers: Signer[] = await hre.ethers.getSigners()
  for (const signer of signers) {
    accounts.push({ signer, address: await signer.getAddress() })
  }
  return accounts
}