import {expect} from 'chai';
import {BigNumber} from 'ethers';
import {ethers, network} from 'hardhat';
import {xoshiro128ss} from './rng';
import {GraphToken__factory} from '@graphprotocol/contracts/dist/types/factories/GraphToken__factory';
import {hexDataSlice, randomBytes} from 'ethers/lib/utils';

interface Subscription {
  firstBlock: number;
  lastBlock: number;
  pricePerBlock: BigNumber;
}

const genInt = (rng: () => number, min: number, max: number): number =>
  Math.floor(rng() * (max - min + 1) + min);

it('Model Test', async () => {
  const signers = await ethers.getSigners();
  const owner = signers[0];
  const clients = signers.slice(1, 4);
  const minValueExp = 18 - 6;
  const maxPriceExp = 18 + 2;
  const epochShift = 2;

  const seed = hexDataSlice(randomBytes(32), 0);
  // const seed =
  //   '0x71286e79ed412af8e94801f408d86f8901909df212e3b9125d003384382f2700';

  const rng = xoshiro128ss(seed);
  let currentBlock = 1;
  let pricePerBlock = BigNumber.from(10).pow(18 - 3);
  const subscriptions: Array<Array<Subscription>> = clients.map(_ => []);
  let totalBalance = BigNumber.from(0);
  let unlockedBalance = BigNumber.from(0);

  console.log({
    seed,
    owner: owner.address,
    clients: clients.map(s => s.address),
    minValueExp,
    maxPriceExp,
    epochShift,
    initialPricePerBlock: pricePerBlock.toString(),
  });

  const initialClientBalance = BigNumber.from(10).pow(18 + 6);
  const token = await new GraphToken__factory(owner).deploy(
    initialClientBalance.mul(clients.length)
  );
  const contract = await (
    await ethers.getContractFactory('Subscriptions')
  ).deploy(
    token.address,
    BigNumber.from(10).pow(minValueExp),
    epochShift,
    pricePerBlock
  );

  await network.provider.send('evm_mine');
  for (const client of clients) {
    await token.transfer(client.address, initialClientBalance);
    await token.connect(client).approve(contract.address, initialClientBalance);
  }
  await network.provider.send('evm_mine');
  currentBlock += 2;

  while (currentBlock <= 16) {
    console.log('--- Block', currentBlock, '---');
    for (let i = 0; i < genInt(rng, 0, 3); i++) {
      switch (genInt(rng, 0, 2)) {
        case 0:
          pricePerBlock = BigNumber.from(10).pow(
            genInt(rng, minValueExp, maxPriceExp)
          );
          console.log(`SetPrice(${pricePerBlock.toString()})`);
          await contract.setPricePerBlock(pricePerBlock);

          break;

        case 1:
          const client = genInt(rng, 0, clients.length - 1);
          const subscriber = clients[client];
          const lastBlock = currentBlock + genInt(rng, 1, 10);
          totalBalance = totalBalance.add(
            pricePerBlock.mul(lastBlock - currentBlock + 1)
          );
          subscriptions[client].push({
            firstBlock: currentBlock,
            lastBlock,
            pricePerBlock,
          });
          console.log(
            `Subscribe(${subscriber.address}, ${currentBlock}, ${lastBlock})`
          );
          await contract
            .connect(subscriber)
            .subscribe(subscriber.address, lastBlock);

          break;

        case 2:
          const lastEpochBlock =
            currentBlock - (currentBlock % (1 << epochShift)) - 1;
          // console.log(
          //   subscriptions.flatMap(s => s.map(s => [s.firstBlock, s.lastBlock]))
          // );
          for (const i in subscriptions) {
            for (const sub of subscriptions[i]) {
              if (sub.firstBlock > lastEpochBlock) continue;
              const collectableBlocks =
                Math.min(sub.lastBlock, lastEpochBlock) - sub.firstBlock + 1;
              // console.log('collect', collectableBlocks);
              unlockedBalance = unlockedBalance.add(
                sub.pricePerBlock.mul(collectableBlocks)
              );
            }
            subscriptions[i] = subscriptions[i]
              .filter(s => s.lastBlock > lastEpochBlock)
              .map(s => ({
                ...s,
                firstBlock: Math.max(s.firstBlock, lastEpochBlock + 1),
              }));
          }
          // console.log(
          //   subscriptions.flatMap(s => s.map(s => [s.firstBlock, s.lastBlock]))
          // );
          console.log(`Collect`);
          await contract.collect();

          break;

        // TODO: Unsubscribe (refund)

        default:
          throw 'unreachable';
      }
    }

    currentBlock += 1;
    await network.provider.send('evm_mine');

    const block = await ethers.provider.getBlockWithTransactions('latest');
    for (const transaction of block.transactions) {
      const tx = await network.provider.send('debug_traceTransaction', [
        transaction.hash,
      ]);
      if (tx.failed) {
        console.log(
          contract.interface.decodeFunctionResult(
            transaction.data.slice(0, 10),
            `0x${tx.returnValue}`
          )
        );
      }
    }

    const prevBlock = currentBlock - 1;
    expect(await ethers.provider.getBlockNumber()).eq(prevBlock);
    for (const i in clients) {
      const isSubscribed = await contract.isSubscribed(clients[i].address);
      const shouldBeSubscribed = subscriptions[i].some(
        s => prevBlock <= s.lastBlock
      );
      expect(shouldBeSubscribed, 'subscription status').eq(isSubscribed);
    }
    expect(totalBalance, 'balance').eq(await token.balanceOf(contract.address));
    expect(unlockedBalance, 'unlocked').eq(await contract.unlockedTokens());
  }
});
