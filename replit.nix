{ pkgs }: {
	deps = [
    pkgs.rustc
		pkgs.rustfmt
		pkgs.cargo
		pkgs.rust-analyzer
		pkgs.pkg-config
		pkgs.openssl # for candle-mistral (hf-hub model download)
		pkgs.sqlite # for sqlx-sqlite example
	];
}