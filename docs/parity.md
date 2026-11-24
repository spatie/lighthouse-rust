---
title: PHP compatibility
weight: 7
---

The behavior reference is [spatie/lighthouse-php at 564f4c05](https://github.com/spatie/lighthouse-php/tree/564f4c05b364d3725cf0426fb22feceba33144db). The Rust API preserves its domain vocabulary, configuration defaults and result helpers. The PHP docs' older PWA and service-worker examples have been updated because current Lighthouse removed those features.

## Configuration and execution

- `Lighthouse::url()` replaces both PHP's constructor and named constructor. It returns `Result` and deliberately accepts only absolute HTTP(S) URLs. PHP's URL validator also accepts schemes that cannot sensibly be audited here. Covered in `tests/configuration.rs`.
- Setters consume and return `Self`; `run()` borrows and can be called repeatedly. There is no extra build step.
- `default_lighthouse_config()`, `default_chrome_options()` and `lighthouse_script_arguments()` expose the same defaults and argument data. The returned arguments are a named JSON object rather than the private positional Node protocol. Default settings are checked against the PHP source in `tests/configuration.rs`.
- `user_agent()`, `headers()`, `budgets()`, `categories()`, `only_audits()`, `skip_audits()`, `form_factor()`, `screen_emulation()`, both throttle methods and both disabling methods preserve their PHP settings and replacement/call-order semantics. Enums and `ScreenEmulation` replace string/optional-argument overloads; use `FromStr` for dynamic enum values. `throttle_cpu(None)` selects the default multiplier; `Some(n)` overrides it. Configuration tests check exact values and units.
- Free-form Lighthouse config, Chrome options and budgets remain JSON because those upstream schemas are extensible. `with_config()` and `with_chrome_options()` replace rather than merge. Rust validates the object shapes needed by setters; Lighthouse validates the rest. Negative shape cases are tested.
- `ScreenEmulation::device_scale_ratio` serializes to Lighthouse's actual `deviceScaleFactor` setting. PHP spells this key `deviceScaleRatio`, which Lighthouse ignores. The Rust mapping preserves the option's purpose and is checked in the live mobile test.
- `timeout_in_seconds()` defaults to 60 seconds; the Node timer reserves the PHP runner's 700 ms cleanup margin. `max_wait_for_load()` uses milliseconds. Rust rejects zero and values outside Node's timer range. The parent captures output in temporary files to avoid pipe deadlocks and enforces its own deadline. Normal/error/timeout cleanup is handled by chrome-launcher; forced termination of an unresponsive Node cannot guarantee cleanup of detached Unix Chrome processes.
- `node_binary()` and `working_directory()` are Rust additions for selecting the installed runtime and resolving local Node dependencies. No shell interpolation is used; input JSON is sent through a temporary stdin file, not command-line arguments.
- Both JSON and HTML are always returned, even with custom output selection. PHP indexes the requested report array, which breaks for reordered/single-format custom output; the Rust runner uses the authoritative `lhr` and generates HTML from it instead.
- Process failures, Lighthouse runtime errors and timeouts have distinct `Error` variants. Real runs against a disposable local page, mobile/custom-output selection, budgets, quoted headers/user agents, a closed local port, invalid config and timeout are exercised in `tests/runtime.rs`. The tests require installed Node, Lighthouse and Chrome; they are not mock-executable compatibility claims.

## Results

- `from_raw_results()` restores the PHP envelope, including JSON-string `report[0]`; `from_reports()`, `Default`, `set_json_report()` and `set_html_report()` support stored results. Malformed reports return errors rather than panicking later.
- PHP's optional accessors become explicit operations: `json_report()` / `get(path)`, `config_settings()` / `config_setting(path)`, and `raw_results()` / `raw_result(path)`. `to_array()` clones the JSON report. Dot paths support objects and array indexes; exact object keys win. Missing paths return `None`, while explicit null stays a JSON null.
- `scores()` preserves PHP's omission of null/missing/zero scores and percentage truncation; `score(Category)` preserves `Some(0)` versus `None`. Sorted `BTreeMap` keys replace PHP insertion ordering for scores and budgets. Audit order is preserved.
- All audit, formatted/numeric metric, budget, version, page-size, benchmark and configuration convenience methods are available. Missing audits return `AuditDoesNotExist`; absent fields within an existing audit return `None`. Formatted CLS stays a display string rather than PHP's float coercion; benchmark index is `f64` so fractional values are not lost. Extra headers return `None` when missing instead of an empty PHP array.
- `html()` borrows the HTML string; `save_html()` returns actual write errors. Report/builder `Debug` output is redacted. Explicit raw access and Node stderr may still expose sensitive content.
- `tests/results.rs` executes these contracts using the PHP package's original Lighthouse 9.6.8 report. It checks all metric values, four scores, 152 audits, budget selection, settings, restoration and malformed data. Independent edge cases distinguish null, zero and truncation. Live Lighthouse 13 tests separately establish runtime compatibility. Lighthouse 12 removed budget audits: configuration and stored-report APIs remain available, and a separate live Lighthouse 11.7.1 CI run verifies generating budget results. The wrapper does not invent replacements for removed upstream audits.

## Verification scope

CI executes stable checks and real Node/Lighthouse 13/Chrome integration tests on Linux, macOS and Windows. A separate Linux run uses Lighthouse 11.7.1 for budget compatibility. A Linux Rust 1.89.0 job verifies the minimum supported Rust version. Separate jobs/checks cover rustfmt, warning-free Clippy and rustdoc, recursive guide doctests, packaging and `cargo audit`.

Examples needing a live external website use `rust,no_run`: they compile but do not execute during doctests. Configuration-only examples and saved-report examples execute with hidden `# ` setup and assertions. The bundled report is clearly identified as a historical fixture. The local-page runtime tests exercise the live operations without depending on a public site's availability or stable performance scores.
