---
title: Testing
weight: 5
---

Run these checks from the package directory:

```sh
npm ci
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
cargo doc --locked --no-deps --all-features
cargo package --locked
```

Set `RUSTDOCFLAGS` to `-D warnings` when running the documentation commands. In POSIX shells, prefix each command with `RUSTDOCFLAGS='-D warnings'`. In PowerShell, first run `$env:RUSTDOCFLAGS = '-D warnings'`.

README.md and all Markdown pages in docs/ participate in rustdoc tests. Setup lines prefixed with `# ` compile and execute but are hidden in rendered rustdoc. Assertions check the examples' results.

Install Chrome or Chromium and set `CHROME_PATH` if it cannot be found automatically. The runtime tests launch real Chrome and Lighthouse against a disposable local HTTP page; no external website is required. The test-only Chrome options disable sandboxing so they can run inside trusted CI containers. The library does not disable the sandbox by default.

Saved-report doctests execute against the PHP fixture under `tests/fixtures`. External-website examples use `rust,no_run`: rustdoc compiles them without contacting the website. The integration suite separately exercises the live wrapper, report generation, custom output, headers, mobile emulation, budgets, runtime errors and timeouts.

Check the minimum supported Rust version:

```sh
rustup toolchain install 1.89.0 --profile minimal
cargo +1.89.0 test --locked --all-targets --all-features
cargo +1.89.0 test --locked --doc --all-features
```

Check security advisories:

```sh
cargo install cargo-audit --locked
cargo audit
```

CI runs stable checks with Lighthouse 13 on Linux, macOS and Windows, and an additional Lighthouse 11.7.1 job on Linux to exercise legacy budget audits. The MSRV and audit checks run on Linux. An audit requires access to the RustSec advisory database.
