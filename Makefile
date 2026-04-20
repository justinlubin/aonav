.PHONY: build image clean

build:
	cargo build

image:
	docker buildx build \
		--platform linux/amd64 \
		--output type=docker,dest=- \
		-t aonav \
		. \
		| gzip > artifact-eval/aonav-amd64.tar.gz
	docker buildx build \
		--platform linux/arm64 \
		--output type=docker,dest=- \
		-t aonav \
		. \
		| gzip > artifact-eval/aonav-arm64.tar.gz

clean:
	cargo clean
	rm -f -- artifact-eval/aonav-image.tar.gz
