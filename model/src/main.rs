use anyhow::{ensure, Result};
use modelcheck::{
    rand::{rngs::SmallRng, Rng},
    Arbitrary, FailedState, ModelChecker, ModelState,
};
use std::{
    collections::BTreeMap,
    ops::{Range, RangeInclusive},
};

type Address = u8;

#[derive(Clone, Debug, Default)]
struct Model {
    block: u8,
    balances: BTreeMap<Address, u64>,
    subs: BTreeMap<Address, Subscription>,
    epochs: BTreeMap<u8, Epoch>,
    uncollected_epoch: u8,
    collect_per_epoch: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Subscription {
    start: u8,
    end: u8,
    rate: u8,
}

#[derive(Clone, Debug, Default)]
struct Epoch {
    delta: i64,
    extra: i64,
}

impl Model {
    fn value(sub: &Subscription) -> u64 {
        sub.rate as u64 * (sub.end - sub.start) as u64
    }

    fn locked(&self, sub: &Subscription) -> u64 {
        sub.rate as u64 * (sub.end.min(self.block) as i8 - sub.start as i8).max(0) as u64
    }

    fn unlocked(&self, sub: &Subscription) -> u64 {
        sub.rate as u64 * (sub.end as i8 - sub.start.max(self.block) as i8).max(0) as u64
    }

    fn collectable(&self) -> u64 {
        self.subs.values().map(|sub| self.locked(sub)).sum()
    }

    fn block_to_epoch(block: u8) -> u8 {
        ((block / EPOCH_BLOCKS) + (block % EPOCH_BLOCKS).min(1)).max(1)
    }

    fn balance(&self, addr: Address) -> u64 {
        self.balances.get(&addr).copied().unwrap_or_default()
    }

    fn sub(&self, addr: Address) -> Subscription {
        self.subs.get(&addr).cloned().unwrap_or_default()
    }

    fn epoch(&self, index: u8) -> Epoch {
        self.epochs.get(&index).cloned().unwrap_or_default()
    }

    fn transfer(&mut self, from: Address, to: Address, amount: u64) -> Result<()> {
        ensure!(self.balance(from) >= amount);
        *self.balances.entry(from).or_default() -= amount;
        *self.balances.entry(to).or_default() += amount;
        Ok(())
    }

    fn set_epochs(&mut self, start: u8, end: u8, rate: i8) {
        let e = Self::block_to_epoch(self.block);
        let e1 = Self::block_to_epoch(start);
        if e <= e1 {
            self.epochs.entry(e1).or_default().delta += rate as i64 * EPOCH_BLOCKS as i64;
            self.epochs.entry(e1).or_default().extra -=
                rate as i64 * (start - ((e1 - 1) * EPOCH_BLOCKS)) as i64;
        }
        let e2 = Self::block_to_epoch(end);
        if e <= e2 {
            self.epochs.entry(e2).or_default().delta -= rate as i64 * EPOCH_BLOCKS as i64;
            self.epochs.entry(e2).or_default().extra +=
                rate as i64 * (end - ((e2 - 1) * EPOCH_BLOCKS)) as i64;
        }
    }

    fn next_block(&mut self) -> Result<()> {
        self.block += 1;
        Ok(())
    }

    fn collect(&mut self) -> Result<()> {
        let mut total = 0;
        while self.uncollected_epoch < Self::block_to_epoch(self.block) {
            let epoch = self.epoch(self.uncollected_epoch);
            self.collect_per_epoch += epoch.delta;
            total += self.collect_per_epoch + epoch.extra;
            self.uncollected_epoch += 1;
        }
        let total = total.try_into().unwrap();
        self.transfer(CONTRACT, OWNER, total).unwrap();
        Ok(())
    }

    fn subscribe(&mut self, addr: Address, mut sub: Subscription) -> Result<()> {
        sub.start = sub.start.max(self.block);
        let prev = self.sub(addr);
        ensure!((addr != CONTRACT) && (prev.end <= self.block) && (sub.start < sub.end));
        *self.balances.entry(addr).or_default() += self.unlocked(&sub);
        self.transfer(addr, CONTRACT, self.unlocked(&sub))?;

        self.set_epochs(sub.start, sub.end, sub.rate as i8);
        self.subs.insert(addr, sub);
        Ok(())
    }

    fn unsubscribe(&mut self, addr: Address) -> Result<()> {
        let sub = self.sub(addr);
        if (sub.start <= self.block) && (self.block < sub.end) {
            self.set_epochs(sub.start, sub.end, -(sub.rate as i8));
            self.set_epochs(sub.start, self.block, sub.rate as i8);
            self.subs.get_mut(&addr).unwrap().end = self.block;
        } else if self.block < sub.start {
            self.set_epochs(sub.start, sub.end, -(sub.rate as i8));
            self.subs.remove(&addr);
        }
        self.transfer(CONTRACT, addr, self.unlocked(&sub)).unwrap();
        Ok(())
    }

