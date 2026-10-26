# Codex Path Native Bridge

A small, backend-only [codex-proxy-rs (CPR)](https://github.com/zyycn/codex-proxy-rs) plugin for compatible Codex clients that request `/v1/codex/responses` instead of CPR's native `/v1/responses` endpoint. It changes the path, not the client or protocol.

## Routing contract

Only the exact path `/v1/codex/responses` with uppercase `GET` or `POST` is rewritten to `/v1/responses`:

| Request | Result |
| --- | --- |
| `POST /v1/codex/responses` | `POST /v1/responses` (including SSE requests) |
| `GET /v1/codex/responses` | `GET /v1/responses` (including WebSocket handshakes) |
| Either path with a query string | Query bytes preserved exactly |
| Any other path or method | Unchanged |

Absolute URI authority is preserved. The existing request is passed to the SDK's consuming `next` exactly once. Headers, settings, timeout, lazy body handles, responses and upgrades are not changed. No request-body conversion or response-body buffering is added.

CPR owns authentication, client-key access scope, account selection, native Codex protocol handling, SSE streaming, WebSocket sessions, continuation and accounting. This bridge does not implement authentication, a model catalog, an upstream adapter or client-specific tooling. It does not make incompatible clients compatible with CPR's protocol. Clients already using `/v1/responses` do not need this alias.

## Compatibility

- Source/package version: `0.2.0`.
- Manifest host range: CPR `>=3.32.0, <3.33.0`. This release targets CPR `3.32.0`'s RPC v6 and middleware v6 contracts; it is not compatible with the old RPC used by `0.1.x`. Future host releases must be checked against their published compatibility contract.
- Official SDK and packaging CLI pinned to CPR revision `6a5300a7b5c649f2a6d548df70ccdd58d15e0025` (`v3.32.0`).
- Stable plugin ID: `selfhosted.omp-native-bridge`. The legacy machine name is retained for identity continuity; it does not restrict the plugin to one client.

The public repository is [codex-path-native-bridge](https://github.com/ninthcj/codex-path-native-bridge). The Rust package/binary keeps its existing name, `cpr-omp-native-bridge`.

## Build and package

Requires Rust/Cargo `1.99.1` or newer, Git, Make and access to the pinned upstream source and Cargo dependencies. `Cargo.lock` is retained. Run from the repository root:

```sh
make test
make cli
make package
```

`make cli` installs the pinned official `cpr-plugin` packaging CLI into the repository's `.tools/`, not a global tool directory. `make package` builds the release binary and writes the installable archive to `dist/` using that CLI.

The default target is derived from `rustc --print host-tuple`. Build for the CPR host's operating system and architecture, not necessarily the development machine's. For example, on a matching Linux x86_64 host or with a configured cross-toolchain:

```sh
make package TARGET=x86_64-unknown-linux-gnu
```

An explicit target requires its Rust target components and a suitable linker/toolchain. A macOS package cannot be installed as a Linux plugin. Generated binaries, packages and local CLI tools are excluded from the source repository; this repository does not supply prebuilt releases.

## Install in CPR

1. Build a package for the host platform.
2. In CPR's **Plugin Management**, choose installation from an **uploaded package** and select the archive from `dist/`.
3. Review the publisher, name, version, target platform and digest. Confirm installation only if you trust the code and its source.
4. Confirm the configuration is enabled and published. This plugin has no configuration parameters or secrets. For an existing installation, review and confirm the switch to the new package version rather than assuming upload alone changes the active version.
5. Use a compatible client's normal CPR credentials and endpoint settings so its requests reach `/v1/codex/responses` on the gateway. Existing CPR access rules still apply.

### Trust and failure scope

CPR plugins run as trusted processes with the gateway's system identity. They can access its data, credentials, configuration and network; CPR does not provide a plugin security sandbox. Package/digest/compatibility checks do not prove code safety.

This plugin contributes **global HTTP middleware**, before routing and authentication. Its path rewrite is narrow, but its process participates in the middleware chain for other HTTP routes too. Runtime/framework failures can therefore affect unrelated requests according to CPR's configured reject/delegate failure policy. Client-key or model scope is not a sandbox around this HTTP contribution.

The implementation does not itself read or store credentials. It still receives the HTTP request, including its headers, through the trusted middleware contract. Do not disable it while clients depend on the alias: disabling it removes this path rewrite, not CPR's native endpoint.

## Verification

The existing three deterministic Rust tests cover:

- `GET`/`POST` rewriting with exact query and authority preservation;
- unchanged unrelated paths and URI forms;
- unchanged unrelated methods, including case variants.

Version `0.2.0` passed the three Rust tests and was built and packaged on macOS arm64 with the pinned official CLI. An offline fixture host exercised the actual release plugin process over RPC v6: registration and rejection of a mismatched contribution, 15 HTTP middleware cases, and clean shutdown. These checks covered the GET WebSocket upgrade response and POST SSE response handles, exact URI/query preservation, unchanged headers/settings/timeouts/lazy body handles, one `next` callback per request, and no body reads or response buffering. The fixture is not a running CPR gateway or an upstream model service; live authentication, network SSE/WebSocket traffic, and continuation on CPR `3.32.0` have not been exercised.

The unchanged routing implementation was previously exercised against CPR `3.18.3` with real SSE and WebSocket requests and multi-turn continuation through a compatible client. That historical evidence does not establish end-to-end compatibility of this SDK upgrade.

For your own installation, verify that authentication still applies, then exercise the client's SSE and WebSocket transports and a continuation turn. A successful upload or an enabled status alone is not evidence of end-to-end client compatibility.

## License

This project's source is licensed under [Apache License 2.0](LICENSE).

It depends on CPR's official `gateway-plugin-sdk`, which inherits `Apache-2.0` from the [pinned upstream workspace](https://github.com/zyycn/codex-proxy-rs/blob/6a5300a7b5c649f2a6d548df70ccdd58d15e0025/backend/Cargo.toml). The pinned upstream root contains no `NOTICE` file. Dependencies remain under their respective licenses; this project's license does not relicense them. Binary redistributors must also satisfy the licenses of the dependencies they distribute.

## Changes

### 0.2.0

Update the official SDK and packaging CLI to CPR `3.32.0`, with RPC v6 and middleware v6. Raise the Rust requirement to match upstream and replace the old broad host range with `>=3.32.0, <3.33.0`. Keep the stable plugin ID and routing implementation unchanged; no cache control, client-specific conversion, or additional call-budget overrides are introduced. Rebuild and replace old plugin packages when upgrading the host; a `0.1.x` package does not become RPC-v6 compatible by changing its metadata.

### 0.1.2

Adopt Apache-2.0 licensing and publish client-neutral build, installation and trust documentation. Keep the stable plugin ID, pinned SDK/CLI and routing implementation unchanged. Package version advanced to distinguish future public packages from earlier artifacts.

## Upstream references

- [Plugin installation and trust](https://github.com/zyycn/codex-proxy-rs/blob/6a5300a7b5c649f2a6d548df70ccdd58d15e0025/docs/plugins.md)
- [Official SDK](https://github.com/zyycn/codex-proxy-rs/blob/6a5300a7b5c649f2a6d548df70ccdd58d15e0025/backend/crates/gateway-plugin/sdk/README.md)
- [Official packaging CLI](https://github.com/zyycn/codex-proxy-rs/blob/6a5300a7b5c649f2a6d548df70ccdd58d15e0025/backend/apps/plugin-cli/README.md)
