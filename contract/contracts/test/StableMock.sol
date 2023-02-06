// SPDX-License-Identifier: MIT

pragma solidity ^0.8.17;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract StableToken is ERC20 {
    constructor (uint256 _initialSupply) ERC20("Mock Stablecoin", "USDC"){
      _mint(msg.sender, _initialSupply);
    }
}