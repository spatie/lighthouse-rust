---
title: Configuring a run
weight: 2
---

There are various methods to configure how Lighthouse should run. You can use these between `url()` and `run()`. Here's an example where we set a custom user agent.

```rust,no_run
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::Lighthouse;

let result = Lighthouse::url("https://example.com")?
    .user_agent("my-custom-user-agent")
    .run()?;
# let _ = result;
# Ok(())
# }
```

`url()` accepts absolute HTTP(S) URLs and returns `Error::InvalidUrl` for invalid input. Configuration methods consume and return the builder. Keep the returned value when configuring it in separate statements. `run()` borrows the builder, so you can reuse a configuration.

## Only run audits in certain categories

By default, Lighthouse will run audits of all categories. To only run the audits of certain categories, call `categories()` and pass it one or more categories you are interested in.

```rust,no_run
# fn main() -> Result<(), spatie_lighthouse::Error> {
use spatie_lighthouse::{Category, Lighthouse};

let result = Lighthouse::url("https://example.com")?
    .categories([Category::BestPractices, Category::Seo])
    .run()?;
# let _ = result;
# Ok(())
# }
```

Use `"seo".parse::<Category>()?` when reading a category from a string. Unknown values return `Error::InvalidEnumValue`. Current Lighthouse releases no longer have a PWA category.

## Skip certain audits

To lower Lighthouse's execution time, you can opt to skip audits by passing their names to `skip_audits()`.

```rust
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let run = Lighthouse::url("https://example.com")?
    .skip_audits(["is-on-https", "unused-javascript"]);
# assert_eq!(run.lighthouse_script_arguments()["lighthouseConfig"]["settings"]["skipAudits"][0], "is-on-https");
# Ok(())
# }
```

Audit identifiers depend on your installed Lighthouse version. For example, the old `service-worker` audit is no longer available in current releases.

## Only run specific audits

You can opt to run only specific audits using `only_audits()`.

```rust
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let run = Lighthouse::url("https://example.com")?
    .only_audits(["is-on-https", "document-title"]);
# assert!(run.lighthouse_script_arguments()["lighthouseConfig"]["settings"].get("onlyCategories").is_none());
# Ok(())
# }
```

Do not combine `skip_audits()` and `only_audits()`; Lighthouse validates that combination.

If you want to run a specific audit and an entire other category, you must call `categories()` after `only_audits()`. In this example we run the `is-on-https` audit together with all audits from the `Seo` category.

```rust
# use spatie_lighthouse::{Category, Lighthouse};
# fn main() -> Result<(), spatie_lighthouse::Error> {
let run = Lighthouse::url("https://example.com")?
    .only_audits(["is-on-https"])
    .categories([Category::Seo]);
# assert_eq!(run.lighthouse_script_arguments()["lighthouseConfig"]["settings"]["onlyCategories"][0], "seo");
# Ok(())
# }
```

## Customizing the user agent

To use a custom user agent, pass a string to `user_agent()`, as in the first example on this page.

## Setting extra headers

You can specify headers that will be sent along with all requests.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let result = Lighthouse::url("https://example.com")?
    .headers([("MyExtraHeader", "value of the header")])
    .run()?;
# let _ = result;
# Ok(())
# }
```

Headers and report data may contain credentials. The builders' `Debug` output is redacted, but raw JSON and errors from Node may include private information. Do not log them indiscriminately.

## Specifying the form factor

By default, Lighthouse will use a desktop profile to run audits. You can change this to mobile using `form_factor()`.

```rust,no_run
# use spatie_lighthouse::{FormFactor, Lighthouse, ScreenEmulation};
# fn main() -> Result<(), spatie_lighthouse::Error> {
let result = Lighthouse::url("https://example.com")?
    .form_factor(FormFactor::Mobile)
    .screen_emulation(ScreenEmulation {
        mobile: true,
        width: Some(390),
        height: Some(844),
        device_scale_ratio: Some(2.0),
        ..Default::default()
    })
    .run()?;
