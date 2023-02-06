import '@nomicfoundation/hardhat-chai-matchers';

import {expect} from 'chai';
import * as deployment from '../utils/deploy';
import {getAccounts, Account, toGRT, toBN, floorBN} from '../utils/helpers';

import {Subscriptions} from '../types/contracts/Subscriptions';
import {StableToken} from '../types/contracts/test/StableMock.sol/StableToken';
import {BigNumber, ethers, utils} from 'ethers';
import {
  latestBlockNumber,
  maxBN,
  mineNBlocks,
  nextBlockNumber,
  setAutoMine,
} from './helpers';

const tenBillion = toGRT('10000000000');
const oneHundred = toGRT('100');
const oneMillion = toGRT('1000000');

describe('Subscriptions contract', () => {
  // Accounts
  let deployer: Account;
  let subscriber1: Account;
  let subscriber2: Account;
  let subscriberNoFunds: Account;

  // Contracts
  let subscriptions: Subscriptions;
  let stableToken: StableToken;

  // Constructor params
  const subscriptionsEpochBlocks = BigNumber.from(100);

  before(async function () {
    // eslint-disable-next-line @typescript-eslint/no-extra-semi
    [deployer, subscriber1, subscriber2, subscriberNoFunds] =
      await getAccounts();

    setAutoMine(true);
  });

  beforeEach(async function () {
    stableToken = await deployment.deployStableToken(
      [tenBillion],
      deployer.signer,
      false
    );
    subscriptions = await deployment.deploySubscriptions(
      [stableToken.address, subscriptionsEpochBlocks],
      deployer.signer,
      false
    );

    // Airdrop some stablecoins
    await stableToken
      .connect(deployer.signer)
      .transfer(subscriber1.address, oneMillion);
    await stableToken
      .connect(deployer.signer)
      .transfer(subscriber2.address, oneMillion);

    // Approve the subscription contract to transfer tokens from the user
    await stableToken
      .connect(subscriber1.signer)
      .approve(subscriptions.address, oneMillion);
    await stableToken
      .connect(subscriber2.signer)
      .approve(subscriptions.address, oneMillion);
  });

  describe('constructor', function () {
    it('should set the owner to the contract deployer address', async function () {
      expect(await subscriptions.owner()).to.eq(deployer.address);
    });

    it('should set the token address', async function () {
      expect(await subscriptions.token()).to.eq(stableToken.address);
    });

    it('should set the epoch block length', async function () {
      expect(await subscriptions.epochBlocks()).to.eq(subscriptionsEpochBlocks);
    });
  });

  describe('blockToEpoch', function () {
    it('should start the epoch index at 1', async function () {
      expect(await subscriptions.blockToEpoch(BigNumber.from(0))).to.eq(
        BigNumber.from(1)
      );
    });

    it('should start the nth epoch at (n-1)*epochBlocks', async function () {
      const n = Math.floor(Math.random() * 1000) + 1;
      const epochBlocks = await subscriptions.epochBlocks();
      const startBlock = epochBlocks.mul(n - 1);

      expect(await subscriptions.blockToEpoch(startBlock)).to.eq(n);
    });

    it('should end the nth epoch at n*epochBlocks', async function () {
      const n = Math.floor(Math.random() * 1000) + 1;
      const epochBlocks = await subscriptions.epochBlocks();
      const endBlock = epochBlocks.mul(n).sub(1);

      expect(await subscriptions.blockToEpoch(endBlock)).to.eq(n);
    });
  });

  describe('locked/unlocked', function () {
    it('should lock no tokens before subscription starts', async function () {
      const blockNumber = await latestBlockNumber();
      const subStart = blockNumber.add(100);
      const subEnd = blockNumber.add(200);
      const subRate = BigNumber.from(1);

      const locked = await subscriptions['locked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      expect(locked).to.eq(0);

      const unlocked = await subscriptions['unlocked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const unlockedExpected = subEnd.sub(subStart).mul(subRate);
      expect(unlocked).to.eq(unlockedExpected);
    });

    it('should lock no tokens at the subscription start boundary', async function () {
      const blockNumber = await latestBlockNumber();
      const subStart = blockNumber.add(1); // subscription tx on next block
      const subEnd = blockNumber.add(200);
      const subRate = BigNumber.from(1);

      const locked = await subscriptions['locked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      expect(locked).to.eq(0);

      const unlocked = await subscriptions['unlocked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const unlockedExpected = subEnd.sub(subStart).mul(subRate);
      expect(unlocked).to.eq(unlockedExpected);
    });

    it('should lock tokens progressively while subscription is active', async function () {
      const blockNumber = await latestBlockNumber();
      const subStart = blockNumber.sub(5);
      const subEnd = blockNumber.add(200);
      const subRate = BigNumber.from(1);

      const locked = await subscriptions['locked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const lockedExpected = blockNumber.sub(subStart).mul(subRate);
      expect(locked).to.eq(lockedExpected);

      const unlocked = await subscriptions['unlocked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const unlockedExpected = subEnd.sub(blockNumber).mul(subRate);
      expect(unlocked).to.eq(unlockedExpected);
    });

    it('should lock all tokens at the subscription end boundary', async function () {
      const blockNumber = await latestBlockNumber();
      const subStart = blockNumber.sub(5);
      const subEnd = blockNumber;
      const subRate = BigNumber.from(1);

      const locked = await subscriptions['locked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const lockedExpected = subEnd.sub(subStart).mul(subRate);
      expect(locked).to.eq(lockedExpected);

      const unlocked = await subscriptions['unlocked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      expect(unlocked).to.eq(0);
    });

    it('should lock all tokens after the subscription expired', async function () {
      const blockNumber = await latestBlockNumber();
      const subStart = blockNumber.sub(5);
      const subEnd = blockNumber.sub(3);
      const subRate = BigNumber.from(1);

      const locked = await subscriptions['locked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const lockedExpected = subEnd.sub(subStart).mul(subRate);
      expect(locked).to.eq(lockedExpected);

      const unlocked = await subscriptions['unlocked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      expect(unlocked).to.eq(0);
    });
  });

  describe('subscribe', function () {
    it('should revert if user is the zero address', async function () {
      const tx = subscriptions.subscribe(
        ethers.constants.AddressZero,
        BigNumber.from(0),
        BigNumber.from(0),
        BigNumber.from(1)
      );
      await expect(tx).revertedWith('user is null');
    });

    it('should revert if user is the contract address', async function () {
      const tx = subscriptions.subscribe(
        subscriptions.address,
        BigNumber.from(0),
        BigNumber.from(0),
        BigNumber.from(1)
      );
      await expect(tx).revertedWith('invalid user');
    });

    it('should revert if start >= end', async function () {
      const blockNumber = await latestBlockNumber();
      const tx = subscriptions.subscribe(
        subscriber1.address,
        blockNumber.add(100),
        blockNumber.add(50),
        BigNumber.from(1)
      );
      await expect(tx).revertedWith('start must be less than end');
    });

    it('should create a subscription for a user', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.sub(10);
      const end = blockNumber.add(510);
      const rate = BigNumber.from(5);

      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
    });

    it('should create a one epoch subscription for a user', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.sub(10);
      const end = blockNumber.add(5);
      const rate = BigNumber.from(5);

      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
    });

    it('should create a subscription for a user in the future', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.add(100);
      const end = blockNumber.add(500);
      const rate = BigNumber.from(5);

      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
    });

    it('should create a subscription for any user', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.add(100);
      const end = blockNumber.add(500);
      const rate = BigNumber.from(5);
      const user = subscriber2.address;

      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate,
        user
      );
    });

    it('should prevent creating a new sub if there is an active one', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.add(100);
      const end = blockNumber.add(500);
      const rate = BigNumber.from(5);
      const user = subscriber2.address;
      const newStart = blockNumber.add(200);
      const newEnd = blockNumber.add(600);

      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate,
        user
      );
      const tx = subscriptions
        .connect(subscriber1.signer)
        .subscribe(user, newStart, newEnd, rate);

      await expect(tx).revertedWith('active subscription must have ended');
    });

    it('should allow user bypassing the active sub restriction (grief protection)', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.add(100);
      const end = blockNumber.add(500);
      const rate = BigNumber.from(5);
      const newStart = blockNumber.add(200);
      const newEnd = blockNumber.add(600);

      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        newStart,
        newEnd,
        rate
      );
    });
  });

  describe('unsubscribe', function () {
    it('should allow user to cancel an active subscription', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.sub(5);
      const end = blockNumber.add(505);
      const rate = BigNumber.from(5);

      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      mineNBlocks(100);

      await unsubscribe(
        stableToken,
        subscriptions,
        subscriber1,
        subscribeBlockNumber
      );
    });

    it('should allow user to cancel an active one epoch subscription', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.sub(5);
      const end = blockNumber.add(5);
      const rate = BigNumber.from(5);

      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      await unsubscribe(
        stableToken,
        subscriptions,
        subscriber1,
        subscribeBlockNumber
      );
    });

    it('should allow user to cancel an upcoming subscription', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.add(50);
      const end = blockNumber.add(505);
      const rate = BigNumber.from(5);
      const startEpoch = await subscriptions.blockToEpoch(start);
      const endEpoch = await subscriptions.blockToEpoch(end);

      // Before state
      const beforeStartEpoch = await subscriptions._epochs(startEpoch);
      const beforeEndEpoch = await subscriptions._epochs(endEpoch);

      // Subscribe and unsubscribe
      const subBlock = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      await unsubscribe(stableToken, subscriptions, subscriber1, subBlock);

      // After state
      const afterStartEpoch = await subscriptions._epochs(startEpoch);
      const afterEndEpoch = await subscriptions._epochs(endEpoch);

      expect(beforeStartEpoch.delta).to.equal(afterStartEpoch.delta);
      expect(beforeStartEpoch.extra).to.equal(afterStartEpoch.extra);
      expect(beforeEndEpoch.delta).to.equal(afterEndEpoch.delta);
      expect(beforeEndEpoch.extra).to.equal(afterEndEpoch.extra);
    });

    it('should revert when canceling an expired subscription', async function () {
      const blockNumber = await latestBlockNumber();
      const start = blockNumber.sub(5);
      const end = blockNumber.add(505);
      const rate = BigNumber.from(5);

      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      mineNBlocks(1000);

      const tx = unsubscribe(
        stableToken,
        subscriptions,
        subscriber1,
        subscribeBlockNumber
      );
      await expect(tx).revertedWith('Subscription has expired');
    });

    it('should revert if user has no subscription', async function () {
      const tx = subscriptions.connect(subscriber2.signer).unsubscribe();
      await expect(tx).revertedWith('no active subscription');
    });
  });
});

