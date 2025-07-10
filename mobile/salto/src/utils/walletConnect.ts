
// Define metadata globally



// Define types for Starknet responses
interface StarknetTransactionResponse {
  transaction_hash: string;
}

// Example ETH transfer transaction
const createEthTransferTransaction = (accountAddress: string) => ({
  accountAddress,
  executionRequest: {
    calls: [
      {
        contractAddress:
          '0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7', // ETH contract address on Starknet
        entrypoint: 'transfer',
        calldata: [
          accountAddress, // recipient (sending to self)
          '0x0000000000000000000000000000000000000000000000000000000000000001', // amount 1 wei
          '0x0',
        ],
      },
    ],
  },
});

const newCalldata = [
  {
    to: '0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d',
    selector:
      '0x219209e083275171774dab1df80982e9df2096516f06319c5c6d71ae0a8480c',
    calldata: [
      '0x44aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46',
      '0x56bc75e2d63100000',
      '0x0',
    ],
  },
  {
    to: '0x44aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46',
    selector: '0xd5c0f26335ab142eb700850eded4619418b0f6e98c5b92a6347b68d2f2a0c',
    calldata: [
      '0x2d09ca739a6d3a5bed6ae8a3190db0966d57f4c4fff34e19738990596879904',
      '0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d',
      '0x56bc75e2d63100000',
      '0x0',
      '0x2a4c56a99f93d0b19f9a3b09640cb9fd1f4c426474a85dedfec573849ab6235',
    ],
  },
];
