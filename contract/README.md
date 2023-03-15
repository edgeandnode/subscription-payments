# Graph Subscriptions Contract

## Contract Deployment

To deploy the contract run:

```bash
PRIVATE_KEY=<> hh deploy --token <STABLE_COIN_ADDRESS> --network <arbitrum-goerli|arbitrum-one>
```

Alternatively you can use the env var `MNEMONIC` to deploy the contract and it will pick the first derived address.

## Tests

To test the contract run:

```bash
yarn test
```

## Contract Design

This contract is designed to allow users of the Graph Protocol to pay gateways for their services
with limited risk of losing tokens. The contract itself makes no assumptions about how the
subscription rate is interpreted by the contract owner users open subscriptions with.

A user's subscription represents a lockup of `rate` tokens per second for the half-open timestamp
range `[start, end)`. The total value of the subscription is `rate * (end - start)`, which is the
amount of tokens the user must tranfer to the contract upon calling `subscribe`. The user may
recover the total value of the subscription via `unsubscribe` up to the start timestamp. The amount
of tokens recoverable by the user decreases at `rate` tokens per block until the `end` timestamp,
when the recoverable amount becomes 0. The contract owner may only collect tokens that are no longer
recoverable by any the user.

### Epochs

For the subscriptions contract, an epoch defines the minimum time frame after which tokens from user
subscriptions can be collected by the contract owner (the gateway operator). The epoch length
(`epochSeconds`) is set once, when the contract is deployed, and cannot be modified.

For example:
An epoch length is 2 hours and a user subscribes for 5 hours at a rate of 10 tokens per hour
starting immediately upon contract creation. 20 tokens will become collectable after 2 hours, 20
more after 4 hours, and 10 more after 6 hours.

The tradeoff is that a longer epoch time is more gas-efficient for the owner to collect, but they
obviously will have more lag before they can collect what they are owed from the contract. Doubling
the epoch time approximately halves the gas cost of collection. (edited)

## Upgrade Strategy

The expected strategy for upgrading or deploying a new contract is as follows:

1. Deploy the new contract
2. Direct users to open any new subscriptions on the new contract

The owner is expected to respect the subscriptions from the old contract, until some reasonable
amount of time has passed. Note that the owner's interpretation of the subscriptions is not enforced
by the contract. So there's currently no need to encode this upgrade strategy in the contracts
either. It is currently up to users to tranfer their subscriptions to the new contract, if they
want to. Allowing the gateway to force this transfer to a new contract would allow the owner to
also break the security guarantees of the contract by suddenly locking all tokens that should
otherwise be recoverable by the users.
