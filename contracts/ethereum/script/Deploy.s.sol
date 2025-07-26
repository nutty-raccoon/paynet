// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/InvoicePayment.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);
        
        InvoicePayment invoicePayment = new InvoicePayment();
        
        console.log("InvoicePayment deployed at:", address(invoicePayment));
        
        vm.stopBroadcast();
    }
}
