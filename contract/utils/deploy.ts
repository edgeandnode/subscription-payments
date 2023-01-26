import { Contract, Signer, ContractFactory, utils, BigNumber } from 'ethers'
import path from 'path'
import { Artifacts } from 'hardhat/internal/artifacts'
import { LinkReferences } from 'hardhat/types'

import { Subscriptions } from '../types/contracts/Subscriptions'

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
): Promise<Contract> {
  if (sender.provider === undefined) throw new Error('Sender has no provider')

  // Deploy
  const artifact = loadArtifact(name)
  const factory = new ContractFactory(artifact.abi, artifact.bytecode)
  const contract = await factory.connect(sender).deploy(...args)
  const txHash = contract.deployTransaction.hash
  console.log(`> Deploy ${name}, txHash: ${txHash}`)

  // Receipt
  const creationCodeHash = hash(factory.bytecode)
  const runtimeCodeHash = hash(await sender.provider.getCode(contract.address))
  console.log('= CreationCodeHash: ', creationCodeHash)
  console.log('= RuntimeCodeHash: ', runtimeCodeHash)
  console.log(`${name} has been deployed to address: ${contract.address}`)

  return contract as unknown as Promise<Contract>
}

// Pass the args in order to this func
export async function deploySubscriptions(
  args: Array<string | BigNumber>,
  sender: Signer,
): Promise<Subscriptions> {
  return deployContract(args, sender, 'Subscriptions') as unknown as Promise<Subscriptions>
}