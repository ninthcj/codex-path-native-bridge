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

- Source/package version: `0.3.3`.
- Manifest host range: CPR `>=3.33.0, <3.37.0`. CPR `3.33.0`, `3.34.0`, `3.35.x` and `3.36.0` all accept only RPC v7 sessions and publish the same manifest schema 2 and middleware v6 contracts, so one package serves them. It is not compatible with the RPC v6 packages released as `0.2.x`, nor with the older RPC used by `0.1.x`. Future host releases must be checked against their published compatibility contract.
- Official SDK and packaging CLI pinned to CPR revision `5ae38dc0c55c192815ef605dde6c05fd767a2c28` (`v3.36.0`). CPR `3.36.0` keeps RPC v7, manifest schema 2 and middleware v6: the SDK, packaging CLI, host compatibility manifest and plugin documentation trees are identical to the `3.35.0` pin, and the plugin-runtime diff is three message strings. Its Responses, usage, admin and backup changes are outside the plugin contract this package contributes to.
- CPR `3.34.0` and later do not start a plugin whose declared host range or capability versions do not match, and an administrator's per-start risk confirmation does not survive a host restart. A `0.3.2` package declares `<3.36.0`, so it stays unstarted after an upgrade to CPR `3.36.x`; use `0.3.3` there.
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

