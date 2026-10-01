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

- Source/package version: `0.1.2`.
- Manifest host range: CPR `>=3.18.3, <4.0.0`; runtime verification has covered CPR `3.18.3`, not every version in that range.
- Official SDK and packaging CLI pinned to CPR revision `9c1067676495623111497111b7cb136544fe1828`.
- Stable plugin ID: `selfhosted.omp-native-bridge`. The legacy machine name is retained for identity continuity; it does not restrict the plugin to one client.

The public repository is [codex-path-native-bridge](https://github.com/ninthcj/codex-path-native-bridge). The Rust package/binary keeps its existing name, `cpr-omp-native-bridge`.

## Build and package

Requires Rust/Cargo `1.97` or newer, Git, Make and access to the pinned upstream source and Cargo dependencies. `Cargo.lock` is retained. Run from the repository root:

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

The unchanged routing implementation has previously been exercised against CPR `3.18.3` with real SSE and WebSocket requests and multi-turn continuation through a compatible client. These observations establish the path bridge's behavior, not compatibility with every client or future CPR release. Version `0.1.2` changes public licensing and package metadata, not routing behavior; those earlier live checks were not a deployment of this version.

For your own installation, verify that authentication still applies, then exercise the client's SSE and WebSocket transports and a continuation turn. A successful upload or an enabled status alone is not evidence of end-to-end client compatibility.

## License

This project's source is licensed under [Apache License 2.0](LICENSE).

It depends on CPR's official `gateway-plugin-sdk`, which inherits `Apache-2.0` from the [pinned upstream workspace](https://github.com/zyycn/codex-proxy-rs/blob/9c1067676495623111497111b7cb136544fe1828/backend/Cargo.toml). The pinned upstream tree contains no `NOTICE` file, so no upstream NOTICE text is reproduced here. Dependencies remain under their respective licenses; this project's license does not relicense them. Binary redistributors must also satisfy the licenses of the dependencies they distribute.

## Changes

### 0.1.2

Adopt Apache-2.0 licensing and publish client-neutral build, installation and trust documentation. Keep the stable plugin ID, pinned SDK/CLI and routing implementation unchanged. Package version advanced to distinguish future public packages from earlier artifacts.

## Upstream references

- [Plugin installation and trust](https://github.com/zyycn/codex-proxy-rs/blob/9c1067676495623111497111b7cb136544fe1828/docs/plugins.md)
- [Official SDK](https://github.com/zyycn/codex-proxy-rs/blob/9c1067676495623111497111b7cb136544fe1828/backend/crates/gateway-plugin/sdk/README.md)
- [Official packaging CLI](https://github.com/zyycn/codex-proxy-rs/blob/9c1067676495623111497111b7cb136544fe1828/backend/apps/plugin-cli/README.md)
