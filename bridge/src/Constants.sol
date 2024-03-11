pragma solidity ^0.8.9;

library BRIDGE {
  address constant TOKEN_CONTRACT_ETH  = 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;
  uint256 constant NULL_TOKEN_ID = 0;
  uint256 constant NULL_AMOUNT = 0;
}

// 30 days
uint256 constant MAX_ROLLBACK_WINDOW = 216000;
uint256 constant TWO_DAYS_BLOCKS = 14400;

uint256 constant DEFAULT_MIN_FEE = 0.00000001 ether;
uint256 constant DEFAULT_MAX_FEE = 1000 ether;
