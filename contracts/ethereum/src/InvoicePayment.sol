// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title InvoicePayment
 * @dev An ERC20 transfer contract with richer events, implementing ERC-7699 standard
 * 
 * The sole purpose of this contract is to provide the ability to pass a transfer reference
 * in a way similar to EIP-7699 (https://github.com/ethereum/ERCs/blob/master/ERCS/erc-7699.md).
 * 
 * We use it during the mint process:
 * 1. The user requires a mint quote from the node, it comes with a UUID.
 * 2. The user deposits to the node address using this Invoice contract,
 *    providing a bytes32 built from the hash of this UUID as `invoiceId`
 * 3. The node listens to on-chain deposits to its address, and uses the `invoiceId` to flag the correct quote as `PAID`
 * 4. The user calls the node's `mint` route with the original UUID and receives the corresponding amount of tokens
 */
contract InvoicePayment is ReentrancyGuard, Ownable {
    
    /**
     * @dev A deposit was made for `invoiceId`
     * @param asset The ERC20 token contract address (address(0) for ETH)
     * @param payee The recipient of the payment
     * @param invoiceId The computed invoice identifier
     * @param payer The address that made the payment
     * @param amount The amount transferred
     */
    event Remittance(
        address indexed asset,
        address indexed payee,
        bytes32 indexed invoiceId,
        address payer,
        uint256 amount
    );

    /**
     * @dev Execute an ERC20 transfer and emit the rich event
     * @param quoteIdHash The hash of the quote UUID
     * @param expiry The expiration timestamp for this invoice
     * @param asset The ERC20 token contract address
     * @param amount The amount to transfer
     * @param payee The recipient address
     */
    function payInvoice(
        bytes32 quoteIdHash,
        uint64 expiry,
        address asset,
        uint256 amount,
        address payee
    ) external nonReentrant {
        require(expiry >= block.timestamp, "Invoice expired");
        require(payee != address(0), "Invalid payee address");
        require(amount > 0, "Amount must be greater than zero");

        address payer = msg.sender;
        
        // Compute the invoice ID using keccak256 hash (similar to Starknet's Poseidon hash)
        bytes32 invoiceId = keccak256(abi.encodePacked(quoteIdHash, expiry, uint256(1))); // 1 for Ethereum chain

        // Transfer the tokens from payer to payee
        IERC20 token = IERC20(asset);
        require(
            token.transferFrom(payer, payee, amount),
            "Transfer failed"
        );

        emit Remittance(asset, payee, invoiceId, payer, amount);
    }

    /**
     * @dev Execute an ETH transfer and emit the rich event
     * @param quoteIdHash The hash of the quote UUID
     * @param expiry The expiration timestamp for this invoice
     * @param payee The recipient address
     */
    function payInvoiceETH(
        bytes32 quoteIdHash,
        uint64 expiry,
        address payable payee
    ) external payable nonReentrant {
        require(expiry >= block.timestamp, "Invoice expired");
        require(payee != address(0), "Invalid payee address");
        require(msg.value > 0, "Amount must be greater than zero");

        address payer = msg.sender;
        uint256 amount = msg.value;
        
        // Compute the invoice ID using keccak256 hash
        bytes32 invoiceId = keccak256(abi.encodePacked(quoteIdHash, expiry, uint256(1))); // 1 for Ethereum chain

        // Transfer ETH to payee
        (bool success, ) = payee.call{value: amount}("");
        require(success, "ETH transfer failed");

        emit Remittance(address(0), payee, invoiceId, payer, amount);
    }

    /**
     * @dev Batch payment function for multiple invoices
     * @param payments Array of payment data
     */
    struct PaymentData {
        bytes32 quoteIdHash;
        uint64 expiry;
        address asset;
        uint256 amount;
        address payee;
    }

    function batchPayInvoices(PaymentData[] calldata payments) external nonReentrant {
        require(payments.length > 0, "No payments provided");
        require(payments.length <= 50, "Too many payments"); // Limit to prevent gas issues

        for (uint256 i = 0; i < payments.length; i++) {
            PaymentData memory payment = payments[i];
            
            require(payment.expiry >= block.timestamp, "Invoice expired");
            require(payment.payee != address(0), "Invalid payee address");
            require(payment.amount > 0, "Amount must be greater than zero");

            address payer = msg.sender;
            
            // Compute the invoice ID
            bytes32 invoiceId = keccak256(abi.encodePacked(payment.quoteIdHash, payment.expiry, uint256(1)));

            // Transfer the tokens
            IERC20 token = IERC20(payment.asset);
            require(
                token.transferFrom(payer, payment.payee, payment.amount),
                "Transfer failed"
            );

            emit Remittance(payment.asset, payment.payee, invoiceId, payer, payment.amount);
        }
    }

    /**
     * @dev Emergency function to recover stuck tokens (only owner)
     * @param token The token contract address
     * @param amount The amount to recover
     */
    function emergencyRecoverToken(address token, uint256 amount) external onlyOwner {
        IERC20(token).transfer(owner(), amount);
    }

    /**
     * @dev Emergency function to recover stuck ETH (only owner)
     */
    function emergencyRecoverETH() external onlyOwner {
        payable(owner()).transfer(address(this).balance);
    }

    /**
     * @dev Compute invoice ID for a given quote
     * @param quoteIdHash The hash of the quote UUID
     * @param expiry The expiration timestamp
     * @return The computed invoice ID
     */
    function computeInvoiceId(bytes32 quoteIdHash, uint64 expiry) external pure returns (bytes32) {
        return keccak256(abi.encodePacked(quoteIdHash, expiry, uint256(1)));
    }
}
