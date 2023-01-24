import {GraphToken__factory} from '@graphprotocol/contracts/dist/types/factories/GraphToken__factory';
import {BigNumber} from 'ethers';
import {ethers, network} from 'hardhat';

async function main() {
  const signer = (await ethers.getSigners())[0]!;
  const initialBalance = BigNumber.from(10).pow(18 + 9);
  const token = await new GraphToken__factory(signer).deploy(initialBalance);
  const contract = await (
    await ethers.getContractFactory('Subscriptions')
  ).deploy(token.address, 3);
  await network.provider.send('evm_mine');

  await token.transfer(signer.address, BigNumber.from(10).pow(18 + 6));
  await network.provider.send('evm_mine');

  console.log(
    JSON.stringify(
      {
        chainId: network.config.chainId,
        signer: signer.address,
        initialBalance: initialBalance.toString(),
        token: token.address,
        contract: contract.address,
      },
      undefined,
      2
    )
  );
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
