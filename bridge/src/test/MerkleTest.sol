// SPDX-License-Identifier: Uqbar

pragma solidity ^0.8.9;

import { Merkle } from "../libraries/Merkle.sol";

contract MerkleTest {
  bool public res;
  bytes32 public hashEncode;
  bytes32 public hashEncodePacked;

  function mtestProof(
    bytes32 _root, bytes32 _node, bytes memory _proof, uint8 _depth) 
  public {
    res = Merkle.checkProof(_root, _node, _proof, _depth);
  }

  function mtestHashing(bytes32 a, bytes32 b) public {
    hashEncode = keccak256(abi.encode(a, b)); 
    hashEncodePacked = keccak256(abi.encodePacked(a, b)); 
  }
}
