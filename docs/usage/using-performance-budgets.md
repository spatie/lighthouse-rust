---
title: Using performance budgets
weight: 5
---

Lighthouse 11 has an interesting feature where you can specify so-called budgets. You can, for example, specify the maximum amount of time for the time-to-interactive metric, or the maximum amount (measured in KB) of JavaScript your page may contain.

Lighthouse 12 removed this feature. To run these audits, install `lighthouse@11.7.1` instead of the current version. The wrapper does not recreate removed upstream audits. Prefer current Lighthouse when you do not need legacy budgets.

You can learn more about this feature in [Use Lighthouse for performance budgets at web.dev](https://web.dev/articles/use-lighthouse-for-performance-budgets).

You can pass your performance budget as a list of JSON objects to `budgets()`.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let url = "https://example.com";
let result = Lighthouse::url(url)?
    .budgets(vec![serde_json::json!({
        "timings": [
            {"metric": "interactive", "budget": 5000}
        ],
        "resourceSizes": [
            {"resourceType": "script", "budget": 170}
        ]
    })])
    .run()?;
# let _ = result;
# Ok(())
# }
```

Budget keys and units follow Lighthouse 11's JSON schema. Time to interactive is no longer scored in the performance category, but remains available to its budget audit.

In the [HTML report](/docs/lighthouse-rust/v1/usage/saving-an-html-report), you can see the results of your budget. The screenshot below comes from the original PHP documentation.

![Lighthouse budget report](https://spatie.be/docs/lighthouse-php/v2/images/budget.jpg)

You can get the budget audit results by calling `budget_results()`.

```rust
# use spatie_lighthouse::LighthouseResult;
# fn main() -> Result<(), spatie_lighthouse::Error> {
# let result = LighthouseResult::from_reports(serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/example-com.json")))?, "")?;
let budgets = result.budget_results();
# assert!(budgets.contains_key("performance-budget"));
# assert!(budgets.contains_key("timing-budget"));
# Ok(())
# }
```

The map includes every audit whose identifier ends with `-budget`, typically `performance-budget` and `timing-budget`. Each value retains Lighthouse's JSON fields, including `details`, `headings` and `items`. It is empty when the report contains no budget audits.
