import {expect} from 'chai';
import {BigNumber} from 'ethers';
import {ethers, network} from 'hardhat';
import {GraphToken__factory} from '@graphprotocol/contracts/dist/types/factories/GraphToken__factory';
import {GraphToken} from '@graphprotocol/contracts/dist/types/GraphToken';
import {hexDataSlice, randomBytes} from 'ethers/lib/utils';
import {SignerWithAddress} from '@nomiclabs/hardhat-ethers/signers';

import {xoshiro128ss} from './rng';

import {Subscriptions} from '../types';
import {Subscriptions__factory} from '../types/factories/contracts/Subscriptions__factory';

interface Subscription {
  start: number;
  end: number;
  rate: BigNumber;
}
interface Op {
  opcode: 'nextBlock' | 'collect' | 'subscribe' | 'unsubscribe' | 'extend';
  user?: number;
  sub?: Subscription;
  end?: number;
}

const nullSub = {start: 0, end: 0, rate: BigNumber.from(0)};

const genInt = (rng: () => number, min: number, max: number): number =>
  Math.floor(rng() * (max - min + 1) + min);

function genOp(rng: () => number, users: number): Op {
  switch (genInt(rng, 0, 4)) {
    case 0:
      return {opcode: 'nextBlock'};
    case 1:
      return {opcode: 'collect'};
    case 2:
      return {
        opcode: 'subscribe',
        user: genInt(rng, 0, users - 1),
        sub: genSub(rng),
      };
    case 3:
      return {opcode: 'unsubscribe', user: genInt(rng, 0, users - 1)};
    case 4:
      return {
        opcode: 'extend',
        user: genInt(rng, 0, users - 1),
        end: genSub(rng).end,
      };
    default:
      throw 'unreachable';
  }
}

const genSub = (rng: () => number): Subscription => ({
  start: genInt(rng, -3, 3),
  end: genInt(rng, 1, 9),
  rate: BigNumber.from(10).pow(18).mul(genInt(rng, 1, 10_000)),
});

class Model {
  contract: Subscriptions;
  token: GraphToken;
  owner: SignerWithAddress;
  block: number = 0;
  balances: Map<string, BigNumber> = new Map();
  subs: Map<string, Subscription> = new Map();
  uncollected: BigNumber = BigNumber.from(0);

  constructor(
    contract: Subscriptions,
    token: GraphToken,
    owner: SignerWithAddress
  ) {
    this.contract = contract;
    this.token = token;
    this.owner = owner;
  }

  user(i: number): Promise<SignerWithAddress> {
    const users = Array.from(this.balances.keys()).filter(
      a => a != this.contract.address
    );
    return ethers.getSigner(users[i]);
  }

  locked(sub: Subscription): BigNumber {
    return sub.rate.mul(Math.max(0, Math.min(this.block, sub.end) - sub.start));
  }

  unlocked(sub: Subscription): BigNumber {
    return sub.rate.mul(Math.max(0, sub.end - Math.max(this.block, sub.start)));
  }

  transfer(from: string, to: string, amount: BigNumber) {
    expect(this.balances.get(from)! >= amount);
    this.balances.set(from, this.balances.get(from)!.sub(amount));
    this.balances.set(to, this.balances.get(to)!.add(amount));
  }

  async check() {
    console.log('--- check block', this.block, '---');
    for (const [addr, balance] of this.balances) {
      // console.log({addr, balance: balance.toString()});
      if (addr == this.owner.address) {
        expect(await this.token.balanceOf(addr)).lte(balance);
      } else if (addr == this.contract.address) {
        expect(await this.token.balanceOf(addr)).gte(balance);
      } else {
        expect(await this.token.balanceOf(addr)).eq(balance);
      }
      const sub = await this.contract.connect(this.owner).subscription(addr);
      const modelSub = this.subs.get(addr);
      // console.log({addr, sub, modelSub});
      expect(sub.start).eq(modelSub?.start);
      expect(sub.end).eq(modelSub?.end);
      expect(sub.rate).eq(modelSub?.rate);
    }
  }

  async collect() {
    const collectable = this.uncollected.add(
      Array.from(this.subs.values()).reduce(
        (sum, sub) => sum.add(this.locked(sub)),
        BigNumber.from(0)
      )
    );
    console.log('collect', {collectable: collectable.toString()});
    this.uncollected = BigNumber.from(0);
    this.transfer(this.contract.address, this.owner.address, collectable);
    await this.contract.connect(this.owner)['collect()']();
  }

