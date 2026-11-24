---
title: Generating your first report
weight: 1
---

With the package installed, this is how you can run Lighthouse.

```rust,no_run
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::Lighthouse;

let result = Lighthouse::url("https://example.com")?.run()?;
# let _ = result;
# Ok(())
# }
```

You can use the `scores()` method to get scores of the four categories Lighthouse runs audits for.

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let scores = result.scores();
// For example: {"accessibility": 92, "best-practices": 100, "performance": 100, "seo": 91}
# assert_eq!(scores.get("accessibility"), Some(&92));
# assert_eq!(scores.get("performance"), Some(&100));
# Ok(())
# }
```

Here's how you can get the results of an audit:

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let audit = result.audit("first-contentful-paint")?;
// The audit includes id, title, score, numericValue, numericUnit and displayValue.
# assert_eq!(audit["title"], "First Contentful Paint");
# assert_eq!(audit["numericValue"], 1262.95);
# Ok(())
# }
```

To get the results of all audits, call `audits()`. It returns the report's audit map, or `None` if you restored a report without audits. The result examples use the PHP package's saved example.com report; real scores vary by page, browser version and machine.
