import '@nomicfoundation/hardhat-chai-matchers';
import {time} from '@nomicfoundation/hardhat-network-helpers';

import {expect} from 'chai';
import * as deployment from '../utils/deploy';
import {getAccounts, Account, toGRT} from '../utils/helpers';

import {Subscriptions} from '../types/contracts/Subscriptions';
import {StableToken} from '../types/contracts/test/StableMock.sol/StableToken';
import {BigNumber, ethers} from 'ethers';
import {
  latestBlockTimestamp,
  latestBlockNumber,
  maxBN,
  mineNBlocks,
  setAutoMine,
} from './helpers';

const tenBillion = toGRT('10000000000');
const oneMillion = toGRT('1000000');
const zero = toGRT('0');

describe('Subscriptions contract', () => {
  // Accounts
  let deployer: Account;
  let subscriber1: Account;
  let subscriber2: Account;
  let subscriberNoFunds: Account;
  let recurringPayments: Account;
  let newRecurringPayments: Account;

  // Contracts
  let subscriptions: Subscriptions;
  let stableToken: StableToken;

  // Constructor params
  const subscriptionsEpochSeconds = BigNumber.from(100);

  before(async function () {
    // eslint-disable-next-line @typescript-eslint/no-extra-semi
    [
      deployer,
      subscriber1,
      subscriber2,
      subscriberNoFunds,
      recurringPayments,
      newRecurringPayments,
    ] = await getAccounts();

    setAutoMine(true);
  });

  beforeEach(async function () {
    stableToken = await deployment.deployStableToken(
      [tenBillion],
      deployer.signer,
      false
    );
    subscriptions = await deployment.deploySubscriptions(
      [
        stableToken.address,
        subscriptionsEpochSeconds,
        recurringPayments.address,
      ],
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
    await stableToken
      .connect(deployer.signer)
      .transfer(recurringPayments.address, oneMillion);

    // Approve the subscription contract to transfer tokens from the user
    await stableToken
      .connect(subscriber1.signer)
      .approve(subscriptions.address, oneMillion);
    await stableToken
      .connect(subscriber2.signer)
      .approve(subscriptions.address, oneMillion);
    await stableToken
      .connect(recurringPayments.signer)
      .approve(subscriptions.address, oneMillion);
  });

  describe('constructor', function () {
    it('should set the owner to the contract deployer address', async function () {
      expect(await subscriptions.owner()).to.eq(deployer.address);
    });

    it('should set the token address', async function () {
      expect(await subscriptions.token()).to.eq(stableToken.address);
    });

    it('should set the epoch z', async function () {
      expect(await subscriptions.epochSeconds()).to.eq(
        subscriptionsEpochSeconds
      );
    });

    it('should set the recurring payments address', async function () {
      expect(await subscriptions.recurringPayments()).to.eq(
        recurringPayments.address
      );
    });
  });

  describe('setters', function () {
    it('should set the recurring payments address', async function () {
      const tx = subscriptions.setRecurringPayments(
        newRecurringPayments.address
      );

      await expect(tx)
        .to.emit(subscriptions, 'RecurringPaymentsUpdated')
        .withArgs(newRecurringPayments.address);
      expect(await subscriptions.recurringPayments()).to.eq(
        newRecurringPayments.address
      );
    });

    it('should prevent unauthorized users from changing the recurring payments address', async function () {
      const tx = subscriptions
        .connect(subscriber1.signer)
        .setRecurringPayments(newRecurringPayments.address);
      await expect(tx).revertedWith('Ownable: caller is not the owner');
    });

    it('should prevent setting the recurring payments address to zero address', async function () {
      const tx = subscriptions.setRecurringPayments(
        ethers.constants.AddressZero
      );
      await expect(tx).revertedWith('recurringPayments cannot be zero address');
    });
  });

  describe('transferOwnership', function () {
    it('should set the owner to the new owner', async function () {
      await subscriptions.transferOwnership(subscriber1.address);
      expect(await subscriptions.owner()).to.eq(subscriber1.address);
    });
  });

  describe('authorizedSigners', function () {
    it('user is always authorized', async function () {
      const user = subscriber1.address;
      expect(await subscriptions.checkAuthorizedSigner(user, user)).to.eq(true);
      await expect(
        subscriptions.connect(subscriber1.signer).addAuthorizedSigner(user)
      ).revertedWith('user is always an authorized signer');
      await expect(
        subscriptions.connect(subscriber1.signer).removeAuthorizedSigner(user)
      ).revertedWith('user is always an authorized signer');
    });

    it('other addresses are unauthorized by default', async function () {
      const user = subscriber1.address;
      const other = subscriber2.address;
      expect(await subscriptions.checkAuthorizedSigner(user, other)).to.eq(
        false
      );
    });

    it('authorizedSigners can be added', async function () {
      const user = subscriber1.address;
      const other = subscriber2.address;
      const tx = await subscriptions
        .connect(subscriber1.signer)
        .addAuthorizedSigner(other);
      expect(await subscriptions.checkAuthorizedSigner(user, other)).to.eq(
        true
      );
      expect(tx)
        .to.emit(subscriptions, 'AuthorizedSignerAdded')
        .withArgs(user, other);
    });

    it('authorizedSigners can be removed', async function () {
      const user = subscriber1.address;
      const other = subscriber2.address;
      await subscriptions
        .connect(subscriber1.signer)
        .addAuthorizedSigner(other);
      const tx = await subscriptions
        .connect(subscriber1.signer)
        .removeAuthorizedSigner(other);
      expect(await subscriptions.checkAuthorizedSigner(user, other)).to.eq(
        false
      );
      expect(tx)
        .to.emit(subscriptions, 'AuthorizedSignerRemoved')
        .withArgs(user, other);
    });
  });

  describe('timestampToEpoch', function () {
    it('should start the epoch index at 1', async function () {
      expect(await subscriptions.timestampToEpoch(BigNumber.from(0))).to.eq(
        BigNumber.from(1)
      );
    });

    it('should start the nth epoch at (n-1)*epochSeconds', async function () {
      const n = Math.floor(Math.random() * 1000) + 1;
      const epochSeconds = await subscriptions.epochSeconds();
      const startBlock = epochSeconds.mul(n - 1);

      expect(await subscriptions.timestampToEpoch(startBlock)).to.eq(n);
    });

    it('should end the nth epoch at n*epochSeconds', async function () {
      const n = Math.floor(Math.random() * 1000) + 1;
      const epochSeconds = await subscriptions.epochSeconds();
      const endBlock = epochSeconds.mul(n).sub(1);

      expect(await subscriptions.timestampToEpoch(endBlock)).to.eq(n);
    });
  });

  describe('locked/unlocked', function () {
    it('should lock no tokens before subscription starts', async function () {
      const now = await latestBlockTimestamp();
      const subStart = now.add(100);
      const subEnd = now.add(200);
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
      const now = await latestBlockTimestamp();
      const subStart = now.add(1); // subscription tx on next block
      const subEnd = now.add(200);
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
      const now = await latestBlockTimestamp();
      const subStart = now.sub(5);
      const subEnd = now.add(200);
      const subRate = BigNumber.from(1);

      const locked = await subscriptions['locked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const lockedExpected = now.sub(subStart).mul(subRate);
      expect(locked).to.eq(lockedExpected);

      const unlocked = await subscriptions['unlocked(uint64,uint64,uint128)'](
        subStart,
        subEnd,
        subRate
      );
      const unlockedExpected = subEnd.sub(now).mul(subRate);
      expect(unlocked).to.eq(unlockedExpected);
    });

    it('should lock all tokens at the subscription end boundary', async function () {
      const now = await latestBlockTimestamp();
      const subStart = now.sub(5);
      const subEnd = now;
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
      const now = await latestBlockTimestamp();
      const subStart = now.sub(5);
      const subEnd = now.sub(3);
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
    it('should revert if start >= end', async function () {
      const now = await latestBlockTimestamp();
      const tx = subscriptions
        .connect(subscriber1.signer)
        .subscribe(now.add(100), now.add(50), BigNumber.from(1));
      await expect(tx).revertedWith('start must be less than end');
    });
    it('should create a subscription for a user', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(10);
      const end = now.add(510);
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
      const now = await latestBlockTimestamp();
      const start = now.sub(10);
      const end = now.add(5);
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
      const now = await latestBlockTimestamp();
      const start = now.add(100);
      const end = now.add(500);
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
    it('should allow user to modify the active sub', async function () {
      const now = await latestBlockTimestamp();
      const start = now.add(100);
      const end = now.add(500);
      const rate = BigNumber.from(5);
      const firstSubValue = end.sub(start).mul(rate);
      const newStart = now.add(200);
      const newEnd = now.add(400);
      const newSubValue = newEnd.sub(newStart).mul(rate);
      const initialBalance = await stableToken.balanceOf(subscriber1.address);
      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
      expect(await stableToken.balanceOf(subscriber1.address)).eq(
        initialBalance.sub(firstSubValue)
      );
      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        newStart,
        newEnd,
        rate
      );
      expect(await stableToken.balanceOf(subscriber1.address)).eq(
        initialBalance.sub(newSubValue)
      );
    });
    it('0xMacro: should allow creating a new sub if there is an expired one', async function () {
      const now = await latestBlockTimestamp();
      const start = now.add(100);
      const end = now.add(500);
      const rate = BigNumber.from(5);
      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      // advance past subscription end
      mineNBlocks(600);
      const newNow = await latestBlockTimestamp();
      expect(newNow).to.be.gte(end);

      const newStart = newNow.add(100);
      const newEnd = newNow.add(500);

      // Should be able to create another subscription
      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        newStart,
        newEnd,
        rate
      );
    });
    it('0xMacro: should reject subscriptions where funds can be locked due to cast truncation', async () => {
      const MaxInt64 = BigNumber.from('9223372036854775807');

      // Scenario: user/UI mistakenly encodes the wrong timestamp values.
      const start = MaxInt64.add(1);
      const end = start.add(100);
      const rate = BigNumber.from(5);

      await expect(
        subscriptions.connect(subscriber1.signer).subscribe(start, end, rate)
      ).revertedWith('end too large');
    });
  });

  describe('unsubscribe', function () {
    it('should allow user to cancel an active subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(5);
      const end = now.add(505);
      const rate = BigNumber.from(5);
      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
      await mineNBlocks(100);
      await unsubscribe(
        stableToken,
        subscriptions,
        subscriber1,
        subscribeBlockNumber
      );
    });
    it('should allow user to cancel an active one epoch subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(5);
      const end = now.add(5);
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
      const now = await latestBlockTimestamp();
      const start = now.add(50);
      const end = now.add(505);
      const rate = BigNumber.from(5);
      const startEpoch = await subscriptions.timestampToEpoch(start);
      const endEpoch = await subscriptions.timestampToEpoch(end);
      // Before state
      const beforeStartEpoch = await subscriptions.epochs(startEpoch);
      const beforeEndEpoch = await subscriptions.epochs(endEpoch);
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
      const afterStartEpoch = await subscriptions.epochs(startEpoch);
      const afterEndEpoch = await subscriptions.epochs(endEpoch);
      expect(beforeStartEpoch.delta).to.equal(afterStartEpoch.delta);
      expect(beforeStartEpoch.extra).to.equal(afterStartEpoch.extra);
      expect(beforeEndEpoch.delta).to.equal(afterEndEpoch.delta);
      expect(beforeEndEpoch.extra).to.equal(afterEndEpoch.extra);
    });
    it('should revert when canceling an expired subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(5);
      const end = now.add(505);
      const rate = BigNumber.from(5);
      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
      await mineNBlocks(1000);
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

  describe('create', function () {
    it('should create a subscription for a user', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(10);
      const end = now.add(510);
      const rate = BigNumber.from(5);
      const data = ethers.utils.defaultAbiCoder.encode(
        ['uint64', 'uint64', 'uint128'],
        [start, end, rate]
      );

      await create(
        stableToken,
        subscriptions,
        recurringPayments,
        subscriber1.address,
        data
      );
    });

    it('should prevent unauthorized users to call create', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(10);
      const end = now.add(510);
      const rate = BigNumber.from(5);
      const data = ethers.utils.defaultAbiCoder.encode(
        ['uint64', 'uint64', 'uint128'],
        [start, end, rate]
      );

      const tx = subscriptions
        .connect(subscriber1.signer)
        .create(subscriber1.address, data);
      await expect(tx).revertedWith(
        'caller is not the recurring payments contract'
      );
    });
  });

  describe('addTo', function () {});

  describe('extend', function () {
    it('should revert if the amount to extend is zero', async function () {
      const tx = subscriptions.addTo(
        ethers.constants.AddressZero,
        BigNumber.from(0)
      );
      await expect(tx).revertedWith('amount must be positive');
    });

    it('should revert if user is the zero address', async function () {
      const tx = subscriptions.addTo(
        ethers.constants.AddressZero,
        BigNumber.from(1000)
      );
      await expect(tx).revertedWith('user is null');
    });

    it('should revert when extending a subscription that does not exist', async function () {
      const tx = subscriptions.addTo(subscriber2.address, BigNumber.from(10));
      await expect(tx).revertedWith('no subscription found');
    });

    it('should revert when the new end time is in the past', async function () {
      const now = await latestBlockTimestamp();
      const start = now.add(500);
      const end = now.add(1000);
      const rate = BigNumber.from(5);
      const amountToExtend = BigNumber.from(2000); // newEnd: end + 2000/5 = 1000 + 400 = 1400

      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      // mine past the newEnd
      await mineNBlocks(1500);

      const tx = addToSubscription(
        stableToken,
        subscriptions,
        recurringPayments,
        subscriber1.address,
        amountToExtend,
        subscribeBlockNumber
      );
      await expect(tx).revertedWith('new end cannot be in the past');
    });
   
    it('should allow extending an active subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now;
      const end = now.add(1000);
      const rate = BigNumber.from(5);
      const amountToExtend = BigNumber.from(2000); // newEnd: end + 2000/5 = 1000 + 400 = 1400      const user = subscriber2.address;
      
      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      // mine past the start of the subscription
      await mineNBlocks(150);

      await addToSubscription(
        stableToken,
        subscriptions,
        recurringPayments,
        subscriber1.address,
        amountToExtend,
        subscribeBlockNumber
      );
    });

    it('should allow extending an expired subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now;
      const end = now.add(1000);
      const rate = BigNumber.from(5);
      const amountToExtend = BigNumber.from(2000); // newEnd: end + 2000/5 = 1000 + 400 = 1400      const user = subscriber2.address;
      
      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      // mine past the end of the subscription, but not past the new end
      await mineNBlocks(1100);

      await addToSubscription(
        stableToken,
        subscriptions,
        recurringPayments,
        subscriber1.address,
        amountToExtend,
        subscribeBlockNumber
      );
    });

    it('should allow extending a one epoch subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now;
      const end = now.add(5);
      const rate = BigNumber.from(5);
      const amountToExtend = BigNumber.from(2000);

      const subscribeBlockNumber = await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
      await addToSubscription(
        stableToken,
        subscriptions,
        recurringPayments,
        subscriber1.address,
        amountToExtend,
        subscribeBlockNumber
      );
    });
  });

  describe.skip('collect', function () {});

  describe('setPendingSubscription', function () {
    it('should set a pending subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now.add(1000);
      const end = now.add(2000);
      const rate = BigNumber.from(1);
      await setPendingSubscription(
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );
    });

    it('should override existing pending subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now.add(1000);
      const end = now.add(2000);
      const rate = BigNumber.from(1);
      await setPendingSubscription(
        subscriptions,
        subscriber1,
        start,
        end,
        rate
      );

      const newStart = now.add(10000);
      const newEnd = now.add(20000);
      const newRate = BigNumber.from(10);
      await setPendingSubscription(
        subscriptions,
        subscriber1,
        newStart,
        newEnd,
        newRate
      );

      const sub = await subscriptions.pendingSubscriptions(subscriber1.address);
      expect(sub.start).to.equal(newStart);
      expect(sub.end).to.equal(newEnd);
      expect(sub.rate).to.equal(newRate);
    });
  });

  describe('fulfil', function () {
    it('should subscribe using pending subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now.add(1000);
      const end = now.add(2000);
      const rate = BigNumber.from(1);
      const value = end.sub(start).mul(rate);

      await subscriptions
        .connect(subscriber1.signer)
        .setPendingSubscription(start, end, rate);

      await stableToken
        .connect(subscriber2.signer)
        .approve(subscriptions.address, value);

      await fulfil(
        stableToken,
        subscriptions,
        subscriber2,
        start,
        end,
        rate,
        subscriber1.address
      );
    });

    it('should subscribe using pending subscription and send leftover tokens back to user', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(100);
      const end = now.add(2000);
      const rate = BigNumber.from(1);
      const value = end.sub(start).mul(rate);

      await subscriptions
        .connect(subscriber1.signer)
        .setPendingSubscription(start, end, rate);

      await stableToken
        .connect(subscriber2.signer)
        .approve(subscriptions.address, value);

      await fulfil(
        stableToken,
        subscriptions,
        subscriber2,
        start,
        end,
        rate,
        subscriber1.address
      );
    });

    it('should revert if there is no pending subscription', async function () {
      const tx = subscriptions
        .connect(subscriber1.signer)
        .fulfil(subscriber2.address, oneMillion);
      await expect(tx).revertedWith('No pending subscription');
    });

    it('should revert if pending subscription has expired', async function () {
      const now = await latestBlockTimestamp();
      const start = now.sub(100);
      const end = now.add(200);
      const rate = BigNumber.from(1);
      const value = end.sub(start).mul(rate);

      await subscriptions
        .connect(subscriber1.signer)
        .setPendingSubscription(start, end, rate);

      await stableToken
        .connect(subscriber2.signer)
        .approve(subscriptions.address, value);

      await mineNBlocks(end.sub(now).toNumber());

      const tx = subscriptions
        .connect(subscriber2.signer)
        .fulfil(subscriber1.address, value);
      await expect(tx).revertedWith('Pending subscription has expired');
    });

    it('should revert if fulfil funds are not enough to create the pending subscription', async function () {
      const now = await latestBlockTimestamp();
      const start = now.add(1000);
      const end = now.add(2000);
      const rate = BigNumber.from(1);
      const value = end.sub(start).mul(rate);

      await subscriptions
        .connect(subscriber1.signer)
        .setPendingSubscription(start, end, rate);

      await stableToken
        .connect(subscriber2.signer)
        .approve(subscriptions.address, value);

      const tx = subscriptions
        .connect(subscriber2.signer)
        .fulfil(subscriber1.address, zero);
      await expect(tx).revertedWith(
        'Insufficient funds to create subscription'
      );
    });

    it('should send extra to user when given more tokens than required for subscription', async function () {
      const startOffset = 100;
      mineNBlocks(startOffset);
      const now = await latestBlockTimestamp();
      const start = now.sub(startOffset);
      const end = now.add(2000);
      const rate = BigNumber.from(1);
      const value = end.sub(start).mul(rate);

      await subscriptions
        .connect(subscriber1.signer)
        .setPendingSubscription(start, end, rate);

      await stableToken
        .connect(subscriber2.signer)
        .approve(subscriptions.address, value);

      const beforeBalance = await stableToken.balanceOf(subscriber1.address);

      await subscriptions
        .connect(subscriber2.signer)
        .fulfil(subscriber1.address, value);

      const extra = (await latestBlockTimestamp()).sub(start).mul(rate);
      const afterBalance = await stableToken.balanceOf(subscriber1.address);
      expect(afterBalance).eq(beforeBalance.add(extra));
    });

    it('0xMacro: may unsubscribe the caller', async () => {
      let now = await latestBlockTimestamp();

      // subscriber1 creates subscription and sets pending subscription
      const sub1Start = (await latestBlockTimestamp()).add(5);
      const sub1End = sub1Start.add(500);
      const sub1Rate = BigNumber.from(5);
      const sub1Value = sub1End.sub(sub1Start).mul(sub1Rate);

      await subscribe(
        stableToken,
        subscriptions,
        subscriber1,
        sub1Start,
        sub1End,
        sub1Rate
      );

      await subscriptions
        .connect(subscriber1.signer)
        .setPendingSubscription(sub1End, sub1End.add(500), sub1Rate);

      // subscr2 creates subscription and prematurely calls fulfil() for subscr1
      now = await latestBlockTimestamp();
      const sub2Start = now.add(5);
      const sub2End = sub2Start.add(500);
      const sub2Rate = BigNumber.from(1);
      const sub2Value = sub2End.sub(sub2Start).mul(sub2Rate);

      await subscribe(
        stableToken,
        subscriptions,
        subscriber2,
        sub2Start,
        sub2End,
        sub2Rate
      );

      await stableToken
        .connect(subscriber2.signer)
        .approve(subscriptions.address, sub1Value);

      await subscriptions
        .connect(subscriber2.signer)
        .fulfil(subscriber1.address, sub1Value);

      // Expect subscriber 2 should still have a subscription registered
      const sub = await subscriptions.subscriptions(subscriber2.address);
      expect(sub.end).to.eq(sub2End);
    });
  });
});

