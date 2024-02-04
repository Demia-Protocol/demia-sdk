# Demia SDK

The Demia SDK is a collection of Rust libraries aimed at providing functionality related to identity management, IOTA integration, secure storage (using Stronghold), and data streaming.

## Table of Contents

- [Introduction](#introduction)
- [Submodules](#submodules)
- [Getting Started](#getting-started)
- [Contributing](#contributing)
- [License](#license)

## Introduction

Demia SDK is designed to streamline the development process by providing modular components for identity management, IOTA interactions, secure storage, and data streaming. These components are organized as separate submodules within the repository.

## Submodules

### 1. Identity.rs

The `identity.rs` submodule focuses on identity management features. It includes functionalities related to user authentication, authorization, and decentralized identity systems.

### 2. Iota.rs

The `iota.rs` submodule provides integration with the IOTA protocol. It enables developers to interact with the IOTA Tangle, facilitating seamless communication and transactions on the IOTA network.

### 3. Stronghold.rs

The `stronghold.rs` submodule offers secure storage capabilities through the use of the Stronghold library. It provides a robust and encrypted storage solution for sensitive data.

### 4. Streams

The `streams` submodule facilitates data streaming functionalities. It enables developers to implement secure and efficient data streaming solutions in their applications.

## Getting Started

To get started with the Demia SDK and its submodules, follow these general steps:

1. Clone the Demia-sdk repository:

   ```bash
   git clone https://github.com/demia-protocol/Demia-sdk.git
   git submodule update --init --recursive

2. Import the SDK into your rust code
   ```rust
   use demia_sdk::iota_client::IotaClient;
   use demia_sdk::streams::lets::Streams;
   use demia_sdk::identity::did::DID;
   use demia_sdk::identity::iota::IOTAIdentity;
   use demia_sdk::iota_stronghold::Stronghold;

## Examples

You can see examples using the library in the [examples](examples/) directory. Try them with:

```shell
# cargo run --example <name of the example without .rs>
cargo run --example main
```

For examples where a seed is required you need to create a `.env` file under the current directory. You can do so by renaming [`.env.example`](.env.example) to `.env`.

## Contributing
We welcome contributions from the community. If you would like to contribute to the Demia SDK, please follow our contribution guidelines.

## License
This project is licensed under the MIT License, making it open for collaboration and use in various applications.
