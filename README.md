# Run Google Lighthouse using Rust

[![Latest version on crates.io](https://img.shields.io/crates/v/spatie-lighthouse.svg?style=flat-square)](https://crates.io/crates/spatie-lighthouse)
[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/lighthouse-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/lighthouse-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/lighthouse-rust/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/spatie-lighthouse.svg?style=flat-square)](https://crates.io/crates/spatie-lighthouse)

[Google Lighthouse](https://developer.chrome.com/docs/lighthouse/overview/) is an open-source, automated tool for improving the quality of web pages. It has audits for performance, accessibility, SEO and more.

This package makes it easy to run Lighthouse using Rust. It wraps an installed Node.js Lighthouse and Chrome, rather than reimplementing Lighthouse in Rust.

```rust,no_run
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::Lighthouse;

let result = Lighthouse::url("https://example.com")?.run()?;

let scores = result.scores();
// For example: {"accessibility": 83, "best-practices": 90, "performance": 98, "seo": 92}
# let _ = scores;
# Ok(())
# }
```

## Support us

[<img src="https://spatie.be/github-ad-image/lighthouse-rust.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/lighthouse-rust)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer, Node.js 22.19 or newer, and Chrome or Chromium.

```sh
cargo add spatie-lighthouse
```

Install the Node packages in your application's working directory:

```sh
npm install lighthouse@^13 chrome-launcher@^1
```

If Chrome isn't detected automatically, set `CHROME_PATH` to its executable. Use `working_directory()` when the Node packages are installed in a different directory.

## Usage

It's easy to configure various options:

```rust,no_run
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::{Category, Lighthouse};

let result = Lighthouse::url("https://example.com")?
    .user_agent("My user agent")
    .headers([("MyExtraHeader", "HeaderValue")])
    .categories([Category::Performance, Category::Accessibility])
    .throttle_cpu(None)
    .max_wait_for_load(5000)
    .run()?;

let audit = result.audit("first-contentful-paint")?;
result.save_html("report.html")?;
# let _ = audit;
# Ok(())
# }
```

All documentation is available [on our documentation site](https://spatie.be/docs/lighthouse-rust/v1/introduction), including [configuration](https://spatie.be/docs/lighthouse-rust/v1/usage/configuring-a-run) and [working with results](https://spatie.be/docs/lighthouse-rust/v1/usage/working-with-results).

Current Lighthouse releases have four categories. Older PHP examples mentioning PWA refer to Lighthouse versions that still had that category. Metric availability also depends on the installed Lighthouse version. Lighthouse 12 removed performance budgets; generating those audits requires the optional older Lighthouse 11.7.1 runtime. See the [budget compatibility notes](https://spatie.be/docs/lighthouse-rust/v1/usage/using-performance-budgets).

## Testing

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

Install the Node dependencies with `npm ci` and make Chrome available before running the integration tests. See [all quality checks](https://github.com/spatie/lighthouse-rust/blob/main/docs/testing.md) and the [PHP parity notes](https://github.com/spatie/lighthouse-rust/blob/main/docs/parity.md). The README and Markdown guides are checked by rustdoc, including setup on hidden `# ` lines. Examples that run an external website compile without executing; local integration tests exercise real Lighthouse runs.

## Changelog

See [CHANGELOG.md](https://github.com/spatie/lighthouse-rust/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/lighthouse-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/lighthouse-rust/blob/main/SECURITY.md).

## Credits

- [Freek Van der Herten](https://github.com/freekmurze)
- [Original PHP package](https://github.com/spatie/lighthouse-php)
- [All contributors](https://github.com/spatie/lighthouse-rust/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/spatie/lighthouse-rust/blob/main/LICENSE.md).
