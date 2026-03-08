.PHONY: build image

build:
	cargo build

image:
	podman build .
