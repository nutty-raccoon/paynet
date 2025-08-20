// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {WETH9} from "../src/WETH.sol";

contract WethScript is Script {
    WETH9 public weth;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        weth = new WETH9();

        vm.stopBroadcast();
    }
}
