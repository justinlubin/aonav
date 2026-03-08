.PHONY: build image clean

build:
	cargo build

image:
	mkdir -p target
	podman build -t aonav .
	# podman save aonav:latest | gzip > target/aonav-image.tar.gz

clean:
	cargo clean
