# Scrypto

[![CI](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml/badge.svg)](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml)

Language for building DeFi apps on Radix.

Documentation: https://docs.radixdlt.com/main/scrypto/introduction.html

## Installation

1. Install Rust
   * Windows:
       * Download and install [`rustup-init.exe`](https://win.rustup.rs/x86_64)
       * Install "Desktop development with C++" with [Build Tools for Visual Studio 2019](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)
       * Install [LLVM 13.0.1](https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.1/LLVM-13.0.1-win64.exe) (make sure you tick the option that adds LLVM to the system PATH)
   * Linux and macOS:
       * Make sure a C++ compiler and LLVM is installed (`sudo apt install build-essential llvm` with Ubuntu)
       * Install Rust compiler
       ```
       curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
       ```
2. Enable `cargo` in the current shell:
   * Windows:
       * Start a new PowerShell
   * Linux and macOS:
       ```
       source $HOME/.cargo/env
       ```
3. Add WebAssembly target
    ```
    rustup target add wasm32-unknown-unknown
    ```
4. Install simulator
    ```
    git clone https://github.com/radixdlt/radixdlt-scrypto.git
    cd radixdlt-scrypto
    cargo install --path ./simulator
    ```
5. (Optional) Open Scrypto documentation for later use
    ```
    ./doc.sh
    ```

## Getting Started

If you want a quick walkthrough of how to deploy and run some code, please see the [Run Your First Project](https://docs.radixdlt.com/main/scrypto/getting-started/run-first-project.html) tutorial. If you prefer to soldier through on your own, keep reading below.

### Writing Scrypto Code

1. Start by creating a new package:
```
scrypto new-package <package_name>
cd <package_name>
```
2. Check out the files under your current directory:
  - Source code is within `src/lib.rs`;
  - Test code is within `tests/lib.rs`.
3. Build your package:
```
scrypto build
```
4. Run tests:
```
scrypto test
```

### Playing with Radix Engine

| Action | Command |
|---|---|
| To create an account | ``` resim new-account ``` |
| To change the default account | ``` resim set-default-account <account_address> ``` |
| To create a resource with fixed supply | ``` resim new-resource-fixed <amount> ``` |
| To create a resource with mutable supply | ``` resim new-resource-mutable ``` |
| To mint resource | ``` resim mint <amount> <resource_def>``` |
| To transfer resource | ``` resim transfer <amount> <resource_def> <recipient_address> ``` |
| To publish a package | ``` resim publish <path_to_package_dir_or_wasm_file> ``` |
| To call a function | ``` resim call-function <package_address> <blueprint_name> <function> <args> ``` |
| To call a method | ``` resim call-method <component_address> <method> <args> ``` |
| To export the ABI of a blueprint | ``` resim export-abi <package_address> <blueprint_name> ``` |
| To show info about an address | ``` resim show <address> ``` |

**Note:** The commands use the default account as transaction sender.

## Project Layout

- `sbor`: The binary data format used by Scrypto.
- `sbor-derive`: Derives for encoding and decoding Rust `struct` and `enum`.
- `scrypto`: Scrypto standard library.
- `scrypto-abi`: Scrypto blueprint ABI.
- `scrypto-derive`: Derives for defining and importing Scrypto blueprints.
- `radix-engine`: The Scrypto execution engine.
- `simulator`: A simulator that run Scrypto code on a filesystem based ledger.
- `examples`: Scrypto examples.
