# Report fixture

`example-com.json` is the unmodified `report[0]` object from [spatie/lighthouse-php's example-com-result.json](https://github.com/spatie/lighthouse-php/blob/564f4c05b364d3725cf0426fb22feceba33144db/tests/stubs/example-com-result.json), licensed under the MIT license. It is an example.com report from Lighthouse 9.6.8, not a current benchmark.

Extraction: `jq '.report[0]' example-com-result.json > example-com.json`.

It preserves the original measurements for PHP-compatible result tests and executable examples. Real-runtime integration tests separately audit a disposable local page with the installed Lighthouse and Chrome.
