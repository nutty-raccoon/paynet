# Ethereum Integration for PayNet

This document describes the Ethereum integration added to PayNet, implementing support for a second blockchain alongside the existing Starknet support.

## Overview

The Ethereum integration follows the same architectural patterns as Starknet, providing:

- **Smart Contracts**: Solidity contracts implementing ERC-7699-like invoice payment functionality
- **Liquidity Source**: Rust implementation of deposit and withdrawal interfaces for Ethereum
- **Substreams**: Event indexing for Ethereum transactions
- **Type System**: Ethereum-specific types, units, and constants
- **Configuration**: Support for Ethereum networks (mainnet, sepolia, devnet)

## Architecture

### Components Added

1. **Ethereum Types** (`crates/libs/ethereum/types/`)
   - Asset definitions (ETH, USDC, USDT)
   - Unit types (Gwei, MilliUsdc)
   - Chain ID management
   - Network constants

2. **Ethereum Liquidity Source** (`crates/libs/ethereum/liquidity-source/`)
   - Deposit interface implementation
   - Withdrawal interface implementation
   - Transaction processing
   - Mock implementations for testing

3. **Smart Contracts** (`contracts/ethereum/`)
   - `InvoicePayment.sol`: Main contract for invoice payments
   - Support for both ERC20 and ETH payments
   - Batch payment functionality
   - Rich event emission for indexing

4. **Substreams** (`crates/substreams/ethereum/`)
   - Event filtering and parsing
   - Remittance event extraction
   - Integration with Ethereum foundational substreams

5. **Integration Updates**
   - Updated `Method` enum to include `Ethereum`
   - Enhanced `LiquiditySources` to support multiple chains
   - Configuration support for Ethereum networks

## Smart Contract Features

The `InvoicePayment` contract provides:

### Core Functions
- `payInvoice()`: Pay with ERC20 tokens
- `payInvoiceETH()`: Pay with native ETH
- `batchPayInvoices()`: Process multiple payments in one transaction
- `computeInvoiceId()`: Calculate invoice ID for a quote

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

### Security Features
- Reentrancy protection
- Expiry validation
- Emergency recovery functions
- Batch size limits

## Usage Flow

### For ETH Payments
1. User requests mint quote from PayNet node
2. User calls `payInvoiceETH()` with:
   - Quote ID hash
   - Expiry timestamp
   - Payee address (PayNet node)
   - ETH amount as msg.value
3. Contract emits `Remittance` event
4. PayNet indexer processes event and marks quote as paid
5. User can mint tokens via PayNet API

### For ERC20 Payments
1. User approves ERC20 token spending to InvoicePayment contract
2. User calls `payInvoice()` with:
   - Quote ID hash
   - Expiry timestamp
   - Token contract address
   - Amount
   - Payee address
3. Contract transfers tokens and emits event
4. Same indexing and minting flow as ETH

## Configuration

### Ethereum Node Configuration
```toml
# ethereum-config.toml
chain_id = "ETH_SEPOLIA"
cashier_account_address = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
ethereum_rpc_node_url = "https://sepolia.infura.io/v3/YOUR_KEY"
ethereum_substreams_url = "https://ethereum-substreams.example.com"
```

### Environment Variables
```bash
ETHEREUM_CASHIER_PRIVATE_KEY=0x...  # Private key for cashier account
```

### Command Line Usage
```bash
# Run with Ethereum support
cargo run --features ethereum -- --ethereum-config ethereum-config.toml

# Run with both Starknet and Ethereum
cargo run --features starknet,ethereum -- \
  --config starknet-config.toml \
  --ethereum-config ethereum-config.toml
```

## Development Setup

### Prerequisites
- Rust toolchain
- Foundry (for Ethereum contracts)
- Docker (for local devnet)

### Local Development
```bash
# Start Ethereum devnet
docker-compose -f docker-compose.ethereum.yml up -d

# Deploy contracts
cd contracts/ethereum
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

# Run tests
forge test

# Start PayNet node
cargo run --features ethereum -- --ethereum-config config/ethereum-docker.toml
```

## Testing

### Contract Tests
```bash
cd contracts/ethereum
forge test -vvv
```

### Integration Tests
The existing integration test framework has been extended to support Ethereum:
- Mock Ethereum liquidity source for unit tests
- Integration tests with local Ethereum devnet
- End-to-end payment flow testing

## Differences from Starknet Implementation

| Aspect | Starknet | Ethereum |
|--------|----------|----------|
| Hash Function | Poseidon | Keccak256 |
| Native Asset | ETH (via STRK token) | ETH (native) |
| Contract Language | Cairo | Solidity |
| Address Format | Felt252 | 20-byte hex |
| Transaction Model | Account abstraction | EOA + Contract |
| Event Indexing | Starknet events | Ethereum logs |

## Future Enhancements

1. **ERC-7699 Compliance**: Full implementation of the ERC-7699 standard
2. **Layer 2 Support**: Integration with Polygon, Arbitrum, Optimism
3. **Advanced Batching**: More sophisticated batch payment strategies
4. **Gas Optimization**: Further contract optimizations for gas efficiency
5. **Multi-sig Support**: Support for multi-signature cashier accounts

## Deployment Checklist

- [ ] Deploy InvoicePayment contract to target network
- [ ] Configure substreams indexer for the network
- [ ] Set up monitoring and alerting
- [ ] Test payment flows end-to-end
- [ ] Configure backup and recovery procedures
- [ ] Update documentation and API specs

## Security Considerations

1. **Private Key Management**: Secure storage of cashier private keys
2. **Contract Verification**: Verify contracts on Etherscan
3. **Rate Limiting**: Implement appropriate rate limits
4. **Monitoring**: Monitor for unusual transaction patterns
5. **Emergency Procedures**: Have emergency pause/recovery mechanisms

## Support

For issues related to Ethereum integration:
1. Check the contract tests and integration tests
2. Verify configuration files and environment variables
3. Check Ethereum node connectivity and sync status
4. Review substreams indexer logs
5. Consult the main PayNet documentation for general troubleshooting
