use std::{collections::BTreeMap, fs, path::Path};

use serde_json::{Map, Value, json};

use crate::{Category, Error, FormFactor};

/// The JSON and HTML reports returned by a Lighthouse run.
#[derive(Clone)]
#[must_use]
pub struct LighthouseResult {
    raw: Value,
}

impl std::fmt::Debug for LighthouseResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Reports can contain cookies, headers and private page content.
        formatter
            .debug_struct("LighthouseResult")
            .finish_non_exhaustive()
    }
}

impl Default for LighthouseResult {
    fn default() -> Self {
        Self {
            raw: json!({"report": [{}, ""], "runtimeError": null}),
        }
    }
}

impl LighthouseResult {
    /// Read the PHP-compatible raw envelope, normalizing a JSON-string report if necessary.
    ///
    /// Returns a typed runtime error or rejects a missing/invalid JSON or HTML report.
    pub fn from_raw_results(mut raw: Value) -> Result<Self, Error> {
        if let Some(error) = raw.get("runtimeError").filter(|error| !error.is_null()) {
            return Err(Error::LighthouseReportedError {
                code: error["code"].as_str().unwrap_or("UNKNOWN").into(),
                message: error["message"]
                    .as_str()
                    .unwrap_or("Unknown Lighthouse error")
                    .into(),
            });
        }
        let report = raw
            .get_mut("report")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| Error::InvalidData("missing report array".into()))?;
        if let Some(Value::String(json)) = report.first() {
            report[0] = serde_json::from_str(json)?;
        }
        if !report.first().is_some_and(Value::is_object)
            || !report.get(1).is_some_and(Value::is_string)
        {
            return Err(Error::InvalidData(
                "expected JSON object and HTML string".into(),
            ));
        }
        if let Some(error) = report[0]
            .get("runtimeError")
            .filter(|error| !error.is_null())
        {
            return Err(Error::LighthouseReportedError {
                code: error["code"].as_str().unwrap_or("UNKNOWN").into(),
                message: error["message"]
                    .as_str()
                    .unwrap_or("Unknown Lighthouse error")
                    .into(),
            });
        }
        Ok(Self { raw })
    }

    /// Construct a result from reports stored separately.
    pub fn from_reports(json: Value, html: impl Into<String>) -> Result<Self, Error> {
        Self::from_raw_results(json!({"report": [json, html.into()], "runtimeError": null}))
    }

    /// Replace the JSON report; it must be an object.
    pub fn set_json_report(self, json: Value) -> Result<Self, Error> {
        let mut raw = self.raw;
        raw["report"][0] = json;
        Self::from_raw_results(raw)
    }

    /// Replace the HTML report.
    pub fn set_html_report(mut self, html: impl Into<String>) -> Self {
        self.raw["report"][1] = html.into().into();
        self
    }

    /// All JSON report data.
    #[must_use]
    pub fn json_report(&self) -> &Value {
        &self.raw["report"][0]
    }

    /// Read a JSON report property using dot notation, including array indexes.
    #[must_use]
    pub fn get(&self, path: &str) -> Option<&Value> {
        get_path(self.json_report(), path)
    }

    /// An owned copy of the JSON report (PHP's `toArray()`).
    #[must_use]
    pub fn to_array(&self) -> Value {
        self.json_report().clone()
    }

    /// All raw runner data; may contain private page content and request headers.
    #[must_use]
    pub fn raw_results(&self) -> &Value {
        &self.raw
    }

    /// Read raw runner data with dot notation.
    #[must_use]
    pub fn raw_result(&self, path: &str) -> Option<&Value> {
        get_path(&self.raw, path)
    }

    /// The HTML report.
    #[must_use]
    pub fn html(&self) -> &str {
        // Construction validates both report slots; no public method can change their types.
        self.raw["report"][1].as_str().unwrap_or_default()
    }

    /// Write the HTML report to disk. Returns filesystem errors without suppressing them.
    pub fn save_html(&self, path: impl AsRef<Path>) -> Result<&Self, Error> {
        fs::write(path, self.html())?;
        Ok(self)
    }

    /// All configuration settings recorded in the report, if present.
    #[must_use]
    pub fn config_settings(&self) -> Option<&Value> {
        self.get("configSettings")
    }

    /// Read a configuration property with dot notation.
    #[must_use]
    pub fn config_setting(&self, path: &str) -> Option<&Value> {
        self.config_settings()
            .and_then(|settings| get_path(settings, path))
    }

    /// Whether the report records network throttling as enabled.
    #[must_use]
    pub fn network_throttling_was_enabled(&self) -> bool {
        self.config_setting("disableNetworkThrottling")
            .and_then(Value::as_bool)
            != Some(true)
    }

    /// Whether the report records CPU throttling as enabled.
    #[must_use]
    pub fn cpu_throttling_was_enabled(&self) -> bool {
        self.config_setting("disableCpuThrottling")
            .and_then(Value::as_bool)
            != Some(true)
    }

    /// The recorded browser profile, if present. Unknown values return an enum error.
    pub fn form_factor(&self) -> Result<Option<FormFactor>, Error> {
        self.config_setting("formFactor")
            .and_then(Value::as_str)
            .map(str::parse)
            .transpose()
    }

    /// The recorded emulated user agent, if present.
    #[must_use]
    pub fn user_agent(&self) -> Option<&str> {
        self.config_setting("emulatedUserAgent")
            .and_then(Value::as_str)
    }

    /// Extra request headers, if present.
    #[must_use]
    pub fn headers(&self) -> Option<&Map<String, Value>> {
        self.config_setting("extraHeaders")
            .and_then(Value::as_object)
    }

    /// One category score, truncated to a percentage; missing/null scores return `None`.
    #[must_use]
    pub fn score(&self, category: Category) -> Option<u32> {
        self.json_report()["categories"][category.as_str()]["score"]
            .as_f64()
            .map(|score| (score * 100.0) as u32)
    }

    /// Nonzero category scores, matching PHP's filtering. Use `score()` to distinguish zero.
    ///
    /// Keys are sorted lexicographically rather than preserving PHP's category order.
    #[must_use]
    pub fn scores(&self) -> BTreeMap<String, u32> {
        Category::ALL
            .into_iter()
            .filter_map(|category| {
                self.score(category)
                    .filter(|score| *score != 0)
                    .map(|score| (category.as_str().into(), score))
            })
            .collect()
    }

    /// All audit results, preserving Lighthouse's audit order, if present.
    #[must_use]
    pub fn audits(&self) -> Option<&Map<String, Value>> {
        self.get("audits").and_then(Value::as_object)
    }

    /// One audit. Returns `AuditDoesNotExist` if it is absent or null.
    pub fn audit(&self, name: &str) -> Result<&Value, Error> {
        self.audits()
            .and_then(|audits| audits.get(name))
            .filter(|audit| !audit.is_null())
            .ok_or_else(|| Error::AuditDoesNotExist(name.into()))
    }

    /// Audit identifiers, in Lighthouse's order.
    #[must_use]
    pub fn audit_names(&self) -> Vec<&str> {
        self.audits()
            .into_iter()
            .flat_map(|audits| audits.keys().map(String::as_str))
            .collect()
    }

    /// Audits whose identifiers end in `-budget`.
    #[must_use]
    pub fn budget_results(&self) -> BTreeMap<&str, &Value> {
        self.audits()
            .into_iter()
            .flat_map(|audits| audits.iter())
            .filter(|(name, _)| name.ends_with("-budget"))
            .map(|(name, value)| (name.as_str(), value))
            .collect()
    }

    /// The Lighthouse version that generated this report.
    #[must_use]
    pub fn lighthouse_version(&self) -> Option<&str> {
        self.get("lighthouseVersion").and_then(Value::as_str)
    }

    /// A rough approximation of the host device's CPU performance.
    #[must_use]
    pub fn benchmark_index(&self) -> Option<f64> {
        self.get("environment.benchmarkIndex")
            .and_then(Value::as_f64)
    }

    /// Total transferred page size in bytes; a missing audit is an error.
    pub fn total_page_size_in_bytes(&self) -> Result<Option<u64>, Error> {
        Ok(self.audit("total-byte-weight")?["numericValue"]
            .as_f64()
            .map(|value| value as u64))
    }
}

