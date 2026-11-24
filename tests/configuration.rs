//! Public configuration and validation contracts.

use serde_json::json;
use spatie_lighthouse::{Category, Error, FormFactor, Lighthouse, ScreenEmulation};

#[test]
fn defaults_match_the_php_package() {
    assert_eq!(
        Lighthouse::url("https://example.com")
            .unwrap()
            .lighthouse_script_arguments(),
        json!({
            "url": "https://example.com",
            "chromeOptions": {"chromeFlags": ["--headless"]},
            "lighthouseConfig": {
                "extends": "lighthouse:default",
                "settings": {
                    "onlyCategories": ["performance", "accessibility", "best-practices", "seo"],
                    "formFactor": "desktop", "output": ["json", "html"],
                    "disableNetworkThrottling": true, "disableCpuThrottling": true,
                    "throttlingMethod": "provided", "screenEmulation": {"disabled": true}
                }
            },
            "timeout": 59300, "maxWaitForLoad": null
        })
    );
}

#[test]
fn custom_configuration_preserves_values_and_units() {
    let arguments = Lighthouse::url("https://example.com")
        .unwrap()
        .user_agent("My user agent")
        .headers([("MyExtraHeader", "A 'quoted' $value; with spaces")])
        .budgets(vec![
            json!({"timings": [{"metric": "interactive", "budget": 5000}]}),
        ])
        .form_factor(FormFactor::Mobile)
        .screen_emulation(ScreenEmulation {
            mobile: true,
            width: Some(390),
            height: Some(844),
            device_scale_ratio: Some(2.5),
            ..Default::default()
        })
        .throttle_cpu(Some(7))
        .throttle_network()
        .timeout_in_seconds(120)
        .max_wait_for_load(5000)
        .lighthouse_script_arguments();
    let settings = &arguments["lighthouseConfig"]["settings"];
    assert_eq!(arguments["timeout"], 119300);
    assert_eq!(arguments["maxWaitForLoad"], 5000);
    assert_eq!(settings["emulatedUserAgent"], "My user agent");
    assert_eq!(
        settings["extraHeaders"]["MyExtraHeader"],
        "A 'quoted' $value; with spaces"
    );
    assert_eq!(settings["budgets"][0]["timings"][0]["budget"], 5000);
    assert_eq!(settings["formFactor"], "mobile");
    assert_eq!(
        settings["screenEmulation"],
        json!({"mobile": true, "disabled": false, "width": 390, "height": 844, "deviceScaleFactor": 2.5})
    );
    assert_eq!(settings["throttling"]["cpuSlowdownMultiplier"], 7);
    assert_eq!(settings["throttlingMethod"], "simulate");
    assert_eq!(settings["disableCpuThrottling"], false);
    assert_eq!(settings["disableNetworkThrottling"], false);
}

#[test]
fn audit_and_category_selections_follow_call_order() {
    let run = Lighthouse::url("https://example.com")
        .unwrap()
        .categories([Category::Accessibility])
        .only_audits(["is-on-https", "first-contentful-paint"]);
    let arguments = run.lighthouse_script_arguments();
    assert!(
        arguments["lighthouseConfig"]["settings"]
            .get("onlyCategories")
            .is_none()
    );
    let arguments = run
        .categories([Category::Seo])
        .skip_audits(["unused-javascript"])
        .lighthouse_script_arguments();
    assert_eq!(
        arguments["lighthouseConfig"]["settings"]["onlyCategories"],
        json!(["seo"])
    );
    assert_eq!(
        arguments["lighthouseConfig"]["settings"]["onlyAudits"],
        json!(["is-on-https", "first-contentful-paint"])
    );
    assert_eq!(
        arguments["lighthouseConfig"]["settings"]["skipAudits"],
        json!(["unused-javascript"])
    );
}

#[test]
fn replacement_options_and_throttling_toggles_do_not_merge_or_reset_other_settings() {
    let arguments = Lighthouse::url("https://example.com")
        .unwrap()
        .with_config(json!({"settings": {}}))
        .unwrap()
        .with_chrome_options(json!({"chromePath": "/custom/chrome"}))
        .unwrap()
        .throttle_cpu(Some(3))
        .do_not_throttle_cpu()
        .throttle_network()
        .do_not_throttle_network()
        .lighthouse_script_arguments();
    assert_eq!(
        arguments["lighthouseConfig"],
        json!({"settings": {
            "disableCpuThrottling": true, "throttlingMethod": "simulate",
            "throttling": {"cpuSlowdownMultiplier": 3}, "disableNetworkThrottling": true
        }})
    );
    assert_eq!(
        arguments["chromeOptions"],
        json!({"chromePath": "/custom/chrome"})
    );
}

#[test]
fn invalid_inputs_return_typed_errors_instead_of_panicking() {
    for url in [
        "invalid-url",
        "file:///etc/passwd",
        "http:example.com",
        "https://exa mple.com",
        "https://example.com\n",
    ] {
        assert!(
            matches!(Lighthouse::url(url), Err(Error::InvalidUrl(_))),
            "{url}"
        );
    }
    assert_eq!(
        "best-practices".parse::<Category>().unwrap(),
        Category::BestPractices
    );
    assert_eq!("mobile".parse::<FormFactor>().unwrap(), FormFactor::Mobile);
    assert!("pwa".parse::<Category>().is_err());
    assert!("tablet".parse::<FormFactor>().is_err());
    for config in [
        json!([]),
        json!({"settings": []}),
        json!({"settings": {"throttling": "slow"}}),
    ] {
        assert!(
            Lighthouse::url("https://example.com")
                .unwrap()
                .with_config(config)
                .is_err()
        );
    }
    assert!(
        Lighthouse::url("https://example.com")
            .unwrap()
            .with_chrome_options(json!([]))
            .is_err()
    );
    assert!(matches!(
        Lighthouse::url("https://example.com")
            .unwrap()
            .timeout_in_seconds(0)
            .run(),
        Err(Error::InvalidData(_))
    ));
}

#[test]
fn debug_does_not_disclose_urls_headers_or_options() {
    let run = Lighthouse::url("https://example.com?secret=token")
        .unwrap()
        .headers([("Authorization", "private-token")]);
    let debug = format!("{run:?}");
    assert!(!debug.contains("token"));
    assert!(!debug.contains("example.com"));
}
