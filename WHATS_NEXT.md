This is a hobby project and plans change on the fly, but there are things I'd likely work on or consider next:

+ For admin metrics - send upds over SSE, optimize perf & looks. Use [Chart.js](https://www.chartjs.org/) or smth alike

+ logs + metrics into db with 0x0(shared/localhost) owner?

+ DB and rest spawned in different processes and DB restarted only on migrations? How long DB restart takes with gigs of data?

+ indexes in dbs

+ how distributed scan might work? Need to keep all owners in the contact book which shared access? Mapping from any storage field type to CryptoAddress. Needs to implement smthlike `IntoOwnerAddress` like `IntoSqlKey`?
+ each `User` must have some kind of address? Roles/permissions for send/sync per table/row based on address/id/pk?
+ solana's programmatic delegation to programmatic addresses is kinda nice

+ host contact book = mapping CryptoAddress(or alias) -> NetworkAddress. Incoming requests can be from any network address as long as signature matches address. Should allow P2P etc, so no hard dependency on TLS. Separate current_thread rt sending/receiving blocks of txs and queries around?

+ `Send` and `Sync` have similar semantics but with threads instead of hosts. Attributes `web(Send)`(single owner) and `web(Sync)`(distributed) to derive required stuff. Sendable monitoring tables for remote debugging
+ `web(Send)` and `web(Sync)`, `owner` : owner is optional struct/field attr to send tx (and queries) to another host (otherwise 0x0/localhost), and owner is required field attr to sync txs with other hosts. 

+ `owner` attribute that defines sharding?
+ single ownership(fast path): only owner can write into **^^^** and must sign to send, 0x0 (localhost/root) owner by default, but host can have a keypair(lock?) to write data? => tables can be split between owners into shards, owners can share read and/or delegate write access to other owners for replication and other stuff
+ distributed ownership(slow path): no wide blockchain standard for address - use which or make custom? Any indexed value is ok? Has to be relevant to signatures etc? BIP44 and Hierarchical Deterministic (HD) wallets seem relevant for management of this stuff. Use most of the key derivation scheme but custom paths like `/{table_name}/{index_key}`?


+ add runtimes(main and db) stats and db/sled storage_stats to system stats ([tokio](https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html)). Improve current reporting (RAM usage in mbs, ...) - add hover with mbs info.

+ finish custom storage impl: indexes, add/drop column methods & validations & automigrations
+ auth upgrades - simpler UX + DX, support more providers/methods
+ subdomains and multiple-services on single machine support
+ [rust-i18n](https://github.com/longbridgeapp/rust-i18n) or another i18n solution
+ `axum-valid`-like integration for `Vals` or smth alike

Some ideas are more complex/crazy but interesting:
+ prest blogging about its own system monitoring stats etc using small llm
+ example with a built-in minimalistic polkadot chain - customizable + optionally distributed + optionally public DB
+ web3 tooling for the frontend, either with the above polkadot idea or for solana, with as little JS as possible
+ GlueSQL-based persistent DB in the SW that syncs with the host (meteor-like)

There are also longer term things which will be needed or nice to have before the stable release of prest:
* stabilization of async iterator and other basic concurrent std apis
* stable releases of most important dependencies like axum and sled 
* parallel frontend and cranelift backend of the rust compiler for faster builds
* more optional configs all around for flexibility
* find a way to re-export wasm-bindgen into the prest to avoid need for other deps 
* better Service Worker DX in Rust
* wider range of examples: interactive UIs, mobile builds, webgpu-based LLMs, ...?
