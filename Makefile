.PHONY: lint
lint:
	cargo +nightly fmt
	cargo check
	cargo clippy

.PHONY: test
test:
	cargo test

.PHONY: install
install:
	cargo install --path .

.PHONY: authors
authors:
	./scripts/generate-authors.sh
