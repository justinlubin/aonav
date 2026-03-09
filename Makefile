.PHONY: build image clean

build:
	cargo build

image:
	podman build -t aonav .
	mkdir -p target
	podman save aonav:latest | gzip > target/aonav-image.tar.gz

clean:
	cargo clean
