Example of a todo application that is using Solana blockchain for storage. One of the cool things about Solana is that it supports writing onchain programs in Rust so we can reuse program's types in the offchain prest code to simplify interactions. Also, there is an [Anchor](https://www.anchor-lang.com/) framework that simplifies smart contract development by abstracting onchain accounts so that we don't have to worry all technical details. To get started we'll need to add anchor dependencies and some patches to make it compatible with prest:

{Cargo.toml}

You might also notice profile overrides which are needed to make program's code as small as possible because onchain storage is quite expensive.

Next comes the program's code:

{src/lib.rs}

Here we have definitions for the onchain data and available instructions. Each instruction requires a context which defines accounts that are needed for the execution. Some of them like `Signer` and `System` are built-in, but `TodoList` is defined by this program.

Last piece is the application which will prepare and deploy the program to the local network and allow us to interact with it:

{src/main.rs}

This example is hardcoded for an easy local setup and demo purposes, but overall solana interactions aren't much different. However, in a real project you'll probably want to run transactions from the frontend signed by users' keys, and current solana sdks do not support doing that in rust, so you'll probably need to add some javascript.