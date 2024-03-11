pragma solidity ^0.8.9;

import { HashArray as HA } from "../libraries/HashArray.sol";

contract HashArrayTest {
  bool public res;

  function testLookup(
    HA.HashArrayLookup memory _lookup, bytes32 _element, bytes32 _root
    ) public {
    res = HA.validElement(_lookup, _element, _root);
  }
}
