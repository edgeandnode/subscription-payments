# Graph Subscription Standard

Prototype Contract for Subscription Payments

## Test Subgraph

- `docker compose build`
- `docker compose up`

```bash
(cd contracts && yarn && yarn build && yarn deploy-local)
yq ".dataSources[0].source.address |= \"$(jq <contracts/contract-deployment.json '.contract' -r)\"" \
  -i subgraph/subgraph.yaml
yq ".dataSources[0].network |= \"hardhat\"" \
  -i subgraph/subgraph.yaml
echo "waiting for graph-node..."
while true; do curl -sf "localhost:8020"; [ $? -eq 22 ] && break; sleep 1; done
(cd subgraph && yarn && yarn create-local && yarn deploy-local)
```

```bash
echo "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" | cargo run -- \
  --subscriptions=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  --token=0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  subscribe --end="$(date -u '+%Y-%m-%dT%TZ' --date='10 min')" --rate=100000000000000
```

```graphql
{
  inits {
    token
  }
  subscribes {
    user {
      id
    }
    start
    end
    rate
  }
  unsubscribes {
    user {
      id
    }
  }
  userSubscriptions {
    user {
      id
    }
    start
    end
    rate
    cancelled
  }
  users {
    id
    authorizedSigners {
      id
    }
  }
}
```
