# Ethereum Invoice Payment Contract

This directory contains the Ethereum smart contracts for the PayNet invoice payment system, implementing functionality similar to ERC-7699 for invoice references in transfers.

## Overview

The `InvoicePayment` contract provides the ability to execute ERC20 and ETH transfers with rich event data, allowing the PayNet system to track payments against specific invoice IDs.

## Contract Features

- **ERC20 Token Payments**: Support for any ERC20 token transfers with invoice tracking
- **ETH Payments**: Native ETH transfers with invoice tracking  
- **Batch Payments**: Process multiple invoice payments in a single transaction
- **Expiry Validation**: Invoices have expiration timestamps to prevent stale payments
- **Rich Events**: Detailed `Remittance` events for off-chain indexing
- **Emergency Recovery**: Owner can recover stuck tokens/ETH

## Usage Flow

1. User requests a mint quote from the PayNet node, receiving a UUID
2. User calls `payInvoice()` or `payInvoiceETH()` with:
   - `quoteIdHash`: Hash of the quote UUID
   - `expiry`: Expiration timestamp
   - `asset`: Token contract address (or address(0) for ETH)
   - `amount`: Amount to transfer
   - `payee`: Recipient address (usually the PayNet node)
3. Contract computes `invoiceId` and emits `Remittance` event
4. PayNet node indexes the event and marks the quote as PAID
5. User can then call the node's mint endpoint to receive tokens

## Development

### Prerequisites

- [Foundry](https://getfoundry.sh/)

### Setup

```bash
cd contracts/ethereum
forge install
```

### Testing

```bash
forge test
```

### Deployment

```bash
# Set environment variables
export PRIVATE_KEY=0x...
export RPC_URL=https://...

# Deploy to network
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify
```

## Contract Interface

### Main Functions

```solidity
function payInvoice(
    bytes32 quoteIdHash,
    uint64 expiry,
    address asset,
    uint256 amount,
    address payee
) external;

function payInvoiceETH(
    bytes32 quoteIdHash,
    uint64 expiry,
    address payable payee
) external payable;

function batchPayInvoices(PaymentData[] calldata payments) external;

function computeInvoiceId(bytes32 quoteIdHash, uint64 expiry) external pure returns (bytes32);
```

### Events

```solidity
event Remittance(
    address indexed asset,
    address indexed payee,
    bytes32 indexed invoiceId,
    address payer,
    uint256 amount
);
```

## Security Considerations

- Uses OpenZeppelin's `ReentrancyGuard` to prevent reentrancy attacks
- Validates expiry timestamps to prevent stale invoice payments
- Includes emergency recovery functions for stuck funds
- Limits batch payment size to prevent gas issues

## Integration with PayNet

The contract is designed to integrate with the PayNet substreams indexer, which will:

1. Monitor `Remittance` events on-chain
2. Extract `invoiceId` from events
3. Match against pending mint quotes in the database
4. Update quote status to enable token minting

## Differences from Starknet Version

- Uses `keccak256` instead of Poseidon hash for invoice ID computation
- Supports both ERC20 and native ETH payments
- Includes batch payment functionality
- Uses Solidity/EVM patterns instead of Cairo/Starknet patterns