# let _ = result;
# Ok(())
# }
```

The form factor and screen emulation are separate. Screen emulation is disabled by default; calling `screen_emulation()` replaces it. The default `ScreenEmulation` value enables desktop emulation with Lighthouse's default dimensions.

`device_scale_ratio` sets the number of physical pixels per CSS pixel using Lighthouse's `deviceScaleFactor` setting.

## Enable throttling

By default, Lighthouse will not throttle CPU and connection speed. To enable throttling, use these methods.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let result = Lighthouse::url("https://example.com")?
    .throttle_cpu(None)
    .throttle_network()
    .run()?;
# let _ = result;
# Ok(())
# }
```

Optionally, pass `Some(4)` to `throttle_cpu()` to set a CPU slowdown multiplier. The higher the number, the more throttling is applied. You'll find more information [in the Lighthouse docs](https://github.com/GoogleChrome/lighthouse/blob/main/docs/throttling.md#cpu-throttling).

`do_not_throttle_cpu()` and `do_not_throttle_network()` turn the corresponding flags off without resetting other settings. `throttle_network()` leaves the throttling method unchanged. Modern Lighthouse uses `throttlingMethod`; use `with_config()` for exact control over the simulation. The reported flags describe the submitted settings, not independent measurements of throttling.

## Customize the Lighthouse configuration

To have fine-grained control of which options will be sent to Lighthouse, pass a JSON object to `with_config()`.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let mut config = Lighthouse::default_lighthouse_config();
config["settings"]["locale"] = "nl".into();

let result = Lighthouse::url("https://example.com")?
    .with_config(config)?
    .run()?;
# let _ = result;
# Ok(())
# }
```

This replaces the entire configuration, not just the settings. `default_lighthouse_config()` returns the defaults: `extends` is `lighthouse:default`; settings select the four categories, desktop form factor, JSON and HTML output, no throttling, `throttlingMethod: "provided"`, and disabled screen emulation.

Configuration, settings and any non-null throttling block must be objects. The Rust wrapper checks those shapes before any fluent setter can index them. Lighthouse validates the rest of its schema. The wrapper always returns both JSON and HTML, even when a custom `output` selects only one format.

## Customize the Chrome options

Under the hood, Lighthouse will run an instance of Chrome to run the audits. Customize the options given to chrome-launcher using `with_chrome_options()`.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let mut options = Lighthouse::default_chrome_options();
options["chromePath"] = "/path/to/chrome".into();

let result = Lighthouse::url("https://example.com")?
    .with_chrome_options(options)?
    .run()?;
# let _ = result;
# Ok(())
# }
```

This replaces the entire options object. `default_chrome_options()` returns `{"chromeFlags": ["--headless"]}`. Sandboxing is not disabled automatically.

## Setting a timeout

By default, if the Lighthouse process takes more than 60 seconds it will be aborted and `Error::Timeout` will be returned. You can adjust the timeout using `timeout_in_seconds()`.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let result = Lighthouse::url("https://example.com")?
    .timeout_in_seconds(120)
    .max_wait_for_load(5000)
    .run()?;
# let _ = result;
# Ok(())
# }
```

The timeout is in seconds; `max_wait_for_load()` is in milliseconds and controls only the page-load wait. The Node timer reserves 700 ms for Chrome cleanup before the hard deadline. Timeouts must be between 1 and 2,147,483 seconds, within Node's timer range.

Chrome is killed on normal completion, errors and Node timeouts. Rust also enforces the hard deadline and reaps Node. If Node becomes unresponsive, detached Chrome processes on Unix may need external cleanup. Use an isolated worker/container when auditing untrusted URLs. This is not an SSRF filter; restrict reachable destinations in your application or network.

## Handling errors

Fallible methods return `Result<T, Error>`. Use `?` to propagate errors, or match the error when your application needs different recovery behavior:

- `Error::CouldNotRunLighthouse` means Node could not start, a dependency or configuration failed, or the runner returned invalid output.
- `Error::LighthouseReportedError { code, message }` means Lighthouse ran but could not audit the page. The code identifies failures such as an unreachable page.
- `Error::Timeout` means the overall deadline was reached.
- `Error::Io` and `Error::Json` expose filesystem and JSON failures.

The wrapper passes JSON directly to Node without a shell. URLs, headers and custom options are not interpreted as shell commands.
