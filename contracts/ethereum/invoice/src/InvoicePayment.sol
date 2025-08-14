// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// Interface defining the pay_invoice function
interface IInvoicePayment {
    function payInvoice(bytes32 quoteIdHash, uint64 expiry, address asset, uint256 amount, address payee) external;
}

contract InvoicePayment is IInvoicePayment {
    // Event emitted when a payment is made for an invoice
    event Remittance(
        address indexed asset, address indexed payee, bytes32 invoiceId, uint256 amount, address indexed payer
    );

    // Pay an invoice by transferring ERC20 tokens from payer to payee
    function payInvoice(bytes32 quoteIdHash, uint64 expiry, address asset, uint256 amount, address payee) external {
        address payer = msg.sender;

        IERC20 token = IERC20(asset);

        require(expiry >= block.timestamp, "Invoice expired");

        // Generate a unique invoice ID using the quoteIdHash, expiry, and a constant
        bytes32 invoiceId = keccak256(abi.encodePacked(quoteIdHash, expiry, uint256(2)));

        require(token.transferFrom(payer, payee, amount), "Transfer failed");

        emit Remittance(asset, payee, invoiceId, amount, payer);
    }
}
