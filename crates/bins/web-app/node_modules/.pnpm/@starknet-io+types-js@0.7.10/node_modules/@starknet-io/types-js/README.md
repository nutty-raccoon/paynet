# Starknet types JS/TS

🐺 Starknet TypeScript types 🚀

[![GitHub Workflow Status](https://github.com/starknet-io/types-js/actions/workflows/publish.yml/badge.svg)](https://github.com/starknet-io/types-js/actions/workflows/publish.yml)
[![Project license](https://img.shields.io/github/license/starknet-io/types-js.svg?style=flat-square)](LICENSE)
[![Pull Requests welcome](https://img.shields.io/badge/PRs-welcome-ff69b4.svg?style=flat-square)](https://github.com/starknet-io/types-js/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)

Shared TypeScript type definitions for Starknet projects

## Installation

```bash
npm i @starknet-io/types-js
```

## Types

#### API [Starknet JSON RPC Specification](https://github.com/starkware-libs/starknet-specs/tree/master/api)

```ts
// type import
import type { SomeApiType } from '@starknet-io/types-js';
// or entire namespace import
import { API } from '@starknet-io/types-js';
```

#### Wallet API [Wallet JSON RPC Specification](https://github.com/starkware-libs/starknet-specs/tree/48e77bf4aaf687388b40b8198e3105401941517a/wallet-api)

```ts
// type import
import type { SomeWalletApiType } from '@starknet-io/types-js';
// or entire namespace import
import { WALLET_API } from '@starknet-io/types-js';
```

##### SNIP-12 [Hashing and signing typed structured data](https://github.com/starknet-io/SNIPs/blob/main/SNIPS/snip-12.md)

- /src/wallet-api/typedData.ts

## Version

(MAJOR.MINOR) Version of this package follows [starknet-spec](https://github.com/starkware-libs/starknet-specs) semantic versioning. PATCH version can diverge based on bug-fixes
`Starknet types v0.7.x <-> Starknet Spec v0.7.x`

## Devs Build

```bash
npm run build
```

## Devs Testing

```bash
npm run lint
```

## License

This repository is licensed under the MIT License, see [LICENSE](LICENSE) for more information.
