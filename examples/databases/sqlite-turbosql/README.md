Showcasing probably the easiest library that allows using local sqlite instance with just a couple of lines of code - [turbosql](https://github.com/trevyn/turbosql). Ideaologically similar to the GlueSQL integration and the `Table` macro of prest. All you need to get started is to derive `Turbosql` on the struct that you want to use as a table and make sure that the struct's types are compatible (all columns are optional, first goes the rowid with i64 and then others):

{Cargo.toml}

{src/main.rs}