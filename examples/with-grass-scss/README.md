This example contains [SCSS](https://sass-lang.com/) styles which are compiled using [grass](https://github.com/connorskees/grass). Grass was specifically designed with an extremely simple API and high performance, and prest has a couple of utils to set up the build pipeline and include the output artifacts. We'll use a very simple scss file just to showcase how you can include those into your prest app:

{styles.scss}

First we add a couple of dependencies:

{Cargo.toml}

Then we extend the build pipeline with grass invocation and writing the result into the special path that cargo creates for output artifacts using a prest util to find it automatically.

{build.rs}

And finally we can use prest's `embed_build_output_as!` macro to include all the files from that directory:

{serve.rs}