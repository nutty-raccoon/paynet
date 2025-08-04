// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {InvoicePayment} from "../src/InvoicePayment.sol";

contract InvoicePaymentScript is Script {
    InvoicePayment public invoicePayment;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        invoicePayment = new InvoicePayment();

        vm.stopBroadcast();
    }
}
