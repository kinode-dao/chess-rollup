pragma solidity ^0.8.9;

import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import { IERC20 } from  "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { IERC721 } from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

import { Types as T } from "../Types.sol";
import "../Constants.sol";
import "../libraries/Merkle.sol";
import { HashArray as HA } from "../libraries/HashArray.sol";

/*
Requirements when posting a batch:
- townRoot must be accessible in the stateRoot
- townRoot must be the townRoot of the previous batch
- txs must be the actual txs processed
- all deposits committed to must be in txs
- the townRoot state update must be in txs

- batch must be posted 1 block after locking. This lets us safely know deposits in the same block as a batch posting came after it, when looking up deposits
*/

contract Rollup is PausableUpgradeable, OwnableUpgradeable, ReentrancyGuardUpgradeable {
  modifier validTown(uint256 _town) {
    require(_town < towns.length, "INVALID_TOWN_INDEX");
    _;
  }

  modifier onlySequencer(uint256 _town) {
    require(msg.sender == towns[_town].sequencer, "ONLY_SEQUENCER");
    _;
  }
  
  /*
   * Variables
  */ 
  mapping(bytes32 => bool) private withdrawals;
  uint256 public rollbackWindowBlocks;
  uint256 public minBatchGapBlocks;
  uint256 public depositLimitWei;
  T.Town[] public towns;
  bool public whitelistEnabled;
  mapping(address => bool) public depositWhitelist;
  bytes32 townRoot;         //not currently used; forward compatibility

  //////////////////////////////////////////////////
  //////
  //////////////////////////////////////////////////
  
  function initialize
    (uint256 _depositLimitWei, address _sequencer)
    external initializer
  { 
    __Pausable_init();
    __Ownable_init();
    __ReentrancyGuard_init();
    depositLimitWei = _depositLimitWei;

    T.Town memory t;
    t.id = towns.length;
    t.sequencer = _sequencer;
    t.minFeeEth = DEFAULT_MIN_FEE;
    t.maxFeeEth = DEFAULT_MAX_FEE;
    towns.push(t);

    rollbackWindowBlocks = TWO_DAYS_BLOCKS;
    minBatchGapBlocks = 25;
  }

  function addTown(address _sequencer) external onlyOwner {
    T.Town memory t;
    t.id = towns.length;
    t.sequencer = _sequencer;
    t.minFeeEth = DEFAULT_MIN_FEE;
    t.maxFeeEth = DEFAULT_MAX_FEE;
    towns.push(t);
  }

  function _pushDeposit(uint256 _town, bytes32 _depositHash)
  internal validTown(_town) {
    towns[_town].depositRoot = HA.appendElement(_depositHash, towns[_town].depositRoot);
  }

  function _pushBatch(uint256 _town, bytes32 _batchHash)
  internal validTown(_town) {
    towns[_town].batchRoot = HA.appendElement(_batchHash, towns[_town].batchRoot);
  }

  function pause() external onlyOwner {
    _pause();
  }

  function unpause() external onlyOwner {
    _unpause();
  }

  function setWhitelistEnabled(bool _enabled) external onlyOwner {
    whitelistEnabled = _enabled;
  }

  function addToWhitelist(address _addr) external onlyOwner {
    depositWhitelist[_addr] = true;
  }

  function setSequencer(uint256 _town, address _sequencer)
  external onlyOwner validTown(_town) {
    towns[_town].sequencer = _sequencer;
  }

  function getSequencer(uint256 _town) public view validTown(_town) returns (address) {
    return towns[_town].sequencer;
  }

  function setMinFee(uint256 _town, uint256 _minFeeEth)
  external onlyOwner validTown(_town) {
    towns[_town].minFeeEth = _minFeeEth;
  }

  function getMinFee(uint256 _town) public view validTown(_town) returns (uint256) {
    return towns[_town].minFeeEth;
  }

  function setMaxFee(uint256 _town, uint256 _maxFeeEth)
  external onlyOwner validTown(_town) {
    towns[_town].maxFeeEth = _maxFeeEth;
  }

  function getMaxFee(uint256 _town) public view validTown(_town) returns (uint256) {
    return towns[_town].maxFeeEth;
  }

  function numTowns() public view returns (uint256) {
    return towns.length;
  }

  function setDepositLimit(uint256 _newLimitWei) external onlyOwner {
    require(_newLimitWei >= address(this).balance, "LIMIT_TOO_LOW");
    depositLimitWei = _newLimitWei;
  }

  function setRollbackWindowBlocks(uint64 _numBlocks) external onlyOwner {
    require(_numBlocks <= MAX_ROLLBACK_WINDOW, "ROLLBACK_WINDOW_TOO_LONG");
    rollbackWindowBlocks = _numBlocks;
  }

  function setMinBatchGapBlocks(uint256 _numBlocks) external onlyOwner {
    minBatchGapBlocks = _numBlocks;
  }

  function getDepositRoot(uint256 _town)
  public view validTown(_town) returns (bytes32) {
    return towns[_town].depositRoot;
  }

  function getBatchRoot(uint256 _town)
  public view validTown(_town) returns (bytes32) {
    return towns[_town].batchRoot;
  }

  function getDepositLockBlock(uint256 _town)
  public view validTown(_town) returns (uint256) {
    return towns[_town].depositLockBlock;
  }

  function isLocked(uint256 _town) public view validTown(_town) returns (bool) {
    return towns[_town].depositLockBlock != 0;
  }

  function lockDeposits(uint256 _town)
  external onlySequencer(_town) validTown(_town) {
    require(towns[_town].depositLockBlock == 0, "DEPOSITS_ALREADY_LOCKED");
    towns[_town].depositLockBlock = block.number;
  }

  function unlockDeposits(uint256 _town)
  external onlySequencer(_town) validTown(_town) {
    towns[_town].depositLockBlock = 0;
  }

  function unlockDepositsInternal(uint256 _town)
  internal {
    towns[_town].depositLockBlock = 0;
  }

  function _canDeposit(uint256 _town) internal view returns (bool) {
    return towns[_town].depositLockBlock == 0;
  }

  /* guarantees a one block gap after last deposit, which lets us know that deposits
     in the same block as a batch came after it
     */
  function canPostBatch(uint256 _town) internal view returns (bool) {
    return 
      towns[_town].depositLockBlock != 0
      &&
      block.number >= towns[_town].depositLockBlock + 1;
  }

  /* 
   * @dev Concatenate all batchRoots and hash that
  */
  function _calcTownRoot() internal view returns (bytes32) {
    require (towns.length > 0, "NO_TOWNS");
    bytes memory batchRoots;
    for(uint256 i = 0; i < towns.length; i++) {
      batchRoots = abi.encodePacked(
        batchRoots,
        towns[i].batchRoot
      );
    }
    return keccak256(batchRoots);
  }

  function hashDeposit(T.Deposit memory _d) public pure returns (bytes32) {
    return keccak256(abi.encode(
      _d.town, _d.tokenContract, _d.tokenId, _d.uqbarDest, _d.amount,
      _d.blockNumber, _d.prevDepositRoot 
    ));
  }

  function hashBatch(T.Batch memory _b) public pure returns (bytes32) {
    return keccak256(abi.encode(
      _b.town, _b.sequencer, _b.txRoot, _b.stateRoot, _b.townRoot,
      _b.endDepositRoot, _b.blockNumber, _b.prevBatchRoot                          
    ));
  }

  function depositEth(uint256 _town, address _uqbarDest) 
  external payable whenNotPaused validTown(_town) { 
    require(msg.value > 0, "NO_VALUE");
    require(_canDeposit(_town), "DEPOSITS_LOCKED");
    require(depositLimitWei >= address(this).balance, "OVER_DEPOSIT_LIMIT");

    bytes32 prevDepositRoot = getDepositRoot(_town);
    T.Deposit memory d = T.Deposit(_town, BRIDGE.TOKEN_CONTRACT_ETH, BRIDGE.NULL_TOKEN_ID,
                                   _uqbarDest, msg.value, block.number, prevDepositRoot);
    _pushDeposit(_town, hashDeposit(d));

    emit T.DepositMade(_town, BRIDGE.TOKEN_CONTRACT_ETH, BRIDGE.NULL_TOKEN_ID, _uqbarDest, 
                       msg.value, block.number, prevDepositRoot);
  }

  function depositERC20(uint256 _town, address _uqbarDest, 
                        address _tokenContract, uint256 amount
    ) external whenNotPaused validTown(_town) {
    require(amount > 0, "NO_VALUE");
    require(_canDeposit(_town), "DEPOSITS_LOCKED");

    IERC20(_tokenContract).transferFrom(msg.sender, address(this), amount);

    bytes32 prevDepositRoot = getDepositRoot(_town);
    T.Deposit memory d = T.Deposit(_town, _tokenContract, 0, _uqbarDest, amount,
                                   block.number, prevDepositRoot);
    _pushDeposit(_town, hashDeposit(d));

    emit T.DepositMade(_town, _tokenContract, BRIDGE.NULL_TOKEN_ID, _uqbarDest, amount,
                     block.number, prevDepositRoot);
  }

  function depositERC721(uint256 _town, address _uqbarDest, 
                         address _tokenContract, uint256 _tokenId
    ) external whenNotPaused validTown(_town) {
    require(_canDeposit(_town), "DEPOSITS_LOCKED");

    IERC721(_tokenContract).transferFrom(msg.sender, address(this), _tokenId);

    bytes32 prevDepositRoot = getDepositRoot(_town);
    T.Deposit memory d = T.Deposit(_town, _tokenContract, _tokenId, _uqbarDest, 
                           BRIDGE.NULL_AMOUNT, block.number, prevDepositRoot);
    _pushDeposit(_town, hashDeposit(d));

    emit T.DepositMade(_town, _tokenContract, _tokenId, _uqbarDest,
           BRIDGE.NULL_AMOUNT, block.number, prevDepositRoot);
  }
  ///////////////
  //
  ///////////////

  function _hashTxs(bytes memory _txs) internal pure returns (bytes32) {
    return keccak256(_txs); 
  }

  function _isLatestBatch(bytes32 _townBatchRoot, T.Batch memory _b)
  internal view returns (bool) {
    return _townBatchRoot == HA.appendElement(hashBatch(_b), _b.prevBatchRoot);
  }

  /* 
   * @dev batches must be posted minBatchGapBlocks apart
   * @param _endDepositRootCheck: what the endDepositRoot will be 
   *  allows clients posting to "checksum" their deposits and
   *  make tx fail if checksums don't match
   */
  function postBatch(
    uint256 _town, bytes calldata _txs, bytes32 _stateRoot,
    T.Batch memory _prevBatch, bytes32 _endDepositRootCheck
  ) external whenNotPaused validTown(_town) onlySequencer(_town) {
    require( canPostBatch(_town), "DEPOSITS_NOT_LOCKED" );

    T.Town memory t = towns[_town];
    bytes32 townBatchRoot = getBatchRoot(_town);

    if(townBatchRoot != 0) {
      require(_town == _prevBatch.town, "TOWN_MUST_MATCH");
      require(_isLatestBatch(townBatchRoot, _prevBatch), "PREV_BATCH_INVALID");
      require(_stateRoot != _prevBatch.stateRoot, "STATE_ROOT_MUST_BE_DIFFERENT");
      require(_prevBatch.blockNumber + minBatchGapBlocks <= block.number,
              "MIN_GAP_BLOCKS_FOR_BATCHES");
    }

    bytes32 txRoot = _hashTxs(_txs);
    bytes32 townRoot = _calcTownRoot();
    bytes32 endDepositRoot = getDepositRoot(_town);

    require(_endDepositRootCheck == endDepositRoot, "END_DEPOSIT_ROOT_MISMATCH");

    T.Batch memory b = T.Batch(
      _town, t.sequencer, txRoot, _stateRoot, townRoot,
      endDepositRoot, block.number, townBatchRoot
    );

    _pushBatch(_town, hashBatch(b));
    unlockDepositsInternal(_town);

    emit T.PostBatch(_town, t.sequencer, txRoot, _stateRoot, townRoot,
      endDepositRoot, block.number, townBatchRoot);
  }

  //////////////////////////////////////////////////
  // Withdrawals
  //////////////////////////////////////////////////

  function _hashWithdrawal(bytes memory _w) internal pure returns (bytes32) {
    // TODO
    return keccak256(_w);
  }

  /*
   * @dev Withdraw from a batch's stateRoot
   * @param _b Batch from which to withdrawal
   * @param _w Withdrawal to make
   * @param _proof Merkle proof of withdrawal
   * @param _depth Merkle proof depth
  */
  function withdrawal(T.Batch memory _b, bytes memory _withdrawal, 
                      bytes memory _proof, uint8 _depth)
    external whenNotPaused nonReentrant {
    bytes32 townBatchRoot = getBatchRoot(_b.town);
    require(_isLatestBatch(townBatchRoot, _b), "PREV_BATCH_INVALID");
    require(_b.blockNumber < block.number - rollbackWindowBlocks); 

    bytes32 wHash = _hashWithdrawal(_withdrawal);
    require(!withdrawals[wHash], "WITHDRAWAL_ALREADY_MADE");

    require(Merkle.checkProof(_b.stateRoot, wHash, _proof, _depth),
            "INVALID_PROOF");
    
    withdrawals[wHash] = true;

    // TODO: read RLP items here

    /*
    require(_b.town == _w.town, "TOWN_MUST_MATCH");
    // ETH
    if ( _w.tokenContract == BRIDGE.TOKEN_CONTRACT_ETH ) {
      (bool success, ) = _w.ethDest.call{value: _w.amount}("");
      require(success, "Failed to send Ether");
    }
    // ERC20
    else if ( _w.amount != 0 ) {
      IERC20(_w.tokenContract).transferFrom(address(this), _w.ethDest, _w.amount);
    }
    // ERC721
    else {
        IERC721(_w.tokenContract).transferFrom(address(this), _w.ethDest, _w.tokenId);
    }
    */
  }

  // TODO: rollback function
  //  prove batch is in batches (use HashArray)
  //  it must be inside the rollbackWindowBlocks
}

