# Graph Subscription Standard

Prototype Contract for Subscription Payments

## Test Subgraph

- `docker compose build`
- `docker compose up`

```bash
(cd contract && npx hardhat compile && yarn deploy-local)
yq ".dataSources[0].source.address |= \"$(jq <contract/contract-deployment.json '.contract' -r)\"" \
  subgraph/subgraph.yaml -iy
yq ".dataSources[0].network |= \"hardhat\"" \
  subgraph/subgraph.yaml -iy
```

```bash
(cd subgraph && yarn create-local && yarn deploy-local)
cd contract && npx hardhat console --network localhost
```

```typescript
await network.provider.send('evm_mine');
```

```bash
echo "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" | cargo run -- \
  --subscriptions=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  --token=0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  subscribe --end-block=10 --price-per-block=100000000000000
```

```graphql
{
  inits {
    token
  }
  subscribes {
    user
    start
    end
    rate
  }
  unsubscribes {
    user
  }
  activeSubscriptions {
    user
    start
    end
    rate
  }
}
```
