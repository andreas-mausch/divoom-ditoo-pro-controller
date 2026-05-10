BINARY      = target/release/divoom-ditoo-pro-controller
ENTITLEMENTS = divoom.entitlements

build:
	cargo build --release
	codesign -s - --entitlements $(ENTITLEMENTS) --force $(BINARY)

run: build
	$(BINARY) $(ARGS)

test:
	cargo test --all
