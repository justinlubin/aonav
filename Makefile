.PHONY: build image image-export clean

build:
	cargo build

image:
	podman build -t aonav .
	mkdir -p target

image-export: image
	podman save aonav:latest | gzip > artifact-eval/aonav-image.tar.gz

clean:
	cargo clean
	rm -f -- artifact-eval/aonav-image.tar.gz