    fn extend(&mut self, addr: Address, end: u8) -> Result<()> {
        let sub = self.sub(addr);
        ensure!((sub.start < self.block) && (self.block < sub.end));
        ensure!(sub.end < end);
        let addition = sub.rate as u64 * (end - sub.end) as u64;
        self.transfer(addr, CONTRACT, addition)?;
        self.set_epochs(sub.start, sub.end, -(sub.rate as i8));
        self.set_epochs(sub.start, end, sub.rate as i8);
        self.subs.get_mut(&addr).unwrap().end = end;
        Ok(())
    }

    fn check(&self) {
        assert!(
            self.sub(CONTRACT) == Subscription::default(),
            "subscription for contract"
        );
        // balance[CONTRACT] >= subs.map((_, sub) => unlocked(sub)).sum()
        assert!(
            self.balance(CONTRACT) >= self.subs.values().map(|sub| self.unlocked(sub)).sum(),
            "contract lacks sufficient tokens"
        );
        // [](ADDRS.filter(user => user != CONTRACT).all(user =>
        //   unsubscribe(user) -> balance'[user] == (balance[user] + unlocked(subs[user]))
        // ))
        assert!(
            ADDRS.filter(|addr| *addr != CONTRACT).all(|user| {
                let mut state = self.clone();
                let recoverable = self.unlocked(&self.sub(user));
                let _ = state.unsubscribe(user);
                state.balance(user) == (self.balance(user) + recoverable)
            }),
            "user failed to recover unlocked tokens"
        );
        // <>(collect() -> balance'[OWNER] >= (collectable -_ value(subs'[OWNER])))
        assert!(
            {
                let eventually = EPOCH_BLOCKS
                    * (Self::block_to_epoch(
                        self.subs.values().map(|sub| sub.end).max().unwrap_or(0),
                    ) + 1);
                let mut state = self.clone();
                state.block = state.block.max(eventually);
                let _ = state.collect();
                let owner_sub = Self::value(&state.sub(OWNER));
                state.balance(OWNER) >= self.collectable().saturating_sub(owner_sub)
            },
            "failed to collect"
        );
        // TODO: [](ADDRS.all(sub => extend(addr, _) ->
        //   unlocked'(subs'[addr]) >= unlocked(subs[addr]) &&
        //   locked'(subs'[addr]) == locked(subs[addr])
        // ))
        // TODO: [](ADDRS.filter(user => user != CONTRACT).all(user =>
        //   (balances'[user] + unlocked'(subs[user])) >=
        //   (balances[user] + (unlocked(subs[user]) - subs[user].rate))
        // ))
    }
}

#[derive(Clone, Debug)]
enum Step {
    NextBlock,
    Collect,
    Subscribe(Address, Subscription),
    Unsubscribe(Address),
    Extend(Address, u8),
}

const ADDRS: RangeInclusive<Address> = 0..=5;
const CONTRACT: Address = 0;
const OWNER: Address = 1;
const RATES: RangeInclusive<u8> = 1..=3;
const EPOCHS: RangeInclusive<u8> = 1..=10;
const EPOCH_BLOCKS: u8 = 5;
const BLOCKS: Range<u8> = 0..(EPOCH_BLOCKS * *EPOCHS.end());

impl Arbitrary for Model {
    fn gen(_rng: &mut SmallRng) -> Self {
        Self::default()
    }
}

impl Arbitrary for Step {
    fn gen(rng: &mut SmallRng) -> Self {
        match rng.gen_range(0..=4) {
            0 => Self::NextBlock,
            1 => Self::Collect,
            2 => Self::Subscribe(
                rng.gen_range(ADDRS),
                Subscription {
                    start: rng.gen_range(BLOCKS),
                    end: rng.gen_range(BLOCKS),
                    rate: rng.gen_range(RATES),
                },
            ),
            3 => Self::Unsubscribe(rng.gen_range(ADDRS)),
            4 => Self::Extend(rng.gen_range(ADDRS), rng.gen_range(BLOCKS)),
            _ => unreachable!(),
        }
    }
}

impl ModelState for Model {
    type Step = Step;

    fn step(&mut self, step: Self::Step) {
        let _ = match step {
            Step::NextBlock => self.next_block(),
            Step::Collect => self.collect(),
            Step::Subscribe(addr, sub) => self.subscribe(addr, sub),
            Step::Unsubscribe(addr) => self.unsubscribe(addr),
            Step::Extend(addr, end) => self.extend(addr, end),
        };
        self.check();
    }
}

fn main() {
    for _ in 0..1000 {
        run_model();
    }
}

fn run_model() {
    let mut modelchecker = ModelChecker::<Model>::default();
    if let Err(FailedState {
        mut state,
        mut steps,
        error,
    }) = modelchecker.run(100)
    {
        let last_step = steps.pop().unwrap();
        for step in steps {
            println!("{:?}", step);
            state.step(step);
        }
        println!("===");
        println!("state: {state:#?}");
        println!("last_step: {last_step:?}");
        println!("error: {error:#?}");
        state.step(last_step);
    }
}
