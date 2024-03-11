// SPDX-License-Identifier: Uqbar

pragma solidity ^0.8.9;

import { RLPReader } from "../libraries/RLPReader.sol";

contract DataAvailability {
  // for storing test output
  uint256 public uint256Tmp;
  uint256 public uint256Tmp2;
  bool public boolTmp;

  constructor() {
    uint256Tmp = 33;
  }

  function calldataTest(bytes calldata _in) external {
    require(2 > 1);
    bytes32 hash = keccak256(_in); 
    // emit DepositCreated(5, hash);
  }

  function rlpExtractListTest(bytes calldata _in) external {
    RLPReader.RLPItem memory ri = RLPReader.toRlpItem(_in);
    
    RLPReader.RLPItem[] memory ris = RLPReader.toList(ri);

    uint256Tmp = ris.length;
    uint256Tmp2 = RLPReader.rlpLen(ris[3]);
    boolTmp = RLPReader.isList(ri);
  }

  function getItem(bytes calldata _in, uint64 index) external {
    RLPReader.RLPItem memory ri = RLPReader.toRlpItem(_in);
    
    RLPReader.RLPItem memory itemAtIndex = getAtIndex(ri, index);
    boolTmp = true; 
  }

  function getAtIndex(RLPReader.RLPItem memory ri, uint64 index)
    internal pure returns (RLPReader.RLPItem memory)
  {
    RLPReader.Iterator memory iter = RLPReader.iterator(ri);

    uint64 i = 0;
    while(RLPReader.hasNext(iter)) {
      if(i == index) {
        return RLPReader.next(iter);
      }
      RLPReader.next(iter);
      i++;
    }
    revert("index out of bounds for list");
  }

  // test hashing multiple things in a row
  function multiHash(bool _in) external returns (uint256) {
    bytes32 z; 
    uint16 x = 10;
    bytes1 z1 = 0x0;
    bytes1 mask = 0x01;
    uint256 bigger = 0x928282;
    uint256 big_mask = 0x01;
    bytes memory zero = abi.encodePacked(z);
    bytes memory zero_2x = abi.encodePacked(z, z);
    keccak256(zero);
    keccak256(zero);
    keccak256(zero);
    keccak256(zero);
    keccak256(zero_2x);
    keccak256(zero_2x);
    keccak256(zero);
    uint256Tmp = 2;
    mask = 0x0;
    /*
    if (_in) { 
      z1 = 0x01;
    }
    */
    //return bigger;
    return bigger & big_mask;
  }
}
