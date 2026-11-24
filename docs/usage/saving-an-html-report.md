---
title: Saving an HTML report
weight: 4
---

Lighthouse can generate a nicely formatted HTML report with the scores and results of all audits.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
Lighthouse::url("https://example.com")?
    .run()?
    .save_html("report.html")?;
# Ok(())
# }
```

This is how that report looks in the browser. The screenshot comes from the original PHP documentation; current Lighthouse versions may render it differently.

![Lighthouse HTML report](https://spatie.be/docs/lighthouse-php/v2/images/report.jpg)

To get the HTML without saving it to a file, call `html()`.

```rust,no_run
# use spatie_lighthouse::Lighthouse;
# fn main() -> Result<(), spatie_lighthouse::Error> {
let result = Lighthouse::url("https://example.com")?.run()?;
let html = result.html();
# let _ = html;
# Ok(())
# }
```

`html()` borrows the result's HTML string. `save_html()` overwrites the file and returns a filesystem error if it cannot write it. It does not create missing parent directories.
