NAME = under
OUT = out

target/debug/$(NAME): $(wildcard src/*.rs) Cargo.toml
	cargo build
