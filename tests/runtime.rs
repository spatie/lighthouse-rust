//! Real Node/Lighthouse/Chrome checks against a disposable local HTTP server.

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use serde_json::json;
use spatie_lighthouse::{Category, Error, FormFactor, Lighthouse, ScreenEmulation};

static RUNTIME: Mutex<()> = Mutex::new(());

struct Page {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Page {
    fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stopping = stop.clone();
        let captured = requests.clone();
        let worker = thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_millis(100)))
                            .unwrap();
                        let mut request = [0; 8192];
                        if let Ok(length) = stream.read(&mut request) {
                            if length == 0 {
                                continue;
                            }
                            captured
                                .lock()
                                .unwrap()
                                .push(String::from_utf8_lossy(&request[..length]).to_lowercase());
                            let body = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><meta name=\"description\" content=\"A local Lighthouse test page\"><title>Lighthouse test page</title><link rel=\"icon\" href=\"data:,\"></head><body><main><h1>Lighthouse test page</h1><p>This page is served locally for integration testing.</p></main></body></html>";
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                                body.len()
                            );
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(error) => panic!("local test server: {error}"),
                }
            }
        });
        Self {
            url,
            requests,
            stop,
            worker: Some(worker),
        }
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().unwrap().join().unwrap();
    }
}

fn run(url: &str) -> Lighthouse {
    Lighthouse::url(url)
        .unwrap()
        .working_directory(
            std::env::var_os("LIGHTHOUSE_TEST_DIRECTORY")
                .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into()),
        )
        .with_chrome_options(
            json!({"chromeFlags": ["--headless", "--no-sandbox", "--disable-dev-shm-usage"]}),
        )
        .unwrap()
        .timeout_in_seconds(90)
}

#[test]
fn real_audit_returns_scores_metrics_budgets_headers_and_html() {
    let _guard = RUNTIME.lock().unwrap_or_else(|error| error.into_inner());
    let page = Page::serve();
    let result = run(&page.url)
        .user_agent("Spatie-Lighthouse-Test/1.0")
        .headers([("X-Spatie-Test", "A 'quoted' $value; with spaces")])
        .budgets(vec![json!({"resourceSizes": [{"resourceType": "total", "budget": 1000}], "timings": [{"metric": "first-contentful-paint", "budget": 5000}]})])
        .max_wait_for_load(5000)
        .run().unwrap();
    // Short load deadlines can leave performance unscored (NO_TTI_CPU_IDLE_PERIOD).
    // The report must still include all categories and the available scores.
    assert_eq!(
        result.get("categories").unwrap().as_object().unwrap().len(),
        4
    );
    for category in [
        Category::Accessibility,
        Category::BestPractices,
        Category::Seo,
    ] {
        assert!(result.score(category).is_some(), "{category}");
    }
    assert_eq!(result.score(Category::Accessibility), Some(100));
    assert_eq!(result.form_factor().unwrap(), Some(FormFactor::Desktop));
    assert!(!result.network_throttling_was_enabled());
    assert!(!result.cpu_throttling_was_enabled());
    assert!(result.first_contentful_paint_in_ms().unwrap().unwrap() > 0.0);
    assert!(result.total_page_size_in_bytes().unwrap().unwrap() > 0);
    assert!(result.benchmark_index().unwrap() > 0.0);
    if result.lighthouse_version().unwrap().starts_with("11.") {
        assert_eq!(
            result.budget_results().keys().copied().collect::<Vec<_>>(),
            ["performance-budget", "timing-budget"]
        );
        assert!(result.audit("performance-budget").unwrap()["details"].is_object());
        assert!(result.audit("timing-budget").unwrap()["details"].is_object());
    } else {
        // Lighthouse 12 removed budgets; the wrapper must not invent audit results.
        assert!(result.budget_results().is_empty());
    }
    assert_eq!(
        result.headers().unwrap()["X-Spatie-Test"],
        "A 'quoted' $value; with spaces"
    );
    assert_eq!(result.user_agent(), Some("Spatie-Lighthouse-Test/1.0"));
    assert!(result.html().contains("Lighthouse"));
    assert!(result.html().contains("Lighthouse test page"));
    let requests = page.requests.lock().unwrap();
    assert!(requests.iter().any(|request| {
        request.contains("x-spatie-test: a 'quoted' $value; with spaces")
            && request.contains("user-agent: spatie-lighthouse-test/1.0")
    }));
}

#[test]
fn custom_output_and_mobile_audit_selection_still_produce_both_report_formats() {
    let _guard = RUNTIME.lock().unwrap_or_else(|error| error.into_inner());
    let page = Page::serve();
    let mut config = Lighthouse::default_lighthouse_config();
    config["settings"]["output"] = json!(["html"]);
    let result = run(&page.url)
        .with_config(config)
        .unwrap()
        .form_factor(FormFactor::Mobile)
        .screen_emulation(ScreenEmulation {
            mobile: true,
            width: Some(390),
            height: Some(844),
            device_scale_ratio: Some(2.0),
            ..Default::default()
        })
        .only_audits(["document-title", "is-on-https"])
        .run()
        .unwrap();
    assert_eq!(result.form_factor().unwrap(), Some(FormFactor::Mobile));
    assert_eq!(
        result.config_setting("screenEmulation.width"),
        Some(&json!(390))
    );
    assert_eq!(
        result
            .config_setting("screenEmulation.deviceScaleFactor")
            .and_then(serde_json::Value::as_f64),
        Some(2.0)
    );
    assert_eq!(result.audit("document-title").unwrap()["score"], 1);
    assert!(matches!(
        result.audit("first-contentful-paint"),
        Err(Error::AuditDoesNotExist(_))
    ));
    assert!(result.html().contains("<!doctype html>"));
    assert!(result.html().contains("<html lang="));
}

#[test]
fn process_failure_runtime_failure_and_timeout_are_distinct() {
    let _guard = RUNTIME.lock().unwrap_or_else(|error| error.into_inner());
    let page = Page::serve();
    let missing = run(&page.url)
        .node_binary("spatie-node-that-does-not-exist")
        .run()
        .unwrap_err();
    assert!(matches!(missing, Error::CouldNotRunLighthouse(_)));
    let invalid_config = run(&page.url)
        .with_config(json!({"extends": "does-not-exist"}))
        .unwrap()
        .run()
        .unwrap_err();
    assert!(
        matches!(invalid_config, Error::CouldNotRunLighthouse(_)),
        "{invalid_config}"
    );
    let closed = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", closed.local_addr().unwrap());
    drop(closed);
    let runtime_error = run(&url).max_wait_for_load(1000).run().unwrap_err();
    assert!(
        matches!(runtime_error, Error::LighthouseReportedError { .. }),
        "{runtime_error}"
    );
    let started = Instant::now();
    let timeout = run(&page.url).timeout_in_seconds(1).run().unwrap_err();
    assert!(matches!(timeout, Error::Timeout(_)), "{timeout}");
    assert!(started.elapsed() < Duration::from_secs(5));
}
