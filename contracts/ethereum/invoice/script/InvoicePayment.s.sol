// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {InvoicePayment} from "../src/InvoicePayment.sol";

contract InvoicePaymentScript is Script {
    InvoicePayment public invoicePayment;

    // Fixed salt for deterministic deployment
    bytes32 constant SALT = keccak256("PayNet.InvoicePayment.v1");

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        // Deploy using CREATE2 for deterministic address
        invoicePayment = new InvoicePayment{salt: SALT}();

        // Log the deployed address
        console.log("InvoicePayment deployed at:", address(invoicePayment));

        vm.stopBroadcast();
    }
}
