import '@nomicfoundation/hardhat-chai-matchers';
import '@nomiclabs/hardhat-ethers';

import {expect} from 'chai';
import * as deployment from '../utils/deploy';
import {getAccounts, Account, toGRT, toBN, floorBN} from '../utils/helpers';

import {Subscriptions} from '../types/contracts/Subscriptions';
import {StableToken} from '../types/contracts/test/StableMock.sol/StableToken';
import {BigNumber, ethers} from 'ethers';
import {network} from 'hardhat';

const tenBillion = toGRT('10000000000');
const oneHundred = toGRT('100');
const oneMillion = toGRT('1000000');

describe('Subscriptions contract', () => {
  // Accounts
  let deployer: Account;
  let subscriber1: Account;
  let subscriber2: Account;
  let subscriber3: Account;

  // Contracts
  let subscriptions: Subscriptions;
  let stableToken: StableToken;

  // Constructor params
  const subscriptionsEpochBlocks = BigNumber.from(100);

  before(async function () {
    // eslint-disable-next-line @typescript-eslint/no-extra-semi
    [deployer, subscriber1, subscriber2, subscriber3] = await getAccounts();

    await network.provider.send('evm_setAutomine', [true]);
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

      console.log(`n is ${n}`);
      console.log(`startBlock is ${startBlock}`);
      console.log(`epochBlocks is ${epochBlocks}`);
      
      
      expect(await subscriptions.blockToEpoch(startBlock)).to.eq(n);
    });

    it('should end the nth epoch at n*epochBlocks', async function () {
      const n = Math.floor(Math.random() * 1000) + 1;
      const epochBlocks = await subscriptions.epochBlocks();
      const endBlock = epochBlocks.mul(n).sub(1);

      expect(await subscriptions.blockToEpoch(endBlock)).to.eq(n);
    });
  });

  // describe('fulfil', function () {
  //   it('should fulfil orders', async function () {
  //     const beforeBillingBalance = await billing.userBalances(user1.address)
  //     const beforeServiceBalance = await token.balanceOf(banxaFulfillmentService.address)

  //     const tx = banxaWrapper.connect(banxaFulfillmentService.signer).fulfil(user1.address, oneHundred)
  //     await expect(tx)
  //       .emit(banxaWrapper, 'OrderFulfilled')
  //       .withArgs(banxaFulfillmentService.address, user1.address, oneHundred)

  //     const afterBillingBalance = await billing.userBalances(user1.address)
  //     const afterServiceBalance = await token.balanceOf(banxaFulfillmentService.address)

  //     expect(afterBillingBalance).eq(beforeBillingBalance.add(oneHundred))
  //     expect(afterServiceBalance).eq(beforeServiceBalance.sub(oneHundred))
  //   })

  //   it('should fail to fulfil orders for address(0)', async function () {
  //     const tx = banxaWrapper.connect(banxaFulfillmentService.signer).fulfil(AddressZero, oneHundred)
  //     await expect(tx).revertedWithCustomError(banxaWrapper, 'InvalidZeroAddress')
  //   })

  //   it('should fail to fulfil orders with zero tokens', async function () {
  //     const tx = banxaWrapper.connect(banxaFulfillmentService.signer).fulfil(user1.address, toBN(0))
  //     await expect(tx).revertedWithCustomError(banxaWrapper, 'InvalidZeroAmount')
  //   })
  // })

  // describe('rescue', function () {
  //   it('should rescue tokens', async function () {
  //     // deploy token2 and accidentally send to the BanxaWrapper contract
  //     const token2 = await deployment.deployToken([tenBillion], me.signer, true)
  //     await token2.connect(me.signer).transfer(user1.address, oneMillion)
  //     await token2.connect(user1.signer).transfer(banxaWrapper.address, oneMillion)

  //     // the bad transfer of GRT
  //     await token.connect(user1.signer).transfer(banxaWrapper.address, oneMillion)

  //     const tokenBeforeUser = await token.balanceOf(user1.address)
  //     const token2BeforeUser = await token2.balanceOf(user1.address)
  //     const tokenBeforeBanxa = await token.balanceOf(banxaWrapper.address)
  //     const token2BeforeBanxa = await token2.balanceOf(banxaWrapper.address)

  //     const tx = await banxaWrapper.connect(governor.signer).rescueTokens(user1.address, token.address, oneMillion)
  //     await expect(tx).emit(banxaWrapper, 'TokensRescued').withArgs(user1.address, token.address, oneMillion)
  //     await banxaWrapper.connect(governor.signer).rescueTokens(user1.address, token2.address, oneMillion)

  //     const tokenAfterUser = await token.balanceOf(user1.address)
  //     const token2AfterUser = await token2.balanceOf(user1.address)
  //     const tokenAfterBanxa = await token.balanceOf(banxaWrapper.address)
  //     const token2AfterBanxa = await token2.balanceOf(banxaWrapper.address)

  //     expect(tokenAfterUser).eq(tokenBeforeUser.add(oneMillion))
  //     expect(token2AfterUser).eq(token2BeforeUser.add(oneMillion))
  //     expect(tokenAfterBanxa).eq(tokenBeforeBanxa.sub(oneMillion))
  //     expect(token2AfterBanxa).eq(token2BeforeBanxa.sub(oneMillion))
  //   })

  //   it('should fail rescue tokens when not the governor', async function () {
  //     // the bad transfer of GRT
  //     await token.connect(user1.signer).transfer(banxaWrapper.address, oneMillion)
  //     const tx = banxaWrapper.connect(user1.signer).rescueTokens(user1.address, token.address, oneMillion)
  //     await expect(tx).revertedWith('Only Governor can call')
  //   })

  //   it('should fail when trying to send to address zero', async function () {
  //     // the bad transfer of GRT
  //     await token.connect(user1.signer).transfer(banxaWrapper.address, oneMillion)
  //     const tx = banxaWrapper.connect(governor.signer).rescueTokens(AddressZero, token.address, oneMillion)
  //     await expect(tx).revertedWith('Cannot send to address(0)')
  //   })

  //   it('should fail when trying to send zero tokens', async function () {
  //     // the bad transfer of GRT
  //     await token.connect(user1.signer).transfer(banxaWrapper.address, oneMillion)
  //     const tx = banxaWrapper.connect(governor.signer).rescueTokens(user1.address, token.address, toBN(0))
  //     await expect(tx).revertedWith('Cannot rescue 0 tokens')
  //   })
  // })
});