async function subscribe(
  stableToken: StableToken,
  subscriptions: Subscriptions,
  signer: Account,
  start: BigNumber,
  end: BigNumber,
  rate: BigNumber,
  user?: string
) {
  user = user ?? signer.address;

  const amount = rate.mul(end.sub(start));

  // Before state
  const beforeBlock = await latestBlockNumber();
  const beforeBalance = await stableToken.balanceOf(signer.address);
  const beforeContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );

  // * Tx
  const tx = subscriptions
    .connect(signer.signer)
    .subscribe(user, start, end, rate);

  // If start is in the past, override it with the next block where the sub tx will be mined
  const nextBlock = await nextBlockNumber();
  start = start.gte(nextBlock) ? start : nextBlock;

  // * Check events
  await expect(tx)
    .to.emit(subscriptions, 'Subscribe')
    .withArgs(user, start, end, rate);

  // * Check balances
  const afterBalance = await stableToken.balanceOf(signer.address);
  const afterContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );

  // Actual amount deposited might be less than intended if subStart < block.number
  const amountDeposited = beforeBalance.sub(afterBalance);
  expect(amountDeposited).to.lte(amount);
  expect(afterContractBalance).to.eq(
    beforeContractBalance.add(amountDeposited)
  );

  // * Check state
  const sub = await subscriptions._subscriptions(user);
  expect(sub.start).to.eq(start);
  expect(sub.end).to.eq(end);
  expect(sub.rate).to.eq(rate);

  const afterBlock = await latestBlockNumber();

  await testEpochDetails(
    subscriptions,
    start,
    end,
    rate,
    beforeBlock,
    afterBlock
  );

  return (await tx).blockNumber;
}

