pragma solidity ^0.8.9;

library Types {
  // IMPORTANT: first element in deposits is always 0
  // The first element is always skipped
  // Safe because no batch commits to having processed this deposit
  // 
  struct Town {
    uint256 id;
    address sequencer;
    bytes32 depositRoot;
    bytes32 batchRoot;
    uint256 depositLockBlock;     // if 0, deposits are not locked and can be made
    uint256 minFeeEth;
    uint256 maxFeeEth;
  }

  // prevDepositRoot is the root of the deposit tree before this deposit
  //  it acts as a nonce
  struct Deposit {
    uint256 town;
    address tokenContract;   // ERC20 or ERC721 contract, 0x1 for ETH
    uint256 tokenId;         // 0 unless it's an NFT
    address uqbarDest;
    uint256 amount;
    uint256 blockNumber;
    bytes32 prevDepositRoot;
  }

  // prevBatchRoot is the root of the deposit tree before this deposit
  //  it acts as a nonce
  struct Batch {
    uint256 town;
    address sequencer;
    bytes32 txRoot;
    bytes32 stateRoot;
    bytes32 townRoot;
    bytes32 endDepositRoot;
    uint256 blockNumber;
    bytes32 prevBatchRoot;
  }

  event DepositMade(uint256 town, address tokenContract, uint256 tokenId, 
    address uqbarDest, uint256 amount, uint256 blockNumber, bytes32 prevDepositRoot
  );

  event PostBatch(uint256 town, address sequencer, 
    bytes32 txRoot, bytes32 stateRoot, bytes32 townRoot,
    bytes32 endDepositRoot, uint256 blockNumber, bytes32 prevBatchRoot
  );
}
