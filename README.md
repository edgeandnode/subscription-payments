# Graph Subscription Standard

Prototype Contract for Subscription Payments

## Test Subgraph

- `docker compose build`
- `docker compose up`

```bash
cd contract && yarn deploy-local
yq ".dataSources[0].source.address |= \"$(jq <contract/contract-deployment.json '.contract' -r)\"" \
  -i subgraph/subgraph.yaml
cd subgraph && yarn create-local && yarn deploy-local
```