async function subscribe(
  stableToken: StableToken,
  subscriptions: Subscriptions,
  user: Account,
  start: BigNumber,
  end: BigNumber,
  rate: BigNumber
) {
  const amount = rate.mul(end.sub(start));
  const epochSeconds = await subscriptions.epochSeconds();

  // Before state
  const beforeBlock = await latestBlockNumber();
  const beforeBalance = await stableToken.balanceOf(user.address);
  const beforeContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );

  // * Tx
  const tx = subscriptions.connect(user.signer).subscribe(start, end, rate);
  await tx;
  const txTimestamp = await time.latest();
  const txEpoch = BigNumber.from(txTimestamp).div(epochSeconds).add(1);

  // If start is in the past, override it with the tx timestamp
  start = start.gte(txTimestamp) ? start : BigNumber.from(txTimestamp);

  // * Check events
  await expect(tx)
    .to.emit(subscriptions, 'Subscribe')
    .withArgs(user.address, txEpoch, start, end, rate);

  // * Check balances
  const afterBalance = await stableToken.balanceOf(user.address);
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
  const sub = await subscriptions.subscriptions(user.address);
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

  return (await tx).blockNumber!;
}

async function unsubscribe(
  stableToken: StableToken,
  subscriptions: Subscriptions,
  signer: Account,
  subscribeBlockNumber: number | undefined
) {
  const user = signer.address;

  // Before state
  const beforeSub = await subscriptions.subscriptions(user);
  const beforeBlock = await latestBlockNumber();
  const beforeTimestamp = await latestBlockTimestamp();

  const amountUnlocked = maxBN(
    BigNumber.from(0),
    beforeSub.end.sub(maxBN(beforeTimestamp.add(1), beforeSub.start))
  ).mul(beforeSub.rate); // Amount unlocked is the amount that will be freed up with the tx in the next block

  const beforeBalance = await stableToken.balanceOf(user);
  const beforeContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );

  // * Tx
  const tx = await subscriptions.connect(signer.signer).unsubscribe();
  const txBlock = tx.blockNumber!;
  const txTimestamp = await latestBlockTimestamp();
  const txEpoch = await subscriptions.timestampToEpoch(txTimestamp);

  // * Check events
  await expect(tx)
    .to.emit(subscriptions, 'Unsubscribe')
    .withArgs(user, txEpoch);

  // * Check balances
  const afterBalance = await stableToken.balanceOf(user);
  const afterContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );
  expect(afterBalance).to.eq(beforeBalance.add(amountUnlocked));
  expect(afterContractBalance).to.eq(beforeContractBalance.sub(amountUnlocked));

  // * Check state
  const afterSub = await subscriptions.subscriptions(user);

  // Sub gets deleted if it's canceled before starting
  if (txTimestamp.toNumber() < beforeSub.start.toNumber()) {
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
    expect(afterSub.end).to.eq(txTimestamp);

    // Sub + cancel -> Epoch changes should match those of a sub [start, current)
    await testEpochDetails(
      subscriptions,
      beforeSub.start,
      txTimestamp,
      beforeSub.rate,
      BigNumber.from(subscribeBlockNumber! - 1),
      BigNumber.from(txBlock)
    );
  }
}

