FROM rust:1.78.0-bookworm
RUN apt-get update
RUN apt install -y gcc-x86-64-linux-gnu
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup toolchain install stable-x86_64-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown