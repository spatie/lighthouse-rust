---
title: Installation & setup
weight: 4
---

You can install the package via Cargo:

```sh
cargo add spatie-lighthouse
```

This package relies on the `lighthouse` and `chrome-launcher` JavaScript packages being available on your system. In most cases you can accomplish this by issuing these commands in your application's working directory:

```sh
npm install lighthouse@^13 chrome-launcher@^1
```

Chrome or Chromium should also be available on the server. Install a current browser using your operating system's package manager or [Chrome for Testing](https://googlechromelabs.github.io/chrome-for-testing/).

If Chrome is not detected automatically, set the `CHROME_PATH` environment variable to its executable. You can also pass a `chromePath` through `with_chrome_options()`.

Use `node_binary()` to select a particular Node executable, and `working_directory()` when the Node modules are installed outside your application's current directory:

```rust,no_run
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::Lighthouse;

let result = Lighthouse::url("https://example.com")?
    .node_binary("node")
    .working_directory("/path/to/project")
    .run()?;
# let _ = result;
# Ok(())
# }
```

Chrome sandboxing stays enabled by default. Only in a trusted container that cannot provide Chrome's sandbox, opt out explicitly with `with_chrome_options(serde_json::json!({"chromeFlags": ["--headless", "--no-sandbox"]}))?`. Do not disable the sandbox when auditing untrusted pages unless the container supplies equivalent isolation.
