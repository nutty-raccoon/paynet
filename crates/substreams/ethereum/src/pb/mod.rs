// @generated
pub mod invoice_contract {
    // @@protoc_insertion_point(attribute:invoice_contract.v1)
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/invoice_contract.v1.rs"));
        // @@protoc_insertion_point(invoice_contract.v1)
    }
}

pub mod sf {
    pub mod ethereum {
        pub mod r#type {
            // @@protoc_insertion_point(attribute:sf.ethereum.type.v2)
            pub mod v2 {
                // Placeholder for Ethereum types - in a real implementation,
                // these would be generated from the Ethereum foundational substreams
                #[derive(Clone, PartialEq, ::prost::Message)]
                pub struct Logs {
                    #[prost(message, repeated, tag = "1")]
                    pub logs: ::prost::alloc::vec::Vec<Log>,
                }

                #[derive(Clone, PartialEq, ::prost::Message)]
                pub struct Log {
                    #[prost(bytes = "vec", tag = "1")]
                    pub address: ::prost::alloc::vec::Vec<u8>,
                    #[prost(bytes = "vec", repeated, tag = "2")]
                    pub topics: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
                    #[prost(bytes = "vec", tag = "3")]
                    pub data: ::prost::alloc::vec::Vec<u8>,
                    #[prost(uint32, tag = "4")]
                    pub index: u32,
                    #[prost(message, optional, tag = "5")]
                    pub receipt: ::core::option::Option<TransactionReceipt>,
                }

                #[derive(Clone, PartialEq, ::prost::Message)]
                pub struct TransactionReceipt {
                    #[prost(message, optional, tag = "1")]
                    pub transaction: ::core::option::Option<Transaction>,
                }

                #[derive(Clone, PartialEq, ::prost::Message)]
                pub struct Transaction {
                    #[prost(bytes = "vec", tag = "1")]
                    pub hash: ::prost::alloc::vec::Vec<u8>,
                }
                // @@protoc_insertion_point(sf.ethereum.type.v2)
            }
        }
    }
}
