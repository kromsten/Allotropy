.PHONY: local-chain lint optimize build build-local-image

GAS_LIMIT ?= "150000000"
OPTIMIZER_SUFFIX := $(shell if [ "$$(uname)" = "Darwin" ]; then echo "-arm64"; else echo ""; fi)
TEST_ADDRS ?= $(shell jq -r '.[].address' ./typescript/config/accounts.json | tr '\n' ' ')


local-chain: build-local-image
	docker kill gaia || true
	docker volume rm -f gaia_data
	docker run --rm -d --name gaia \
		-e DENOM=uatom \
		-e CHAINID=testing \
		-e GAS_LIMIT=$(GAS_LIMIT) \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=gaia_data,target=/root \
		local-gaia /data/entry-point.sh $(TEST_ADDRS)


build-local-image:
	docker build -t local-gaia ./docker_builds


optimize:
	docker run --rm -v "$$(pwd)":/code \
	--mount type=volume,source="$$(basename "$$(pwd)")_cache",target=/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/optimizer$(OPTIMIZER_SUFFIX):0.17.0


build:
	RUSTFLAGS='-C link-arg=-s' cargo build  --target wasm32-unknown-unknown --release --lib

lint:
	cargo clippy --all-targets -- -D warnings
