#!/usr/bin/env node
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ansicolors_1 = __importDefault(require("ansicolors"));
const cardinal = __importStar(require("cardinal"));
const fs = __importStar(require("fs-extra"));
const path = __importStar(require("path"));
const yargs = __importStar(require("yargs"));
const argv = yargs
    .option('input', {
    alias: 'i',
    demandOption: true,
    describe: 'Input ABI file',
    type: 'string',
})
    .option('output', {
    alias: 'o',
    demandOption: true,
    describe: 'Output TypeScript file',
    type: 'string',
})
    .help()
    .alias('help', 'h')
    .parseSync();
function usage(module) {
    return `import { Contract, RpcProvider, constants } from 'starknet';
import { ABI } from './${module}';

async function main() {
    const address = "CONTRACT_ADDRESS_HERE";
    const provider = new RpcProvider({ nodeUrl: constants.NetworkName.SN_MAIN });
    const contract = new Contract(ABI, address, provider).typedv2(ABI);

    const version = await contract.getVersion();
    console.log("version", version)

    // Abiwan is now successfully installed, just start writing your contract
    // function calls (\`const ret  = contract.your_function()\`) and you'll get
    // helpful editor autocompletion, linting errors ... for free ! Enjoy !
}
main().catch(console.error)`;
}
async function run() {
    const json = await fs.readJson(argv.input);
    let abi = json.abi;
    if (typeof abi === 'string') {
        abi = JSON.parse(abi);
    }
    const content = `export const ABI = ${JSON.stringify(abi, null, 2)} as const;\n`;
    await fs.writeFile(argv.output, content);
    const output_path = path.parse(argv.output);
    const usage_snippet = usage(output_path.name);
    const usage_snippet_highlighted = cardinal.highlight(usage_snippet);
    console.log(`✅ Successfully generated ${ansicolors_1.default.red(argv.output)}`);
    console.log(`💡 Here's a code snippet to get you started:\n`);
    console.log(usage_snippet_highlighted);
}
run().catch(console.error);
//# sourceMappingURL=generate.js.map