An explicit target requires its Rust target components and a suitable linker/toolchain. A macOS package cannot be installed as a Linux plugin. Generated binaries, packages and local CLI tools are excluded from the source repository. Linux x86_64 packages and SHA-256 checksums are available in [GitHub Releases](https://github.com/ninthcj/codex-path-native-bridge/releases); other targets can be built from source.

## Install in CPR

1. In CPR's **Plugin Management**, choose **GitHub** and enter `ninthcj/codex-path-native-bridge`. Select `v0.3.3` (or a later compatible release) and the archive matching the host platform. Use the published SHA-256 when requested.
2. Alternatively, build a package for the host platform and choose **uploaded package** to install the archive from `dist/`.
3. Review the publisher, name, version, target platform and digest. Confirm installation only if you trust the code and its source.
4. Confirm the configuration is enabled and published. This plugin has no configuration parameters or secrets. For an existing installation, review and confirm the switch to the new package version rather than assuming upload alone changes the active version.
5. Use a compatible client's normal CPR credentials and endpoint settings so its requests reach `/v1/codex/responses` on the gateway. Existing CPR access rules still apply.

### Trust and failure scope

CPR plugins run as trusted processes with the gateway's system identity. They can access its data, credentials, configuration and network; CPR does not provide a plugin security sandbox. Package/digest/compatibility checks do not prove code safety.

This plugin contributes **global HTTP middleware**, before routing and authentication. Its path rewrite is narrow, but its process participates in the middleware chain for other HTTP routes too. Runtime/framework failures can therefore affect unrelated requests according to CPR's configured reject/delegate failure policy. Client-key or model scope is not a sandbox around this HTTP contribution.

The implementation does not itself read or store credentials. It still receives the HTTP request, including its headers, through the trusted middleware contract. Do not disable it while clients depend on the alias: disabling it removes this path rewrite, not CPR's native endpoint.

## Verification

The three deterministic routing tests cover:

- `GET`/`POST` rewriting with exact query and authority preservation;
- unchanged unrelated paths and URI forms;
- unchanged unrelated methods, including case variants.

Version `0.3.3` passed those tests plus three process-level RPC tests under both `cargo test` and `cargo test --release` on macOS arm64, and the release tests also passed for `x86_64-unknown-linux-gnu` inside a disposable Rust `1.99.1` container. The process tests spawn the compiled plugin binary and speak the SDK's RPC v7 framing to it as the host: they complete the handshake and Ready exchange, reject an RPC v6 handshake and a mismatched middleware contribution, and run 15 HTTP middleware calls. Those calls cover `GET`, `POST`, absolute-URI authority, exact query bytes, upgrade headers, request-body handles and response-body handles. Each call asserts that the plugin invokes `next` exactly once with the expected rewritten or unchanged URI, that headers, settings, timeout, version and lazy body handles are otherwise unchanged, and that status, headers, body handle and response source come back unchanged. Unexpected body reads, response buffering, duplicate `next` calls and unclean shutdown fail the test.

The `0.3.3` Linux x86_64 package was built from the `v0.3.3` tag with the pinned packaging CLI, and the published archive matched its SHA-256 sidecar.

It was installed through CPR `3.33.0`'s native GitHub verify/install flow (verify `200`, install `201`) on a production instance. Installation and activation are separate steps: the artifact was installed first, then a version plan (`200`) and version switch (`200`) moved the existing instance to it, preserving its configuration, bindings and middleware failure policy, and leaving the previous revision draining. The running instance reported revision 93, enabled/running, `actualArtifactSha256` `5cf27573…`, no compatibility warning, no load error and no API deprecation. Health probes returned `204` for healthz and `200` for health; both native `POST /v1/responses` and alias `POST /v1/codex/responses` rejected an unauthenticated request with `401`.

The host was then upgraded in place from CPR `3.33.0` to `3.34.0` (`ghcr.io/zyycn/codex-proxy-rs@sha256:624b984c…`) by changing only the compose image tag; PostgreSQL and Redis were reused, the container `nofile` limits (`65536`/`524288`), the `127.0.0.1:8080` binding, the log directory mount and the existing `config.yaml` were preserved. The host became healthy without waiting for the full start period, and the plugin instance came back enabled/running on the same revision and artifact with `compatibilityWarning` null, `loadError` null and no deprecations, which is the case a `0.3.0` package (<3.34.0) would fail. Healthz `204`, health `200` and both unauthenticated endpoint `401` probes matched the pre-upgrade results. No model requests were made, so authenticated SSE, WebSocket and continuation behavior was not exercised by this deployment check. The `0.2.0` Linux package had previously been installed through CPR `3.32.0` with live HTTP/WebSocket probes, but that evidence belongs to the RPC v6 package.

The host was then upgraded in place from CPR `3.34.0` to `3.35.1` (`ghcr.io/zyycn/codex-proxy-rs@sha256:3d0fb404…`) the same way, again changing only the compose image tag. CPR `3.35.1` satisfies the `>=3.35.0, <3.36.0` range this package requires but not the `0.3.1` range, so the previous artifact stopped with a compatibility warning during the upgrade gap and was then replaced: GitHub verify `200`, install `201`, version plan `200` with `compatibilityWarning` null and the existing empty configuration and `reject` middleware binding intact, then version switch `200` to instance revision 107. The running instance reported enabled/running, `actualArtifactSha256` `39e83373…`, no compatibility warning, no load error, no API deprecation and no draining revisions. The PostgreSQL and Redis containers were reused, and the container `nofile` limits (`65536`/`524288`), the `127.0.0.1:8080` binding, the data and log mounts and the existing `config.yaml` were unchanged; the host reported healthy immediately after start. Healthz returned `204` and health `200`; `POST /v1/responses` and `POST /v1/codex/responses` both returned `401` unauthenticated, and `GET` on both returned the same native `400` WebSocket-upgrade rejection, so the rewrite passes the native response through instead of producing one of its own. No model requests were made here either.

The `0.3.3` artifact was installed on the running CPR `3.35.1` instance through the same native GitHub flow and then carried across a host upgrade to CPR `3.36.0`. Verify returned `200` and install `201` with `actualArtifactSha256` `a2c23079…`, version plan `200` with `compatibilityWarning` null and the existing empty configuration and `reject` middleware binding intact, and version switch `200` to instance revision 123; the instance reported enabled/running with no compatibility warning, no load error, no API deprecation and revision 107 draining. The compose image was then moved from the `3.35.1` tag to the `ghcr.io/zyycn/codex-proxy-rs:3.36.0` version tag, which resolves to `sha256:7cc0c08e…`, and the service recreated, which was the only change: the container kept `nofile` `65536`/`524288`, `pids_limit` 256, the `127.0.0.1:8080` binding, the `10001:10001` user and the data, log and `config.yaml` mounts, and the PostgreSQL and Redis containers were not recreated. The host reported healthy (`204` healthz, `200` health) well before the start period elapsed, and the plugin came back enabled/running on revision 123 and the same artifact with `compatibilityWarning` null, `loadError` null, no API deprecation and no draining revisions, which is the case a `0.3.2` package (<3.36.0) would fail after the upgrade. `POST /v1/responses` and `POST /v1/codex/responses` both returned the same native `401` body, and `GET` on both returned the same native `400` WebSocket-upgrade rejection, so the rewrite still passes native responses through. No model requests were made here either, so authenticated SSE, WebSocket and continuation behavior was not exercised by this deployment check.

For your own installation, verify that authentication still applies, then exercise the client's SSE and WebSocket transports and a continuation turn. A successful upload or an enabled status alone is not evidence of end-to-end client compatibility.

## License

This project's source is licensed under [Apache License 2.0](LICENSE).

It depends on CPR's official `gateway-plugin-sdk`, which inherits `Apache-2.0` from the [pinned upstream workspace](https://github.com/zyycn/codex-proxy-rs/blob/5ae38dc0c55c192815ef605dde6c05fd767a2c28/backend/Cargo.toml). The pinned upstream root contains no `NOTICE` file. Dependencies remain under their respective licenses; this project's license does not relicense them. Binary redistributors must also satisfy the licenses of the dependencies they distribute.

## Changes

### 0.3.3

Widen the declared host range to `>=3.33.0, <3.37.0` so CPR `3.36.0` accepts and starts this package, and move the official SDK and packaging CLI pin to CPR revision `5ae38dc0c55c192815ef605dde6c05fd767a2c28` (`v3.36.0`). CPR `3.36.0` keeps RPC v7, manifest schema 2, middleware v6 and the same capability list: the pinned SDK, packaging CLI, host compatibility manifest and plugin documentation are byte-identical between the two revisions, and the plugin-runtime diff is three error-message strings, so the routing implementation, configuration schema, empty configuration and manifest contributions are unchanged. The source diff is the manifest version/range, the crate version, the dependency pin and `Cargo.lock`. The binary is rebuilt because CPR `3.36.0` does not start a plugin whose declared host range does not match, and a `0.3.2` package declares `<3.36.0`.

### 0.3.2

Update the official SDK and packaging CLI to CPR revision `80f7d53085f5c9552e7e74b796a0bec50586ee1b` (`v3.35.0`) and widen the declared host range to `>=3.33.0, <3.36.0` so CPR `3.35.x` accepts and starts this package. CPR `3.35.0` keeps RPC v7, manifest schema 2 and middleware v6: the SDK's wire message definitions are byte-identical to the `3.34.0` pin, and the only capability-version change removes `model_catalog` v1, which this package does not contribute. The routing implementation, configuration schema, empty configuration and manifest contributions are unchanged; the source diff is the manifest version/range, the crate version and the dependency pin. The binary is rebuilt because CPR `3.34.0` and later do not start a plugin whose declared host range does not match, and a `0.3.1` package declares `<3.35.0`.

### 0.3.1

Widen the declared host range to `>=3.33.0, <3.35.0` so CPR `3.34.0` accepts and starts this package. CPR `3.34.0` keeps RPC v7, manifest schema 2, middleware v6 and the existing package inspection, so the pinned SDK/CLI revision, routing implementation, configuration schema and manifest contributions are unchanged; the source diff is the manifest version/range and the crate version. The binary is rebuilt because CPR `3.34.0` no longer starts a plugin whose declared host range does not match, and a `0.3.0` package declares `<3.34.0`.

### 0.3.0

Update the official SDK and packaging CLI to CPR `3.33.0`, with RPC v7 and the unchanged middleware v6 contract. Raise the host range to `>=3.33.0, <3.34.0`; CPR `3.33.0` accepts only RPC v7 sessions, so `0.2.x` RPC v6 packages must be rebuilt rather than reused with changed metadata. Add process-level tests that run the compiled plugin against an offline RPC v7 host fixture. Keep the stable plugin ID, routing implementation, empty configuration and manifest contribution stages unchanged; no cache control, client-specific conversion, streaming change or additional call-budget override is introduced.

### 0.2.0

Update the official SDK and packaging CLI to CPR `3.32.0`, with RPC v6 and middleware v6. Raise the Rust requirement to match upstream and replace the old broad host range with `>=3.32.0, <3.33.0`. Keep the stable plugin ID and routing implementation unchanged; no cache control, client-specific conversion, or additional call-budget overrides are introduced. Rebuild and replace old plugin packages when upgrading the host; a `0.1.x` package does not become RPC-v6 compatible by changing its metadata.

### 0.1.2

Adopt Apache-2.0 licensing and publish client-neutral build, installation and trust documentation. Keep the stable plugin ID, pinned SDK/CLI and routing implementation unchanged. Package version advanced to distinguish future public packages from earlier artifacts.

## Upstream references

- [Plugin installation and trust](https://github.com/zyycn/codex-proxy-rs/blob/5ae38dc0c55c192815ef605dde6c05fd767a2c28/docs/plugins.md)
- [Official SDK](https://github.com/zyycn/codex-proxy-rs/blob/5ae38dc0c55c192815ef605dde6c05fd767a2c28/backend/crates/gateway-plugin/sdk/README.md)
- [Official packaging CLI](https://github.com/zyycn/codex-proxy-rs/blob/5ae38dc0c55c192815ef605dde6c05fd767a2c28/backend/apps/plugin-cli/README.md)
