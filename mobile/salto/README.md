# Salto

This directory contains our mobile app, Salto.

It is built with [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/) and [TypeScript](https://www.typescriptlang.org/) in [Vite](https://vite.dev).

## Build and run

I use `bun`, but you can choose any other JavaScript package manager, such as `pnpm`.

```shell
cd mobile/salto
bun install
```

For Desktop development, run:
```shell
bun tauri dev
```
Desktop development is handy for quickly iterating over the application front-end, however some features, such as QR code scanning and NFC, are only available on mobile.
For those you will prefer building the mobile version of the app. 

For Android development, run:
```shell
bun tauri android init
sh scripts/edit-android-gen-files.sh
bun tauri android dev
```

For iOS development, run:
```shell
bun tauri ios init
bun tauri ios dev
```

By default, this will prompt you to select a virtual device to be emulated on your machine.
I rather recommend you connect your own phone to your computer in order to have the app installed on it. This way you can get the full experience of the app.
Make sure both your phone and computer are connected to the same network as the front-end used by your phone app will be running as a server exposed on your computer (this is required for hot reloading).

## DevX

Front-end changes are cheap, and benefit from hot reloading, while any change to the app backend (the content of `src-tauri`), will require a full recompilation of the app, which is quite time consuming.
Therefore, it is okay to edit the front-end by incremental small touches, but I recommend that when developing on the backend, you do it by big chunks and only rebuild the app when you think it is ready.
Fortunately, Rust is a perfect language for building large pieces of code without constantly running them.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
