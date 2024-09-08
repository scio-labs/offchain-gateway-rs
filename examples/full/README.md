# Offchain Gateway (Rust)

> [!WARNING]
> This repository is under construction 🚧. We are actively improving it hackathon-style.

This is a Rust implementation of the [CCIP gateway](https://alpha-docs.ens.domains/resolvers/ccip). It allows you to issue unlimited gasless subdomains for your name, as well as to create, manage, and moderate namespaces.

> [!NOTE]
> This gateway is built to be an **Opinionated ENS Subname Issuer**, if youre looking for something more generic, please checkout [ensdomains/offchain-resolver](https://github.com/ensdomains/offchain-resolver), and [ensdomains/offchain-resolver-example](https://github.com/ensdomains/offchain-resolver-example).

## Features

- CCIP Spec Compliant Gateway Endpoint
- `AZERO-ID` - reads from AZERO-ID registry.
- Modular authentication for EOA, Admin API, & more.
- View endpoint for profile data.

## Setup

### Run the gateway
1. Set appropriate TLDs and their target registry in `./supported-tlds.json` (file path settable)
2. Set the env variables, for example:
    ```
    export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
    export PROVIDER_URL="wss://ws.test.azero.dev"
    export SUPPORTED_TLD_PATH="./supported-tlds.json"
    ```
3. Execute `cargo run`

### Deploy a Resolver

Using a CCIP Gateway requires a resolver contract to be acting on behalf of the name. Although you could write your own contract, we recommend you deploy a proxy contract through [ccip.tools](https://ccip.tools/).

[![](.github/ccip-tools.png)](https://ccip.tools/)

(Source for the contracts can be found at [ensdomains/ccip-tools](https://github.com/ensdomains/ccip-tools/tree/master/contracts))

When asked for the `Gateway URL` supply your gateway's url (for eg: `https://gateway.example.com/{sender}.json`), and for the `Signers` field supply the address of your gateway (for eg: `[0x225f137127d9067788314bc7fcc1f36746a3c3B5]`), you can find this in the logs of your gateway.

> [!NOTE]
> There are gas costs associated with deploying a resolver, at the time of writing this (30 gwei), it costs ~0.004 ETH (8 USD) to deploy a resolver (see [txs](https://etherscan.io/tx/0x0c90da0a122f38125a8ad1f48ef23cf5f7d399846bd5369b664ff288a31f797c)).

### Set your Resolver

Finally you need to instruct the onchain registry to use your resolver. You can do this by visiting your name in the [ENS Manager App](https://ens.app/) and under the `More` tab, set the `Resolver` field to the address of your resolver.

## Fork this 🍴
Don't like the implementation? Fork this repo and make it your own!

You might also be interested in the [resolution logic](https://github.com/ensdomains/offchain-gateway-rs/blob/main/src/gateway/resolution.rs) and [database modularity](https://github.com/ensdomains/offchain-gateway-rs/blob/main/src/database/mod.rs).

## Integration

This gateway implementation is designed to be modular, light, and easy to integrate. It comes with the abstract Datastore idea, (by default implemented with postgres), authentication, simple resolution logic, and a self-service API.

This means that depending on your use case you can easily swap out the database, the resolution logic, and customize authentication.
