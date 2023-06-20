import { Contract, Signer, ContractFactory, utils, BigNumber } from 'ethers'
import path from 'path'
import { Artifacts } from 'hardhat/internal/artifacts'
import { LinkReferences } from 'hardhat/types'

import { Subscriptions } from '../types/contracts/Subscriptions'
import { Registry, StableToken } from '../types'

type Abi = Array<string | utils.FunctionFragment | utils.EventFragment | utils.ParamType>

type Artifact = {
  contractName: string
  abi: Abi
  bytecode: string
  deployedBytecode: string
  linkReferences?: LinkReferences
  deployedLinkReferences?: LinkReferences
}

const ARTIFACTS_PATH = path.resolve('artifacts')
const artifacts = new Artifacts(ARTIFACTS_PATH)

const loadArtifact = (name: string): Artifact => {
  return artifacts.readArtifactSync(name)
}

const hash = (input: string): string => utils.keccak256(`0x${input.replace(/^0x/, '')}`)

async function deployContract(
  args: Array<string | BigNumber>,
  sender: Signer,
  name: string,
  logging = true
): Promise<Contract> {
  if (sender.provider === undefined) throw new Error('Sender has no provider')

  // Deploy
  const artifact = loadArtifact(name)
  const factory = new ContractFactory(artifact.abi, artifact.bytecode)
  const contract = await factory.connect(sender).deploy(...args)
  const txHash = contract.deployTransaction.hash
  if (logging) {
    console.log(`> Deploy ${name}, txHash: ${txHash}`)
  }

  // Receipt
  const creationCodeHash = hash(factory.bytecode)
  const runtimeCodeHash = hash(await sender.provider.getCode(contract.address))
  if (logging) {
    console.log('= CreationCodeHash: ', creationCodeHash)
    console.log('= RuntimeCodeHash: ', runtimeCodeHash)
    console.log(`${name} has been deployed to address: ${contract.address}`)
  }

  return contract as unknown as Promise<Contract>
}

// Pass the args in order to this func
export async function deploySubscriptions(
  args: Array<string | BigNumber>,
  sender: Signer,
  logging = true
): Promise<Subscriptions> {
  return deployContract(args, sender, 'Subscriptions', logging) as unknown as Promise<Subscriptions>
}

// Pass the args in order to this func
export async function deployStableToken(
  args: Array<string | BigNumber>,
  sender: Signer,
  logging = true
): Promise<StableToken> {
  return deployContract(args, sender, 'StableToken', logging) as unknown as Promise<StableToken>
}

export async function deployRegistry(
  sender: Signer,
  logging = true
): Promise<Registry> {
  return deployContract([], sender, 'Registry', logging) as unknown as Promise<Registry>
}
