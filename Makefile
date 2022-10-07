.PHONY: lint
lint:
	cargo +nightly fmt
	cargo clippy --tests -- -Dclippy::all -Dclippy::pedantic

.PHONY: test
test:
	cargo test

.PHONY: install
install:
	cargo install --path .

.PHONY: authors
authors:
	./scripts/generate-authors.sh
