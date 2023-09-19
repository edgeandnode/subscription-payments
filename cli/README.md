# subscriptions CLI

example creating a subscription on Arbitrum Goerli:

```bash
# see ../contracts/addresses.json for subscriptions contract addresses
cargo run <secret-key-hex.txt -- \
  --provider=https://goerli-rollup.arbitrum.io/rpc \
  --chain-id=421613 \
  --subscriptions=0x29f49a438c747e7Dd1bfe7926b03783E47f9447B \
  --token=0x8fb1e3fc51f3b789ded7557e680551d93ea9d892 \
  subscribe --end="$(date -u '+%Y-%m-%dT%TZ' --date='5 day')" --rate=1
```
