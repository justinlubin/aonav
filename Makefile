.PHONY: build image image-export clean

build:
	cargo build

image:
	podman build -t aonav .
	mkdir -p target

image-export: image
	podman save aonav:latest | gzip > target/aonav-image.tar.gz

clean:
	cargo clean
