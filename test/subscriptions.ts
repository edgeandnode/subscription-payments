import {expect} from 'chai';
import {BigNumber, Contract} from 'ethers';
import {ethers, network} from 'hardhat';
import {xoshiro128ss} from './rng';
import {GraphToken__factory} from '@graphprotocol/contracts/dist/types/factories/GraphToken__factory';
import {hexDataSlice, randomBytes} from 'ethers/lib/utils';
import {SignerWithAddress} from '@nomiclabs/hardhat-ethers/signers';

interface Subscription {
  startBlock: number;
  endBlock: number;
  pricePerBlock: BigNumber;
}

const nullSub = {startBlock: 0, endBlock: 0, pricePerBlock: BigNumber.from(0)};

const genInt = (rng: () => number, min: number, max: number): number =>
  Math.floor(rng() * (max - min + 1) + min);

function genOp(rng: () => number, users: number): any {
  switch (genInt(rng, 0, 3)) {
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
    default:
      throw 'unreachable';
  }
}

const genSub = (rng: () => number): Subscription => ({
  startBlock: genInt(rng, -3, 3),
  endBlock: genInt(rng, 1, 9),
  pricePerBlock: BigNumber.from(10).pow(18).mul(genInt(rng, 1, 10_000)),
});

class Model {
  contract: Contract;
  token: Contract;
  owner: SignerWithAddress;
  block: number = 0;
  balances: Map<string, BigNumber> = new Map();
  subs: Map<string, Subscription> = new Map();
  uncollected: BigNumber = BigNumber.from(0);

  constructor(contract: Contract, token: Contract, owner: SignerWithAddress) {
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
    return sub.pricePerBlock.mul(
      Math.max(0, Math.min(this.block, sub.endBlock) - sub.startBlock)
    );
  }

  unlocked(sub: Subscription): BigNumber {
    return sub.pricePerBlock.mul(
      Math.max(0, sub.endBlock - Math.max(this.block, sub.startBlock))
    );
  }

  truncate(sub: Subscription): Subscription {
    const clamp = (x: number, min: number, max: number): number =>
      Math.min(max, Math.max(min, x));
    return {
      ...sub,
      startBlock: clamp(this.block, sub.startBlock, sub.endBlock),
    };
  }

  transfer(from: string, to: string, amount: BigNumber) {
    expect(this.balances.get(from)! >= amount);
    this.balances.set(from, this.balances.get(from)!.sub(amount));
    this.balances.set(to, this.balances.get(to)!.add(amount));
  }

  async check() {
    console.log('--- check block', this.block, '---');
    for (const [addr, balance] of this.balances) {
      // console.log({addr, balance});
      expect(await this.token.balanceOf(addr)).eq(balance);
      const sub = await this.contract.connect(this.owner).subscription(addr);
      const modelSub = this.subs.get(addr);
      // console.log({addr, sub, modelSub});
      expect(sub.startBlock).eq(modelSub?.startBlock);
      expect(sub.endBlock).eq(modelSub?.endBlock);
      expect(sub.pricePerBlock).eq(modelSub?.pricePerBlock);
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
    this.subs.forEach((sub, addr, map) => {
      map.set(addr, sub.endBlock <= this.block ? nullSub : this.truncate(sub));
    });
    this.uncollected = BigNumber.from(0);
    this.transfer(this.contract.address, this.owner.address, collectable);
    await this.contract.connect(this.owner).collect();
  }

  async subscribe(user: SignerWithAddress, sub: Subscription) {
    sub = {
      ...sub,
      startBlock: this.block + sub.startBlock,
      endBlock: this.block + sub.endBlock,
    };
    const startBlock = Math.max(this.block, sub.startBlock);
    sub.endBlock = Math.max(sub.endBlock, startBlock + 1);
    if (this.subs.get(user.address)!.endBlock > this.block) {
      console.log('subscribe', 'skip');
      return;
    }
    const value = sub.pricePerBlock.mul(sub.endBlock - startBlock);
    console.log('subscribe', {
      user: user.address,
      ...sub,
      pricePerBlock: sub.pricePerBlock.toString(),
      value: value.toString(),
    });
    const prev = this.subs.get(user.address)!;
    this.uncollected = this.uncollected.add(
      prev.pricePerBlock.mul(prev.endBlock - prev.startBlock)
    );
    this.subs.set(user.address, {...sub, startBlock});
    this.transfer(user.address, this.contract.address, value);
    await this.token.connect(user).approve(this.contract.address, value);
    await this.contract
      .connect(user)
      .subscribe(user.address, sub.startBlock, sub.endBlock, sub.pricePerBlock);
  }

  async unsubscribe(user: SignerWithAddress) {
    const sub = this.subs.get(user.address)!;
    if (sub.endBlock >= this.block) {
      console.log('unsubscribe', 'skip');
      return;
    }
    console.log('unsubscribe', {user: user.address});
    this.transfer(this.contract.address, user.address, this.unlocked(sub));
    this.uncollected = this.uncollected.add(this.locked(sub));
    this.subs.set(user.address, nullSub);
    await this.contract.connect(user).unsubscribe();
  }

  async exec(op: any) {
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
        await this.subscribe(await this.user(op.user), op.sub);
        break;
      case 'unsubscribe':
        await this.unsubscribe(await this.user(op.user));
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
  const contract = await (
    await ethers.getContractFactory('Subscriptions')
  ).deploy(token.address);
  await network.provider.send('evm_mine');
  for (const signer of users) {
    await token.transfer(signer.address, initialBalance);
    await token.connect(signer).approve(contract.address, initialBalance);
  }
  await network.provider.send('evm_mine');

  const seed = hexDataSlice(randomBytes(32), 0);
  // const seed =
  //   '0xc72fd53b513f15f62a85eb36ca6547451f650c72ce89788763c3e5589ac94437';
  const rng = xoshiro128ss(seed);
  let steps = Array.from(Array(16).keys()).map(_ => genOp(rng, users.length));
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

  // steps = steps.filter((_, i) => [3, 5, 7, 9, 11].includes(i));
  await model.check();
  for (let op of steps) {
    await model.exec(op);
  }
  await model.exec({opcode: 'nextBlock'});
});