async function unsubscribe(
  stableToken: StableToken,
  subscriptions: Subscriptions,
  signer: Account,
  subscribeBlockNumber: number | undefined
) {
  const user = signer.address;

  // Before state
  const beforeSub = await subscriptions._subscriptions(user);
  const beforeBlock = await latestBlockNumber();

  const amountUnlocked = maxBN(
    BigNumber.from(0),
    beforeSub.end.sub(maxBN(await nextBlockNumber(), beforeSub.start))
  ).mul(beforeSub.rate); // Amount unlocked is the amount that will be freed up with the tx in the next block

  const beforeBalance = await stableToken.balanceOf(user);
  const beforeContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );

  // * Tx
  const tx = subscriptions.connect(signer.signer).unsubscribe();

  // * Check events
  await expect(tx).to.emit(subscriptions, 'Unsubscribe').withArgs(user);

  // * Check balances
  const afterBalance = await stableToken.balanceOf(user);
  const afterContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );
  expect(afterBalance).to.eq(beforeBalance.add(amountUnlocked));
  expect(afterContractBalance).to.eq(beforeContractBalance.sub(amountUnlocked));

  // * Check state
  const afterSub = await subscriptions._subscriptions(user);
  const txBlock = (await tx).blockNumber!;

  // Sub gets deleted if it's canceled before starting
  if (txBlock < beforeSub.start.toNumber()) {
    expect(afterSub.start).to.eq(0);
    expect(afterSub.rate).to.eq(0);
    expect(afterSub.end).to.eq(0);

    await testEpochDetails(
      subscriptions,
      beforeSub.start,
      beforeSub.end,
      beforeSub.rate.mul(-1),
      beforeBlock,
      BigNumber.from(txBlock)
    );
  } else {
    // Otherwise the sub was active, the  end is set to the block where the tx cancelled it
    // Note that it's not possible to have a txBlock > sub.end because the tx will revert
    expect(afterSub.start).to.eq(beforeSub.start);
    expect(afterSub.rate).to.eq(beforeSub.rate);
    expect(afterSub.end).to.eq(txBlock);

    // Sub + cancel -> Epoch changes should match those of a sub [start, current)
    await testEpochDetails(
      subscriptions,
      beforeSub.start,
      BigNumber.from(txBlock),
      beforeSub.rate,
      BigNumber.from(subscribeBlockNumber! - 1),
      BigNumber.from(txBlock)
    );
  }
}

