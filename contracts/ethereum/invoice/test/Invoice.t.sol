// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {InvoicePayment} from "../src/InvoicePayment.sol";
import {MockERC20} from "./mocks/MockERC20.sol";

contract InvoiceTest is Test {
    InvoicePayment public invoicePayment; // Instance of the contract under test
    MockERC20 public token; // Mock ERC20 token for simulating payments

    address public owner = address(0x1); // Owner address (not used in tests)
    address public sender = address(0x2); // Sender address (payer in tests)

    // Event expected to be emitted by InvoicePayment on payment
    event Remittance(
        address indexed asset, address indexed payee, bytes32 invoiceId, uint256 amount, address indexed payer
    );

    // Deploy fresh contract and token before each test
    function setUp() public {
        invoicePayment = new InvoicePayment();
        token = new MockERC20();
    }

    // Test that a successful invoice payment works as expected
    function test_itWorks() public {
        uint256 quoteId = 123235432454;
        bytes32 quoteIdHash = bytes32(quoteId);
        uint64 expiry = 5;
        uint256 amount = 100000;
        address recipient = address(0x3);

        // Mint tokens to sender for payment
        token.mint(sender, amount);

        // Simulate sender approving the contract to spend tokens
        vm.startPrank(sender);
        token.approve(address(invoicePayment), amount);

        // Compute expected invoice ID
        bytes32 invoiceId = keccak256(abi.encodePacked(quoteIdHash, expiry, uint256(2)));

        // Expect the Remittance event to be emitted with these parameters
        vm.expectEmit(true, true, true, true);
        emit Remittance(address(token), recipient, invoiceId, amount, sender);

        // Call the payInvoice function
        invoicePayment.payInvoice(quoteIdHash, expiry, address(token), amount, recipient);

        vm.stopPrank();

        // Assert that sender's balance is now zero and recipient received the amount
        assertEq(token.balanceOf(sender), 0);
        assertEq(token.balanceOf(recipient), amount);
    }

    // Test that payment reverts if the invoice has expired
    function test_RevertWhen_expiry_exceeded() public {
        uint256 quoteId = 123235432454;
        bytes32 quoteIdHash = bytes32(quoteId);
        uint64 expiry = 5;
        uint256 amount = 100000;
        address recipient = address(0x3);

        // Move block timestamp forward to simulate expiry
        uint256 future = block.timestamp + 1 days;
        vm.warp(future);

        // Mint tokens and approve as before
        token.mint(sender, amount);
        vm.startPrank(sender);
        token.approve(address(invoicePayment), amount);

        // Expect revert with "Invoice expired" message
        vm.expectRevert("Invoice expired");
        invoicePayment.payInvoice(quoteIdHash, expiry, address(token), amount, recipient);
        vm.stopPrank();
    }
}
