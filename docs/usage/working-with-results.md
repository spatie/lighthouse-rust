---
title: Working with results
weight: 3
---

Running Lighthouse will return a `LighthouseResult`. On this page, we'll cover the various methods you can call on it.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let result = Lighthouse::url("https://example.com")?.run()?;
# let _ = result;
# Ok(())
# }
```

## Getting the category scores

You can use `scores()` to get scores of the four categories Lighthouse runs audits for. Use `score()` for a single category.

```rust
# use spatie_lighthouse::{Category, LighthouseResult};
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let scores = result.scores();
let accessibility = result.score(Category::Accessibility);
# assert_eq!(scores.get("accessibility"), Some(&92));
# assert_eq!(accessibility, Some(92));
# Ok(())
# }
```

The map uses category identifiers as keys and integer percentages as values. As in PHP, `scores()` omits missing, null and zero scores. `score()` returns `Some(0)` for a real zero and `None` for missing/null. Values truncate rather than round after multiplying by 100. Map keys are sorted lexicographically.

## Getting audit results

Lighthouse will run various audits on your website. You can learn more about the meaning of each audit in [the Lighthouse docs](https://developer.chrome.com/docs/lighthouse/overview).

You can get the names of all audits using `audit_names()`. Here's how you can get the results of an audit:

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let all_audit_names = result.audit_names();
let audit = result.audit("first-contentful-paint")?;
let all_audits = result.audits();
# assert_eq!(all_audit_names[0], "is-on-https");
# assert_eq!(audit["numericValue"], 1262.95);
# assert_eq!(all_audits.unwrap().len(), 152);
# Ok(())
# }
```

`audit()` returns a borrowed `serde_json::Value`, or `Error::AuditDoesNotExist`. `audits()` returns an optional audit map. Both preserve Lighthouse's audit order. The example above uses the original PHP fixture, not a current benchmark.

## Performance metrics convenience methods

There are a few convenience methods to get the most interesting metrics for the performance category. You can call all these methods on `result`. The ones ending in `_in_ms` return results in milliseconds.

- `first_contentful_paint_in_ms()`
- `largest_contentful_paint_in_ms()`
- `speed_index_in_ms()`
- `total_blocking_time_in_ms()`
- `time_to_interactive_in_ms()`
- `cumulative_layout_shift()`
- `total_page_size_in_bytes()`

Some of these methods have equivalents that return a formatted string such as `1.3 s`.

- `formatted_first_contentful_paint()`
- `formatted_largest_contentful_paint()`
- `formatted_speed_index()`
- `formatted_total_blocking_time()`
- `formatted_time_to_interactive()`
- `formatted_cumulative_layout_shift()`

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let milliseconds = result.first_contentful_paint_in_ms()?;
let formatted = result.formatted_first_contentful_paint()?;
# assert_eq!(milliseconds, Some(1262.95));
# assert_eq!(formatted, Some("1.3\u{a0}s"));
# Ok(())
# }
```

These return `Result<Option<T>, Error>`. A missing audit is an error; an audit with no numeric/display value returns `None`. Availability depends on the Lighthouse version and selected audits. Time to interactive is no longer a scored performance metric, but Lighthouse 13 still includes its hidden audit. Formatted cumulative layout shift returns the original display string rather than PHP's float coercion.

## Getting the configuration used to generate the result

The result also contains the settings used to run Lighthouse. Use dot notation to get a specific setting, such as the browser width.

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let settings = result.config_settings();
let width = result.config_setting("screenEmulation.width");
# assert!(settings.unwrap().is_object());
# assert_eq!(width, Some(&serde_json::json!(360)));
# Ok(())
# }
```

There are also convenience methods for interesting pieces of configuration:

- `form_factor()`: returns `Result<Option<FormFactor>, Error>`.
- `user_agent()`: returns an optional user agent string.
- `headers()`: returns an optional map of extra headers.
- `network_throttling_was_enabled()`: returns a boolean from the recorded flag.
- `cpu_throttling_was_enabled()`: returns a boolean from the recorded flag.

As in PHP, absent throttling-disable flags count as enabled. These flags reflect recorded configuration, not independently verified browser behavior.

## Getting the raw results

To get everything returned by the wrapper, use `raw_results()`. It contains `report: [json, html]` and `runtimeError: null` for successful live runs.

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let raw = result.raw_results();
let version = result.raw_result("report.0.lighthouseVersion");
let score = result.get("categories.accessibility.score");
let json = result.json_report();
# assert_eq!(raw["report"][0], *json);
# assert_eq!(version, Some(&serde_json::json!("9.6.8")));
# assert_eq!(score, Some(&serde_json::json!(0.92)));
# Ok(())
# }
```

`get()` reads a JSON report property. `json_report()` borrows the whole JSON report and `to_array()` returns an owned copy. `raw_result()` and `config_setting()` provide the path-taking forms of PHP's optional-argument accessors. Missing paths return `None`; a present JSON null remains `Some(Value::Null)`.

The live wrapper returns the same small envelope as the PHP runner. `lhr.lighthouseVersion` only exists if you restore an older full runner result that includes `lhr`; use `report.0.lighthouseVersion` or `lighthouse_version()` for live runs.

## Getting the benchmark index

Lighthouse computes a benchmark index as a rough approximation of the host device's CPU performance. `benchmark_index()` returns an optional floating-point number, preserving fractional values from current Lighthouse versions.

## Restoring saved reports

Use `from_reports(json, html)` for separately stored reports, or `from_raw_results(raw)` for an envelope. The latter also accepts a JSON string in `report[0]`, as returned by the PHP Node script before normalization.

```rust
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::LighthouseResult;
# let json_report = serde_json::json!({"categories": {"seo": {"score": 0.92}}});
# let html_report = "<html><body>Stored report</body></html>";

let result = LighthouseResult::default()
    .set_json_report(json_report)?
    .set_html_report(html_report);
# assert_eq!(result.score(spatie_lighthouse::Category::Seo), Some(92));
# assert_eq!(result.html(), html_report);
# Ok(())
# }
```
