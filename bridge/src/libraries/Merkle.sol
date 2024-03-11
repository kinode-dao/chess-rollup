// SPDX-License-Identifier: Uqbar

pragma solidity ^0.8.9;

import { RLPReader } from "./RLPReader.sol";

library Merkle {

  /* 
  * @dev Returns the path element of a proof (left/right bits)
  */ 
  function getPath(bytes memory _proof) internal pure returns (uint256) {
    RLPReader.RLPItem memory proof = RLPReader.toRlpItem(_proof);
    RLPReader.Iterator memory iter = RLPReader.iterator(proof);
    require(RLPReader.hasNext(iter), "MERKLE_INVALID_PROOF");
    return RLPReader.toUint(RLPReader.next(iter)); 
  }

  /*
  * @dev Returns true if Merkle proof is valid. proof is bottom-up, left-to-right
  * @param root The Merkle root
  * @param node Already-hashed node to prove
  * @param proof Merkle proof to "zip" with node, from bottom-up. Must be RLP list
  */
  function checkProof(
    bytes32 _root, bytes32 _node, bytes memory _proof, uint8 _depth
  ) internal pure returns (bool) {
    return (_root == calcRoot(_node, _proof, _depth));
  }

  function calcRoot(bytes32 _node, bytes memory _proof, uint8 _depth)
  internal pure returns (bytes32) {
    // TODO make a version of this that takes a flag for whether to run getPath and check it against an index
    require(_depth > 0, "MERKLE_DEPTH_ZERO");
    RLPReader.RLPItem memory proof = RLPReader.toRlpItem(_proof);
    RLPReader.Iterator memory iter = RLPReader.iterator(proof);

    uint256 path = RLPReader.toUint(RLPReader.next(iter)); 
    bytes32 acc = _node; 
    
    while(RLPReader.hasNext(iter)) {
      require(_depth > 0, "MERKLE_PATH_TOO_LONG");
      bytes32 hash = bytes32(RLPReader.toUint(RLPReader.next(iter)));

      // Is hash left or right in tree? (0 Left; 1 Right)
      if(0 == path & 0x1) {
        acc = keccak256(abi.encodePacked(hash, acc));
      }
      else {
        acc = keccak256(abi.encodePacked(acc, hash));
      }

      path >>= 1;
      _depth -= 1;
    }
    require(_depth == 0, "MERKLE_PATH_TOO_SHORT");
    return acc;
  }
}

/*
* MList: an append-only list
  Represented as a list of 16-bit Merkle trees
* List indexes are [a, b]
*  - a: which tree this is (starts at 0; new trees go to head)
*  - b: index inside the tree (2^16 elements per tree)
*
* Each tree root is hash(nujm_tree_elts, Merkle_root)
* Each insertion increases num_tree_elts by 1
* Once num_tree_elts is 2^16, no new trees can be created
*
* When a new tree is created, num_tree_elts is 0, first index is 0.
* This increases by 1 with each new element
*
* No 2nd preimage attack, because depth is specified (16)
*/

library MList16 {
  struct Tree16 {
    bytes32 eltsRootHash;
    uint16 numElts;
    bytes32 root;
  }

  uint16 internal constant UINT16_MAX = 2**16 - 1;
  bytes32 internal constant ZERO_HASH = bytes32(uint256(
    0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a));
  bytes32 internal constant ROOT_16   = bytes32(uint256(
    0xaf9e71b24c04208929196c10464068800b2a96ea1314ca08b3fa10305ba2e54c));

  /* 
  * @dev returns a tree of 0 value nodes with num_tree_elts = 0
  */ 
  function newTree16() internal pure returns (bytes32) {
    return keccak256(abi.encodePacked(ZERO_HASH, ROOT_16));
  }

  function validTree16(Tree16 memory _t) internal pure returns (bool) {
    bytes32 hashNE = keccak256(abi.encodePacked(_t.numElts));
    return 
      _t.eltsRootHash == keccak256(abi.encodePacked(hashNE, _t.root))
        &&
      _t.numElts <= UINT16_MAX + 1;
  }
  
  /*
  * @dev Returns the new Tree hash from inserting elt
  * @param _t A 16-bit Merkle tree struct
  * @param _pathNodes RLP List of Merkle nodes. Omit bitpath.
  * @param _elt Hashed element to insert at index
  */
  function insertElt(Tree16 memory _t, bytes memory _proof, bytes32 _elt)
  internal pure returns (bytes32) {
    require(validTree16(_t), "MLIST_INVALID_numElts");
    uint16 index = _t.numElts;
    require(index == Merkle.getPath(_proof), "MLIST_INVALID_PROOF_PATH");
    require(_t.root == Merkle.calcRoot(ZERO_HASH, _proof, 16),
      "MLIST_INSERT_INVALID_PROOF");

    return Merkle.calcRoot(_elt, _proof, 16);
  }

  /*
  * @dev returns the element at index
  */
  function atIndex(
    Tree16 memory _t, bytes memory _proof, uint16 _index, bytes32 _elt
  ) internal pure returns (bool) {
    require(validTree16(_t), "MLIST_INVALID_numElts");
    require(_index == Merkle.getPath(_proof), "MLIST_INVALID_PROOF_PATH");

    return (_t.root == Merkle.calcRoot(_elt, _proof, 16));
  }

}
