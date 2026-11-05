TARGET ?= $(shell rustc --print host-tuple)
CPR_REV := 9039cef3d87ae86ae9422b17c5d4dae1a5247dfa
CLI := $(CURDIR)/.tools/bin/cpr-plugin

.PHONY: build test cli package

build:
	cargo build --release --locked --target $(TARGET)

test:
	cargo test --locked

cli:
	cargo install --git https://github.com/zyycn/codex-proxy-rs.git --rev $(CPR_REV) --locked --root $(CURDIR)/.tools codex-proxy-plugin-cli

package: build
	$(CLI) package --manifest $(CURDIR)/plugin.json --binary $(CURDIR)/target/$(TARGET)/release/cpr-omp-native-bridge --target $(TARGET) --output-dir $(CURDIR)/dist
