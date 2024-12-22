Simple [scraper](https://github.com/causal-agent/scraper-based)-based parser that collects posts from [AP News](https://apnews.com). Beside scraper it uses [reqwest](https://github.com/seanmonstar/reqwest) which is a standard option in tokio ecosystem to make requests: 

{Cargo.toml}

This example spawns the `scrape` function which fetches the provided url, extracts links to other pages from it (but using only 5 of them later to limit the load), then using `join_all` function from the `futures` crate to get all these pages concurrently, then awaits their bodies, then extracts titles and contents and saves the results:

{src/main.rs}