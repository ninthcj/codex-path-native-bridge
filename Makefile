TARGET ?= $(shell rustc --print host-tuple)
CPR_REV := 6a5300a7b5c649f2a6d548df70ccdd58d15e0025
CLI := $(CURDIR)/.tools/bin/cpr-plugin

.PHONY: build test cli package

build:
	cargo build --release --target $(TARGET)

test:
	cargo test

cli:
	cargo install --git https://github.com/zyycn/codex-proxy-rs.git --rev $(CPR_REV) --locked --root $(CURDIR)/.tools codex-proxy-plugin-cli

package: build
	$(CLI) package --manifest $(CURDIR)/plugin.json --binary $(CURDIR)/target/$(TARGET)/release/cpr-omp-native-bridge --target $(TARGET) --output-dir $(CURDIR)/dist
