#!/bin/bash
set -eu

mkdir -p build
jq '.abi' <artifacts/contracts/Subscriptions.sol/Subscriptions.json \
    | tee build/Subscriptions.abi
jq '.abi' <artifacts/@openzeppelin/contracts/token/ERC20/IERC20.sol/IERC20.json \
    | tee build/IERC20.abi
