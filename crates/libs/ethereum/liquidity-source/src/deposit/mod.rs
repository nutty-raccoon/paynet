#[cfg(feature = "mock")]
mod mock;

#[cfg(feature = "mock")]
pub use mock::*;
#[cfg(not(feature = "mock"))]
pub use not_mock::*;

#[cfg(not(feature = "mock"))]
mod not_mock {
    use bitcoin_hashes::Sha256;
    use liquidity_source::DepositInterface;
    use nuts::Amount;
    use ethereum_types::{
        Asset, Unit, ChainId, EthereumAddress, constants::ON_CHAIN_CONSTANTS,
    };
    use primitive_types::{H256, U256};
    use uuid::Uuid;

    use crate::EthereumInvoiceId;

    #[derive(Debug, Clone)]
    pub struct Depositer {
        chain_id: ChainId,
        our_account_address: EthereumAddress,
    }

    impl Depositer {
        pub fn new(chain_id: ChainId, our_account_address: EthereumAddress) -> Self {
            Self {
                chain_id,
                our_account_address,
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("asset {0} not found in on-chain constants")]
        AssetNotFound(Asset),
        #[error("failed to serialize transaction data: {0}")]
        SerdeJson(#[from] serde_json::Error),
    }

    impl DepositInterface for Depositer {
        type Error = Error;
        type InvoiceId = EthereumInvoiceId;

        fn generate_deposit_payload(
            &self,
            quote_id: Uuid,
            unit: Unit,
            amount: Amount,
            expiry: u64,
        ) -> Result<(Self::InvoiceId, String), Self::Error> {
            let asset = unit.asset();
            let amount_u256 = unit.convert_amount_into_u256(amount);
            let on_chain_constants = ON_CHAIN_CONSTANTS.get(self.chain_id.as_str()).unwrap();
            
            let token_contract_address = match asset {
                Asset::Eth => None, // ETH is native, no contract address
                _ => on_chain_constants
                    .assets_contract_address
                    .get_contract_address_for_asset(asset)
                    .ok_or(Error::AssetNotFound(asset))?,
            };

            let quote_id_hash = H256::from_slice(
                Sha256::hash(quote_id.as_bytes()).as_byte_array()
            );

            // Generate the invoice payment transaction data
            let transaction_data = if asset == Asset::Eth {
                // For ETH, create a direct transfer to the invoice contract
                generate_eth_payment_transaction(
                    on_chain_constants.invoice_payment_contract_address,
                    quote_id_hash,
                    expiry,
                    amount_u256,
                    self.our_account_address,
                )
            } else {
                // For ERC20 tokens, create approve + transferFrom calls
                generate_erc20_payment_transaction(
                    on_chain_constants.invoice_payment_contract_address,
                    quote_id_hash,
                    expiry,
                    token_contract_address.unwrap(),
                    amount_u256,
                    self.our_account_address,
                )
            };

            let invoice_id = EthereumInvoiceId(quote_id_hash);
            let payload = serde_json::to_string(&transaction_data)?;

            Ok((invoice_id, payload))
        }
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct EthereumTransactionData {
        pub to: EthereumAddress,
        pub value: U256,
        pub data: Vec<u8>,
        pub gas_limit: U256,
    }

    fn generate_eth_payment_transaction(
        invoice_contract: EthereumAddress,
        quote_id_hash: H256,
        expiry: u64,
        amount: U256,
        payee: EthereumAddress,
    ) -> EthereumTransactionData {
        // Encode the function call data for the invoice payment
        // This would typically use ethers-rs ABI encoding
        let function_selector = [0x12, 0x34, 0x56, 0x78]; // Placeholder for actual function selector
        let mut data = Vec::new();
        data.extend_from_slice(&function_selector);
        data.extend_from_slice(quote_id_hash.as_bytes());
        data.extend_from_slice(&expiry.to_be_bytes());
        data.extend_from_slice(payee.as_bytes());

        EthereumTransactionData {
            to: invoice_contract,
            value: amount,
            data,
            gas_limit: U256::from(100_000), // Estimated gas limit
        }
    }

    fn generate_erc20_payment_transaction(
        invoice_contract: EthereumAddress,
        quote_id_hash: H256,
        expiry: u64,
        token_contract: EthereumAddress,
        amount: U256,
        payee: EthereumAddress,
    ) -> EthereumTransactionData {
        // For ERC20, we need to call the invoice contract which will do transferFrom
        let function_selector = [0x87, 0x65, 0x43, 0x21]; // Placeholder for actual function selector
        let mut data = Vec::new();
        data.extend_from_slice(&function_selector);
        data.extend_from_slice(token_contract.as_bytes());
        data.extend_from_slice(quote_id_hash.as_bytes());
        data.extend_from_slice(&expiry.to_be_bytes());
        data.extend_from_slice(payee.as_bytes());
        
        // Encode amount as 32 bytes
        let mut amount_bytes = [0u8; 32];
        amount.to_big_endian(&mut amount_bytes);
        data.extend_from_slice(&amount_bytes);

        EthereumTransactionData {
            to: invoice_contract,
            value: U256::zero(), // No ETH value for ERC20 transfers
            data,
            gas_limit: U256::from(150_000), // Higher gas limit for ERC20
        }
    }
}