async function testEpochDetails(
  subscriptions: Subscriptions,
  start: BigNumber,
  end: BigNumber,
  rate: BigNumber,
  beforeBlock: BigNumber,
  afterBlock: BigNumber
) {
  await testStartEpochDetails(
    subscriptions,
    start,
    end,
    rate,
    beforeBlock,
    afterBlock
  );
  await testEndEpochDetails(
    subscriptions,
    start,
    end,
    rate,
    beforeBlock,
    afterBlock
  );
}

async function testStartEpochDetails(
  subscriptions: Subscriptions,
  start: BigNumber,
  end: BigNumber,
  rate: BigNumber,
  beforeBlock: BigNumber,
  afterBlock: BigNumber
) {
  const epochStart = await subscriptions.blockToEpoch(start);
  const epochEnd = await subscriptions.blockToEpoch(end);
  const epochBlocks = await subscriptions.epochBlocks();

  // Before state
  const beforeEpoch = await subscriptions._epochs(epochStart, {
    blockTag: beforeBlock.toNumber(),
  });

  // After state
  const afterEpoch = await subscriptions._epochs(epochStart, {
    blockTag: afterBlock.toNumber(),
  });

  // Check deltas
  if (!epochStart.eq(epochEnd)) {
    expect(afterEpoch.delta.sub(beforeEpoch.delta)).to.eq(
      epochBlocks.mul(rate)
    );
    expect(beforeEpoch.extra.sub(afterEpoch.extra)).to.eq(
      start.sub(epochBlocks.mul(epochStart.sub(1))).mul(rate)
    );
  } else {
    expect(afterEpoch.delta.sub(beforeEpoch.delta)).to.eq(BigNumber.from(0));
    expect(afterEpoch.extra.sub(beforeEpoch.extra)).to.eq(
      end.sub(start).mul(rate)
    );
  }
}

async function testEndEpochDetails(
  subscriptions: Subscriptions,
  start: BigNumber,
  end: BigNumber,
  rate: BigNumber,
  beforeBlock: BigNumber,
  afterBlock: BigNumber
) {
  const epochStart = await subscriptions.blockToEpoch(start);
  const epochEnd = await subscriptions.blockToEpoch(end);
  const epochBlocks = await subscriptions.epochBlocks();

  // Before state
  const beforeEpoch = await subscriptions._epochs(epochEnd, {
    blockTag: beforeBlock.toNumber(),
  });

  // After state
  const afterEpoch = await subscriptions._epochs(epochEnd, {
    blockTag: afterBlock.toNumber(),
  });

  // Check deltas
  if (!epochStart.eq(epochEnd)) {
    expect(beforeEpoch.delta.sub(afterEpoch.delta)).to.eq(
      epochBlocks.mul(rate)
    );
    expect(afterEpoch.extra.sub(beforeEpoch.extra)).to.eq(
      end.sub(epochBlocks.mul(epochEnd.sub(1))).mul(rate)
    );
  } else {
    expect(afterEpoch.delta.sub(beforeEpoch.delta)).to.eq(BigNumber.from(0));
    expect(afterEpoch.extra.sub(beforeEpoch.extra)).to.eq(
      end.sub(start).mul(rate)
    );
  }
}