  async subscribe(user: SignerWithAddress, sub: Subscription) {
    sub = {
      ...sub,
      start: this.block + sub.start,
      end: this.block + sub.end,
    };
    const start = Math.max(this.block, sub.start);
    sub.end = Math.max(sub.end, start + 1);
    if (this.subs.get(user.address)!.end > this.block) {
      console.log('subscribe', 'skip');
      return;
    }
    const value = sub.rate.mul(sub.end - start);
    console.log('subscribe', {
      user: user.address,
      ...sub,
      rate: sub.rate.toString(),
      value: value.toString(),
    });
    const prev = this.subs.get(user.address)!;
    this.uncollected = this.uncollected.add(
      prev.rate.mul(prev.end - prev.start)
    );
    this.subs.set(user.address, {...sub, start});
    this.transfer(user.address, this.contract.address, value);
    await this.token.connect(user).approve(this.contract.address, value);
    await this.contract
      .connect(user)
      .subscribe(user.address, sub.start, sub.end, sub.rate);
  }

  async unsubscribe(user: SignerWithAddress) {
    const sub = this.subs.get(user.address)!;
    if (sub.end >= this.block) {
      console.log('unsubscribe', 'skip');
      return;
    }
    console.log('unsubscribe', {user: user.address});
    this.transfer(this.contract.address, user.address, this.unlocked(sub));
    this.uncollected = this.uncollected.add(this.locked(sub));
    this.subs.set(user.address, nullSub);
    await this.contract.connect(user).unsubscribe();
  }

  async extend(user: SignerWithAddress, end: number) {
    const sub = this.subs.get(user.address)!;
    if (sub.end >= end || this.block < sub.start || sub.end <= this.block) {
      console.log('extend', 'skip');
      return;
    }
    console.log('extend', {user: user.address, end});
    const addition = sub.rate.mul(end - sub.end);
    this.transfer(user.address, this.contract.address, addition);
    sub.end = end;
    await this.token.connect(user).approve(this.contract.address, addition);
    await this.contract.connect(user).extend(user.address, end);
  }

  async exec(op: Op) {
    switch (op.opcode) {
      case 'nextBlock':
        this.block += 1;
        await network.provider.send('evm_mine');
        await this.check();
        break;
      case 'collect':
        await this.collect();
        break;
      case 'subscribe':
        await this.subscribe(await this.user(op.user!), op.sub!);
        break;
      case 'unsubscribe':
        await this.unsubscribe(await this.user(op.user!));
        break;
      case 'extend':
        await this.extend(await this.user(op.user!), op.end!);
        break;
      default:
        throw 'unreachable';
    }
  }
}

it('Model Test', async () => {
  const users = (await ethers.getSigners()).slice(0, 4);
  const owner = users[0];

  const initialBalance = BigNumber.from(10).pow(18 + 6);
  const token = await new GraphToken__factory(owner).deploy(
    initialBalance.mul(users.length)
  );
  const Subscriptions: Subscriptions__factory = await ethers.getContractFactory(
    'Subscriptions'
  );
  const contract: Subscriptions = await Subscriptions.deploy(token.address, 3);
  await network.provider.send('evm_mine');
  for (const signer of users) {
    await token.transfer(signer.address, initialBalance);
    await token.connect(signer).approve(contract.address, initialBalance);
  }
  await network.provider.send('evm_mine');

  const seed = hexDataSlice(randomBytes(32), 0);
  // const seed =
  //   '0xfce3b338b405a94228c054d395cce9c5a89fde9392ed625548446f8e17d4cdac';
  const rng = xoshiro128ss(seed);
  let steps = Array.from(Array(32).keys()).map(_ => genOp(rng, users.length));
  console.log({
    contract: contract.address,
    owner: owner.address,
    users: users
      .filter(user => user.address != owner.address)
      .map(user => user.address),
    seed,
    steps,
  });

  let model = new Model(contract, token, owner);
  model.block = 1 + (await ethers.provider.getBlockNumber());
  for (const signer of [contract, ...users]) {
    model.balances.set(
      signer.address,
      signer.address == contract.address ? BigNumber.from(0) : initialBalance
    );
    model.subs.set(signer.address, nullSub);
  }

  // steps = steps.filter((_, i) =>
  //   [6, 9, 10, 11, 12, 14, 16, 26, 28, 29, 30, 31].includes(i)
  // );
  await model.check();
  for (let op of steps) {
    await model.exec(op);
  }
  await model.exec({opcode: 'nextBlock'});
});
