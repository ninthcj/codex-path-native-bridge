TARGET ?= $(shell rustc --print host-tuple)
CPR_REV := 789609dd53663b323e43567d2476f7f83b6b77a1
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
