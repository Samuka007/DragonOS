all:
	@cargo build --release -p gen_kallsyms
fmt:
	cargo fmt --all $(FMT_CHECK)
clean:
	@cargo clean
check:
	@cargo check --workspace $(CARGO_ZBUILD) --message-format=json
.PHONY: fmt