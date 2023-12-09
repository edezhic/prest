Minimalistic todo app powered by [GlueSQL](https://gluesql.org/docs/)-wrapped [sled](http://sled.rs/) storage showcasing their usage. One of the most controversial examples because I wouldn't recommend using them for almost any production project yet, but I think that both projects have a lot of potential. Let me elaborate:

**sled** is an embedded database (somewhat like [RocksDB](https://rocksdb.org/)) written in rust from scratch. This crate and it's close relative 
[Komora project](https://github.com/komora-io) are building next gen storage-layer solutions that utilize latest research in the field and focused on modern hardware. Reinventing a database might sound like a bit crazy idea, but: 

* such systems require fine-grained memory control and safety more than any other and rust shines in this space
* rust itself introduces almost no overhead so these tools can compete with mature C counterparts
* sled has already been in development for years, has reached **v1 beta** and can beat a lot of mature competitors on common workloads
* future improvements would be much easier to implement than in C codebases because borrow checker will always validate that another refactor or subsystem rewrite doesn't introduce memory bugs

According to it's discord server and discussions around the web there are already at least a couple of dozens of projects using sled. And I expect this number to grow dramatically once it will reach it's first stable release. But sled itself is only focused on being an efficient low-level storage layer that is expected to be used by higher-level libraries like 

**GlueSQL** - SQL query parser and execution layer that can be attached to wide variety of storage options. It's even younger than sled, but can already be used from rust, python and javascript(both node and browser!). Also, it already supports sled, redis, in-memory, json, csv, and browser local, session and indexedDB storages. You can also define your own storage interfaces and even create composite ones that allow different storages for different tables while still supporting JOIN queries across them.

The main benefit of gluesql is that it allows to work with different storages on both client and server side using the same interface. As of now this interface has some issues and does not have anything like an ORM, but it ships with a query builder and you can use good old SQL. 

Enough with the introduction, let's get to the code. Here's our manifest:

{Cargo.toml}

And here goes the todo app:

{serve.rs}