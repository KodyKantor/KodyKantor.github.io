EXEC = compiler
RELEASE = target/release/$(EXEC)
DEV = target/debug/$(EXEC)
RUST_FILES = src/main.rs

.PHONY: all
all: $(RELEASE)

.PHONY: dev
dev: $(DEV)

.PHONY: clean
clean:
	cargo clean

.PHONY: check
check:
	cargo clippy

$(RELEASE): $(RUST_FILES)
	cargo build --release

$(DEV): $(RUST_FILES)
	cargo build
