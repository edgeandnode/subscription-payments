import '@nomicfoundation/hardhat-chai-matchers';

import {expect} from 'chai';
import * as deployment from '../utils/deploy';
import {getAccounts, Account} from '../utils/helpers';

import {Registry} from '../types/contracts/Registry';
import {BigNumber, ethers} from 'ethers';
import {setAutoMine} from './helpers';

describe('Registry contract', () => {
  // Accounts
  let deployer: Account;
  let other: Account;

  // Contracts
  let registry: Registry;

  before(async function () {
    [deployer, other] = await getAccounts();

    setAutoMine(true);
  });

  beforeEach(async function () {
    registry = await deployment.deployRegistry(deployer.signer, false);
  });

  describe('constructor', function () {
    it('should set the owner to the contract deployer address', async function () {
      expect(await registry.owner()).to.eq(deployer.address);
    });
  });

  describe('transferOwnership', function () {
    it('should set the owner to the new owner', async function () {
      await registry.transferOwnership(other.address);
      expect(await registry.owner()).to.eq(other.address);
    });
  });

  describe('insertEntry', function () {
    it('should set entry', async function () {
      const entryID = BigNumber.from(1);
      const entry = {
        subscriptions: [
          '0x0000000000000000000000000000000000000001',
          '0x0000000000000000000000000000000000000002',
        ],
        metadataHash: '0x0000000000000000000000000000000000000000000000000000000000000001',
      };
      await registry.insertEntry(entryID, entry);
      const result = await registry.getEntry(entryID);
      expect(result[0]).to.eql(entry.subscriptions);
      expect(result[1]).to.eql(entry.metadataHash);
    });

    it('should overwrite entry', async function () {
      const entryID = BigNumber.from(1);
      const entry1 = {
        subscriptions: ['0x0000000000000000000000000000000000000001'],
        metadataHash: '0x0000000000000000000000000000000000000000000000000000000000000001',
      };
      const entry2 = {
        subscriptions: ['0x0000000000000000000000000000000000000002'],
        metadataHash: '0x0000000000000000000000000000000000000000000000000000000000000002',
      };
      await registry.insertEntry(entryID, entry1);
      await registry.insertEntry(entryID, entry2);
      const result = await registry.getEntry(entryID);
      expect(result[0]).to.eql(entry2.subscriptions);
      expect(result[1]).to.eql(entry2.metadataHash);
    });
  });

  describe('removeEntry', function () {
    it('should remove entry', async function () {
      const entryID = BigNumber.from(1);
      const subscriptions = ['0x0000000000000000000000000000000000000001'];
      const metadataHash =
        '0x0000000000000000000000000000000000000000000000000000000000000001';
      await registry.insertEntry(entryID, {subscriptions, metadataHash});
      await registry.removeEntry(entryID);
      const result = await registry.getEntry(entryID);
      expect(result[0]).to.eql([]);
      expect(result[1]).to.eql(ethers.constants.HashZero);
    });
  });
});
