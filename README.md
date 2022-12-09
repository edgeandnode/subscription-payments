# subscription-payments-contract

Prototype Contract for Subscription Payments

## Test Subgraph

- `docker compose up`

```bash
npx hardhat run --network localhost scripts/deploy.ts | tee contract-deployment.json
yq ".dataSources[0].source.address |= \"$(jq <contract-deployment.json '.contract' -r)\"" \
  -i subgraph/subgraph.yaml
cd subgraphs && yarn create-local && yarn deploy-local
```