async function create(
  stableToken: StableToken,
  subscriptions: Subscriptions,
  signer: Account,
  userAddress: string,
  data: string
) {
  const decoded = ethers.utils.defaultAbiCoder.decode(
    ['uint64', 'uint64', 'uint128'],
    data
  );
  const start = decoded[0];
  const end = decoded[1];
  const rate = decoded[2];

  const amount = rate.mul(end.sub(start));
  const epochSeconds = await subscriptions.epochSeconds();

  // Before state
  const beforeBlock = await latestBlockNumber();
  const beforeBalance = await stableToken.balanceOf(signer.address);
  const beforeContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );

  // * Tx
  const tx = subscriptions.connect(signer.signer).create(userAddress, data);
  await tx;
  const txTimestamp = await time.latest();
  const txEpoch = BigNumber.from(txTimestamp).div(epochSeconds).add(1);

  // * Check events
  await expect(tx)
    .to.emit(subscriptions, 'Subscribe')
    .withArgs(userAddress, txEpoch, start, end, rate);

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
  const sub = await subscriptions.subscriptions(userAddress);
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

  return (await tx).blockNumber!;
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
  const epochStart = await subscriptions.timestampToEpoch(start);
  const epochEnd = await subscriptions.timestampToEpoch(end);
  const epochBlocks = await subscriptions.epochSeconds();

  // Before state
  const beforeEpoch = await subscriptions.epochs(epochStart, {
    blockTag: beforeBlock.toNumber(),
  });

  // After state
  const afterEpoch = await subscriptions.epochs(epochStart, {
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
  const epochStart = await subscriptions.timestampToEpoch(start);
  const epochEnd = await subscriptions.timestampToEpoch(end);
  const epochBlocks = await subscriptions.epochSeconds();

  // Before state
  const beforeEpoch = await subscriptions.epochs(epochEnd, {
    blockTag: beforeBlock.toNumber(),
  });

  // After state
  const afterEpoch = await subscriptions.epochs(epochEnd, {
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

async function setPendingSubscription(
  subscriptions: Subscriptions,
  signer: Account,
  start: BigNumber,
  end: BigNumber,
  rate: BigNumber
) {
  // Set pending subscription
  const tx = subscriptions
    .connect(signer.signer)
    .setPendingSubscription(start, end, rate);

  await tx;

  const txTimestamp = await time.latest();
  const epochSeconds = await subscriptions.epochSeconds();
  const txEpoch = BigNumber.from(txTimestamp).div(epochSeconds).add(1);

  await expect(tx)
    .to.emit(subscriptions, 'PendingSubscriptionCreated')
    .withArgs(signer.address, txEpoch, start, end, rate);

  const sub = await subscriptions.pendingSubscriptions(signer.address);
  expect(sub.start).to.equal(start);
  expect(sub.end).to.equal(end);
  expect(sub.rate).to.equal(rate);
}

async function fulfil(
  stableToken: StableToken,
  subscriptions: Subscriptions,
  signer: Account,
  start: BigNumber,
  end: BigNumber,
  rate: BigNumber,
  user: string
) {
  const amount = rate.mul(end.sub(start));
  const epochSeconds = await subscriptions.epochSeconds();

  // Before state
  const beforeBlock = await latestBlockNumber();
  const beforeBalance = await stableToken.balanceOf(signer.address);
  const beforeContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );
  const beforeUserBalance = await stableToken.balanceOf(user);

  // * Tx
  const tx = subscriptions.connect(signer.signer).fulfil(user, amount);
  await tx;
  const txTimestamp = await time.latest();
  const txEpoch = BigNumber.from(txTimestamp).div(epochSeconds).add(1);

  // If start is in the past, override it with the tx timestamp
  start = start.gte(txTimestamp) ? start : BigNumber.from(txTimestamp);

  // * Check events
  await expect(tx)
    .to.emit(subscriptions, 'Subscribe')
    .withArgs(user, txEpoch, start, end, rate);

  // * Check balances
  const afterBalance = await stableToken.balanceOf(signer.address);
  const afterContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );
  const afterUserBalance = await stableToken.balanceOf(user);

  // Actual amount deposited might be less than intended if subStart < block.number
  const amountDeposited = rate.mul(end.sub(start));
  const amountLeftover = amount.sub(amountDeposited);

  expect(afterContractBalance).to.eq(
    beforeContractBalance.add(amountDeposited)
  );
  expect(amountDeposited).to.lte(amount);
  expect(afterBalance).to.eq(beforeBalance.sub(amount));
  expect(afterUserBalance).to.eq(beforeUserBalance.add(amountLeftover));

  // * Check state
  const sub = await subscriptions.subscriptions(user);
  expect(sub.start).to.eq(start);
  expect(sub.end).to.eq(end);
  expect(sub.rate).to.eq(rate);

  const pendingSub = await subscriptions.pendingSubscriptions(user);
  expect(pendingSub.start).to.eq(0);
  expect(pendingSub.end).to.eq(0);
  expect(pendingSub.rate).to.eq(0);

  const afterBlock = await latestBlockNumber();

  await testEpochDetails(
    subscriptions,
    start,
    end,
    rate,
    beforeBlock,
    afterBlock
  );

  return (await tx).blockNumber!;
}

async function addToSubscription(
  stableToken: StableToken,
  subscriptions: Subscriptions,
  signer: Account,
  user: string,
  amount: BigNumber,
  subscribeBlockNumber: number | undefined
) {
  // Before state
  const beforeSub = await subscriptions.subscriptions(user);
  const beforeBalance = await stableToken.balanceOf(signer.address);
  const beforeContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );
  const newEnd = beforeSub.end.add(amount.div(beforeSub.rate));
  // const additionalTokens = beforeSub.rate.mul(newEnd.sub(beforeSub.end));

  // * Tx
  const tx = subscriptions.connect(signer.signer).addTo(user, amount);

  // * Check events
  await expect(tx)
    .to.emit(subscriptions, 'Extend')
    .withArgs(user, beforeSub.end, newEnd, amount);

  // * Check balances
  const afterBalance = await stableToken.balanceOf(signer.address);
  const afterContractBalance = await stableToken.balanceOf(
    subscriptions.address
  );
  expect(afterBalance).to.eq(beforeBalance.sub(amount));
  expect(afterContractBalance).to.eq(
    beforeContractBalance.add(amount)
  );

  // * Check state
  const afterSub = await subscriptions.subscriptions(user);
  expect(afterSub.start).to.eq(beforeSub.start);
  expect(afterSub.end).to.eq(newEnd);
  expect(afterSub.rate).to.eq(beforeSub.rate);

  // Sub + extend -> Epoch changes should match those of a sub [start, newEnd)
  await testEpochDetails(
    subscriptions,
    beforeSub.start,
    newEnd,
    beforeSub.rate,
    BigNumber.from(subscribeBlockNumber! - 1),
    BigNumber.from((await tx).blockNumber!)
  );
}
