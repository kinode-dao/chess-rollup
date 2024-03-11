pragma solidity ^0.8.9;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ExampleERC721 is ERC721, Ownable {
  uint256 private _tokenIdCounter;

  constructor(
    string memory name,
    string memory symbol
  ) ERC721(name, symbol) {
    _tokenIdCounter = 1;
  }

  function mintToken(address to) public onlyOwner {
    _safeMint(to, _tokenIdCounter);
    _tokenIdCounter = _tokenIdCounter + 1;
  }
}

