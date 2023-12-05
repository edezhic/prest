# Manual due to https://ask.replit.com/t/deployment-time-outs/73694
cargo build -p blog --target x86_64-unknown-linux-musl --release
mv target/x86_64-unknown-linux-musl/release/serve _temp_deployment