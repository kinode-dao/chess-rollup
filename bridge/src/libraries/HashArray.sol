pragma solidity ^0.8.9;

library HashArray {
  /*
   * @param start: the hash to start from
   * @param path:  hashes to apply to the start hash, bottom to top
   */
  struct HashArrayLookup {
    bytes32 start;
    bytes32[] path;
  }

  function appendElement(bytes32 _element, bytes32 _root) internal pure returns (bytes32) {
    return keccak256(abi.encode(_element, _root));
  }

  function validElement(
    HashArrayLookup memory _lookup, bytes32 _element, bytes32 _root
    ) internal pure returns (bool) {
    bytes32 hash = appendElement(_element, _lookup.start);
    for (uint256 i = 0; i < _lookup.path.length; i++) {
      hash = appendElement(_lookup.path[i], hash);
    }
    return _root == hash;
  } 
}
