// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/InvoicePayment.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockERC20 is ERC20 {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {
        _mint(msg.sender, 1000000 * 10**18); // Mint 1M tokens
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract InvoicePaymentTest is Test {
    InvoicePayment public invoicePayment;
    MockERC20 public token;
    
    address public owner = address(0x1);
    address public payer = address(0x2);
    address public payee = address(0x3);
    
    bytes32 public constant QUOTE_ID_HASH = keccak256("test-quote-id");
    uint64 public constant EXPIRY = 1234567890;
    uint256 public constant AMOUNT = 1000 * 10**18; // 1000 tokens

    function setUp() public {
        vm.startPrank(owner);
        invoicePayment = new InvoicePayment();
        token = new MockERC20("Test Token", "TEST");
        vm.stopPrank();

        // Setup payer with tokens and approvals
        token.mint(payer, AMOUNT * 10);
        vm.prank(payer);
        token.approve(address(invoicePayment), type(uint256).max);
    }

    function testPayInvoice() public {
        vm.warp(EXPIRY - 1000); // Set timestamp before expiry
        
        uint256 payerBalanceBefore = token.balanceOf(payer);
        uint256 payeeBalanceBefore = token.balanceOf(payee);

        // Compute expected invoice ID
        bytes32 expectedInvoiceId = keccak256(abi.encodePacked(QUOTE_ID_HASH, EXPIRY, uint256(1)));

        // Expect the Remittance event
        vm.expectEmit(true, true, true, true);
        emit InvoicePayment.Remittance(address(token), payee, expectedInvoiceId, payer, AMOUNT);

        vm.prank(payer);
        invoicePayment.payInvoice(QUOTE_ID_HASH, EXPIRY, address(token), AMOUNT, payee);

        // Check balances
        assertEq(token.balanceOf(payer), payerBalanceBefore - AMOUNT);
        assertEq(token.balanceOf(payee), payeeBalanceBefore + AMOUNT);
    }

    function testPayInvoiceETH() public {
        vm.warp(EXPIRY - 1000); // Set timestamp before expiry
        
        uint256 payerBalanceBefore = payer.balance;
        uint256 payeeBalanceBefore = payee.balance;
        uint256 ethAmount = 1 ether;

        // Give payer some ETH
        vm.deal(payer, ethAmount * 2);

        // Compute expected invoice ID
        bytes32 expectedInvoiceId = keccak256(abi.encodePacked(QUOTE_ID_HASH, EXPIRY, uint256(1)));

        // Expect the Remittance event
        vm.expectEmit(true, true, true, true);
        emit InvoicePayment.Remittance(address(0), payee, expectedInvoiceId, payer, ethAmount);

        vm.prank(payer);
        invoicePayment.payInvoiceETH{value: ethAmount}(QUOTE_ID_HASH, EXPIRY, payable(payee));

        // Check balances
        assertEq(payee.balance, payeeBalanceBefore + ethAmount);
    }

    function testPayInvoiceExpired() public {
        vm.warp(EXPIRY + 1); // Set timestamp after expiry

        vm.expectRevert("Invoice expired");
        vm.prank(payer);
        invoicePayment.payInvoice(QUOTE_ID_HASH, EXPIRY, address(token), AMOUNT, payee);
    }

    function testPayInvoiceInvalidPayee() public {
        vm.warp(EXPIRY - 1000);

        vm.expectRevert("Invalid payee address");
        vm.prank(payer);
        invoicePayment.payInvoice(QUOTE_ID_HASH, EXPIRY, address(token), AMOUNT, address(0));
    }

    function testPayInvoiceZeroAmount() public {
        vm.warp(EXPIRY - 1000);

        vm.expectRevert("Amount must be greater than zero");
        vm.prank(payer);
        invoicePayment.payInvoice(QUOTE_ID_HASH, EXPIRY, address(token), 0, payee);
    }

    function testBatchPayInvoices() public {
        vm.warp(EXPIRY - 1000);

        // Create batch payment data
        InvoicePayment.PaymentData[] memory payments = new InvoicePayment.PaymentData[](2);
        payments[0] = InvoicePayment.PaymentData({
            quoteIdHash: QUOTE_ID_HASH,
            expiry: EXPIRY,
            asset: address(token),
            amount: AMOUNT,
            payee: payee
        });
        payments[1] = InvoicePayment.PaymentData({
            quoteIdHash: keccak256("second-quote"),
            expiry: EXPIRY,
            asset: address(token),
            amount: AMOUNT / 2,
            payee: payee
        });

        uint256 totalAmount = AMOUNT + AMOUNT / 2;
        uint256 payerBalanceBefore = token.balanceOf(payer);
        uint256 payeeBalanceBefore = token.balanceOf(payee);

        vm.prank(payer);
        invoicePayment.batchPayInvoices(payments);

        // Check balances
        assertEq(token.balanceOf(payer), payerBalanceBefore - totalAmount);
        assertEq(token.balanceOf(payee), payeeBalanceBefore + totalAmount);
    }

    function testComputeInvoiceId() public {
        bytes32 invoiceId = invoicePayment.computeInvoiceId(QUOTE_ID_HASH, EXPIRY);
        bytes32 expectedInvoiceId = keccak256(abi.encodePacked(QUOTE_ID_HASH, EXPIRY, uint256(1)));
        assertEq(invoiceId, expectedInvoiceId);
    }

    function testEmergencyRecoverToken() public {
        // Send some tokens to the contract
        token.mint(address(invoicePayment), AMOUNT);
        
        uint256 ownerBalanceBefore = token.balanceOf(owner);

        vm.prank(owner);
        invoicePayment.emergencyRecoverToken(address(token), AMOUNT);

        assertEq(token.balanceOf(owner), ownerBalanceBefore + AMOUNT);
        assertEq(token.balanceOf(address(invoicePayment)), 0);
    }

    function testEmergencyRecoverETH() public {
        // Send some ETH to the contract
        vm.deal(address(invoicePayment), 1 ether);
        
        uint256 ownerBalanceBefore = owner.balance;

        vm.prank(owner);
        invoicePayment.emergencyRecoverETH();

        assertEq(owner.balance, ownerBalanceBefore + 1 ether);
        assertEq(address(invoicePayment).balance, 0);
    }

    function testOnlyOwnerCanRecoverTokens() public {
        vm.expectRevert("Ownable: caller is not the owner");
        vm.prank(payer);
        invoicePayment.emergencyRecoverToken(address(token), AMOUNT);
    }

    function testOnlyOwnerCanRecoverETH() public {
        vm.expectRevert("Ownable: caller is not the owner");
        vm.prank(payer);
        invoicePayment.emergencyRecoverETH();
    }
}