macro_rules! metric {
    ($numeric:ident, $formatted:ident, $audit:literal, $description:literal) => {
        impl LighthouseResult {
            #[doc = concat!($description, " Returns a missing-audit error, or `None` if its numeric value is absent.")]
            pub fn $numeric(&self) -> Result<Option<f64>, Error> {
                Ok(self.audit($audit)?["numericValue"].as_f64())
            }

            #[doc = concat!("Formatted ", $description, " Returns a missing-audit error, or `None` if its display value is absent.")]
            pub fn $formatted(&self) -> Result<Option<&str>, Error> {
                Ok(self.audit($audit)?["displayValue"].as_str())
            }
        }
    };
}

metric!(
    first_contentful_paint_in_ms,
    formatted_first_contentful_paint,
    "first-contentful-paint",
    "First contentful paint in milliseconds."
);
metric!(
    largest_contentful_paint_in_ms,
    formatted_largest_contentful_paint,
    "largest-contentful-paint",
    "Largest contentful paint in milliseconds."
);
metric!(
    speed_index_in_ms,
    formatted_speed_index,
    "speed-index",
    "Speed index in milliseconds."
);
metric!(
    total_blocking_time_in_ms,
    formatted_total_blocking_time,
    "total-blocking-time",
    "Total blocking time in milliseconds."
);
metric!(
    time_to_interactive_in_ms,
    formatted_time_to_interactive,
    "interactive",
    "Time to interactive in milliseconds (older Lighthouse reports)."
);
metric!(
    cumulative_layout_shift,
    formatted_cumulative_layout_shift,
    "cumulative-layout-shift",
    "Cumulative layout shift (unitless)."
);

fn get_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if let Some(value) = value.as_object().and_then(|object| object.get(path)) {
        return Some(value);
    }
    let mut current = value;
    for part in path.split('.') {
        current = match current {
            Value::Array(array) => array.get(part.parse::<usize>().ok()?)?,
            Value::Object(object) => object.get(part)?,
            _ => return None,
        };
    }
    Some(current)
}
