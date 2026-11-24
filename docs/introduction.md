---
title: Introduction
weight: 1
---

[Google Lighthouse](https://developer.chrome.com/docs/lighthouse/overview/) is an open-source, automated tool for improving the quality of web pages. It has audits for performance, accessibility, SEO and more.

This package makes it easy to run Lighthouse using Rust. Here's an example on how to get the scores of the four categories of audits that Lighthouse offers.

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

It's easy to configure various options:

```rust,no_run
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::{Category, Lighthouse};

let result = Lighthouse::url("https://example.com")?
    .user_agent("My user agent")
    .headers([("MyExtraHeader", "HeaderValue")])
    .categories([Category::Performance, Category::Accessibility])
    .throttle_cpu(None)
    .run()?;
# let _ = result;
# Ok(())
# }
```

Here's how you can get the results of an audit:

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let audit = result.audit("first-contentful-paint")?;
# assert_eq!(audit["numericValue"], 1262.95);
# Ok(())
# }
```

You can also write a full HTML report to disk using `result.save_html("report.html")?`.

This is a wrapper around installed Node.js Lighthouse and Chrome, not a native reimplementation. Continue with [requirements](/docs/lighthouse-rust/v1/requirements) and [installation & setup](/docs/lighthouse-rust/v1/installation-setup).
