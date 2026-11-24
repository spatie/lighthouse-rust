//! Report behavior against the original PHP fixture and asymmetric edge cases.

use serde_json::{Value, json};
use spatie_lighthouse::{Category, Error, FormFactor, LighthouseResult};

fn fixture() -> LighthouseResult {
    LighthouseResult::from_reports(
        serde_json::from_str(include_str!("fixtures/example-com.json")).unwrap(),
        "<html>Report</html>",
    )
    .unwrap()
}

#[test]
fn original_category_scores_and_zero_filtering_match_php() {
    let result = fixture();
    assert_eq!(
        result.scores(),
        [
            ("performance", 100),
            ("accessibility", 92),
            ("best-practices", 100),
            ("seo", 91)
        ]
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect()
    );
    let mut json = result.to_array();
    json["categories"]["performance"]["score"] = Value::Null;
    json["categories"]["accessibility"]["score"] = json!(0);
    json["categories"]["seo"]["score"] = json!(0.927);
    let result = result.set_json_report(json).unwrap();
    assert_eq!(result.score(Category::Performance), None);
    assert_eq!(result.score(Category::Accessibility), Some(0));
    assert_eq!(
        result.scores(),
        [("best-practices".into(), 100), ("seo".into(), 92)].into()
    );
}

#[test]
fn original_audits_and_all_metric_helpers_retain_their_values() {
    let result = fixture();
    assert_eq!(result.audit_names().len(), 152);
    assert_eq!(result.audit_names()[0], "is-on-https");
    assert_eq!(
        result.audit("first-contentful-paint").unwrap()["title"],
        "First Contentful Paint"
    );
    assert_eq!(
        result.first_contentful_paint_in_ms().unwrap(),
        Some(1262.95)
    );
    assert_eq!(
        result.largest_contentful_paint_in_ms().unwrap(),
        Some(1262.951)
    );
    assert_eq!(result.speed_index_in_ms().unwrap(), Some(1258.0));
    assert_eq!(result.total_blocking_time_in_ms().unwrap(), Some(0.0));
    assert_eq!(
        result.time_to_interactive_in_ms().unwrap(),
        Some(1262.9499999389648)
    );
    assert_eq!(result.cumulative_layout_shift().unwrap(), Some(0.0));
    assert_eq!(
        result.formatted_first_contentful_paint().unwrap(),
        Some("1.3\u{a0}s")
    );
    assert_eq!(
        result.formatted_largest_contentful_paint().unwrap(),
        Some("1.3\u{a0}s")
    );
    assert_eq!(result.formatted_speed_index().unwrap(), Some("1.3\u{a0}s"));
    assert_eq!(
        result.formatted_total_blocking_time().unwrap(),
        Some("0\u{a0}ms")
    );
    assert_eq!(
        result.formatted_time_to_interactive().unwrap(),
        Some("1.3\u{a0}s")
    );
    assert_eq!(
        result.formatted_cumulative_layout_shift().unwrap(),
        Some("0")
    );
    assert_eq!(result.total_page_size_in_bytes().unwrap(), Some(850));
    assert_eq!(result.benchmark_index(), Some(2670.0));
    assert_eq!(result.lighthouse_version(), Some("9.6.8"));
    assert_eq!(
        result.budget_results().keys().copied().collect::<Vec<_>>(),
        ["performance-budget", "timing-budget"]
    );
}

#[test]
fn recorded_configuration_and_dot_paths_are_available() {
    let result = fixture();
    assert_eq!(result.form_factor().unwrap(), Some(FormFactor::Desktop));
    assert!(!result.network_throttling_was_enabled());
    assert!(!result.cpu_throttling_was_enabled());
    assert!(result.user_agent().unwrap().contains("Chrome-Lighthouse"));
    assert_eq!(
        result.config_setting("screenEmulation.mobile"),
        Some(&json!(true))
    );
    assert_eq!(
        result.get("categories.accessibility.score"),
        Some(&json!(0.92))
    );
    assert_eq!(
        result.raw_result("report.0.lighthouseVersion"),
        Some(&json!("9.6.8"))
    );
    assert_eq!(result.raw_result("report.4"), None);
    assert_eq!(result.get("missing.value"), None);
}

#[test]
fn reports_can_be_restored_from_json_strings_and_html_saved_exactly() {
    let original = fixture();
    let result = LighthouseResult::from_raw_results(
        json!({"report": [original.json_report().to_string(), original.html()], "runtimeError": null}),
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("report with 'quotes'.html");
    result.save_html(&path).unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "<html>Report</html>"
    );
    assert_eq!(result.json_report(), original.json_report());
    let restored = LighthouseResult::default()
        .set_html_report(result.html())
        .set_json_report(result.to_array())
        .unwrap();
    assert_eq!(restored.raw_results(), result.raw_results());
    assert!(result.save_html(directory.path()).is_err());
}

#[test]
fn malformed_reports_and_runtime_errors_remain_distinct() {
    for raw in [
        json!({}),
        json!({"report": []}),
        json!({"report": [{}, null]}),
        json!({"report": [[], "html"]}),
    ] {
        assert!(matches!(
            LighthouseResult::from_raw_results(raw),
            Err(Error::InvalidData(_))
        ));
    }
    let error = LighthouseResult::from_raw_results(
        json!({"runtimeError": {"code": "NO_FCP", "message": "No paint"}}),
    )
    .unwrap_err();
    assert!(
        matches!(error, Error::LighthouseReportedError {code, message} if code == "NO_FCP" && message == "No paint")
    );
    let result = LighthouseResult::from_reports(
        json!({"audits": {"first-contentful-paint": {"numericValue": null}}}),
        "",
    )
    .unwrap();
    assert_eq!(result.first_contentful_paint_in_ms().unwrap(), None);
    assert!(
        matches!(result.time_to_interactive_in_ms(), Err(Error::AuditDoesNotExist(name)) if name == "interactive")
    );
}
