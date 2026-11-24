use std::{
    ffi::{OsStr, OsString},
    fmt,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use command_group::CommandGroup;
use serde_json::{Value, json};

use crate::{Category, Error, FormFactor, LighthouseResult, ScreenEmulation};

/// Configure and run an installed Node.js Lighthouse against a URL.
///
/// Builders consume and return `Self`. `run()` borrows the configuration so it can be reused.
#[derive(Clone)]
#[must_use]
pub struct Lighthouse {
    url: String,
    config: Value,
    chrome_options: Value,
    timeout: Duration,
    max_wait_for_load: Option<u64>,
    node_binary: OsString,
    working_directory: Option<PathBuf>,
}

impl fmt::Debug for Lighthouse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // URLs, headers and custom options may contain credentials.
        formatter
            .debug_struct("Lighthouse")
            .field("timeout", &self.timeout)
            .field("max_wait_for_load", &self.max_wait_for_load)
            .finish_non_exhaustive()
    }
}

impl Lighthouse {
    /// Start configuring an absolute HTTP(S) URL.
    ///
    /// Returns `InvalidUrl` for relative URLs, unsupported schemes or a missing host.
    pub fn url(url: impl Into<String>) -> Result<Self, Error> {
        let url = url.into();
        let parsed = url::Url::parse(&url).map_err(|_| Error::InvalidUrl(url.clone()))?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || !url.contains("://")
            || url.chars().any(char::is_whitespace)
        {
            return Err(Error::InvalidUrl(url));
        }
        Ok(Self {
            url,
            config: Self::default_lighthouse_config(),
            chrome_options: Self::default_chrome_options(),
            timeout: Duration::from_secs(60),
            max_wait_for_load: None,
            node_binary: "node".into(),
            working_directory: None,
        })
    }

    /// Default PHP-compatible Lighthouse settings: desktop, four categories, no throttling.
    #[must_use]
    pub fn default_lighthouse_config() -> Value {
        json!({
            "extends": "lighthouse:default",
            "settings": {
                "onlyCategories": Category::ALL,
                "formFactor": "desktop",
                "output": ["json", "html"],
                "disableNetworkThrottling": true,
                "disableCpuThrottling": true,
                "throttlingMethod": "provided",
                "screenEmulation": {"disabled": true}
            }
        })
    }

    /// Default Chrome launch options. Sandboxing is not disabled automatically.
    #[must_use]
    pub fn default_chrome_options() -> Value {
        json!({"chromeFlags": ["--headless"]})
    }

    /// Emulate a custom user agent.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config["settings"]["emulatedUserAgent"] = user_agent.into().into();
        self
    }

    /// Replace extra headers sent with page requests.
    pub fn headers<K, V>(mut self, headers: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.config["settings"]["extraHeaders"] = headers
            .into_iter()
            .map(|(key, value)| (key.into(), Value::String(value.into())))
            .collect();
        self
    }

    /// Set Lighthouse performance budgets using its JSON budget schema.
    pub fn budgets(mut self, budgets: Vec<Value>) -> Self {
        self.config["settings"]["budgets"] = budgets.into();
        self
    }

    /// Run only the selected categories. Call after `only_audits()` to select both.
    pub fn categories(mut self, categories: impl IntoIterator<Item = Category>) -> Self {
        self.config["settings"]["onlyCategories"] = categories
            .into_iter()
            .map(|category| Value::String(category.as_str().into()))
            .collect();
        self
    }

    /// Run specific audits, removing any previous category selection.
    pub fn only_audits(mut self, audits: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config["settings"]["onlyAudits"] = audits
            .into_iter()
            .map(|audit| Value::String(audit.into()))
            .collect();
        if let Some(settings) = self.config["settings"].as_object_mut() {
            settings.remove("onlyCategories");
        }
        self
    }

    /// Skip the named audits. Lighthouse validates combinations with `only_audits()`.
    pub fn skip_audits(mut self, audits: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config["settings"]["skipAudits"] = audits
            .into_iter()
            .map(|audit| Value::String(audit.into()))
            .collect();
        self
    }

    /// Set the browser profile; screen emulation is configured separately.
    pub fn form_factor(mut self, form_factor: FormFactor) -> Self {
        self.config["settings"]["formFactor"] = form_factor.as_str().into();
        self
    }

    /// Replace screen emulation settings.
    pub fn screen_emulation(mut self, screen: ScreenEmulation) -> Self {
        self.config["settings"]["screenEmulation"] = json!(screen);
        self
    }

    /// Enable network throttling without changing the current throttling method.
    pub fn throttle_network(mut self) -> Self {
        self.config["settings"]["disableNetworkThrottling"] = false.into();
        self
    }

    /// Disable network throttling without changing other settings.
    pub fn do_not_throttle_network(mut self) -> Self {
        self.config["settings"]["disableNetworkThrottling"] = true.into();
        self
    }

    /// Enable simulated CPU throttling, optionally overriding the slowdown multiplier.
    pub fn throttle_cpu(mut self, multiplier: Option<u32>) -> Self {
        self.config["settings"]["disableCpuThrottling"] = false.into();
        self.config["settings"]["throttlingMethod"] = "simulate".into();
        if let Some(multiplier) = multiplier {
            self.config["settings"]["throttling"]["cpuSlowdownMultiplier"] = multiplier.into();
        }
        self
    }

    /// Disable CPU throttling without resetting the throttling method.
    pub fn do_not_throttle_cpu(mut self) -> Self {
        self.config["settings"]["disableCpuThrottling"] = true.into();
        self
    }

    /// Replace the entire Lighthouse configuration, not just its settings.
    ///
    /// Validates the object structure needed by fluent setters. Lighthouse validates its schema.
    pub fn with_config(mut self, config: Value) -> Result<Self, Error> {
        if !config.is_object()
            || config
                .get("settings")
                .is_some_and(|settings| !settings.is_object())
            || config
                .get("settings")
                .and_then(|settings| settings.get("throttling"))
                .is_some_and(|throttling| !throttling.is_object() && !throttling.is_null())
        {
            return Err(Error::InvalidData(
                "config, settings and throttling must be objects".into(),
            ));
        }
        self.config = config;
        Ok(self)
    }

    /// Replace the entire chrome-launcher options object.
    pub fn with_chrome_options(mut self, options: Value) -> Result<Self, Error> {
        if !options.is_object() {
            return Err(Error::InvalidData(
                "Chrome options must be an object".into(),
            ));
        }
        self.chrome_options = options;
        Ok(self)
    }

    /// Set the overall timeout in seconds. Zero is rejected by `run()`.
    pub fn timeout_in_seconds(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    /// Set Lighthouse's maximum page-load wait, in milliseconds.
    pub fn max_wait_for_load(mut self, milliseconds: u64) -> Self {
        self.max_wait_for_load = Some(milliseconds);
        self
    }

    /// Use a particular Node executable instead of `node` on PATH.
    pub fn node_binary(mut self, binary: impl AsRef<OsStr>) -> Self {
        self.node_binary = binary.as_ref().to_owned();
        self
    }

    /// Resolve installed Node modules and relative Chrome paths from this directory.
    pub fn working_directory(mut self, directory: impl AsRef<Path>) -> Self {
        self.working_directory = Some(directory.as_ref().to_owned());
        self
    }

    /// The arguments sent to the bundled Node runner (may contain credentials).
    #[must_use]
    pub fn lighthouse_script_arguments(&self) -> Value {
        json!({
            "url": self.url,
            "chromeOptions": self.chrome_options,
            "lighthouseConfig": self.config,
            "timeout": self.timeout.as_millis().saturating_sub(700).max(1),
            "maxWaitForLoad": self.max_wait_for_load
        })
    }

    /// Run Lighthouse and return both its JSON and HTML reports.
    ///
    /// Requires Node, locally installed `lighthouse` and `chrome-launcher`, and Chrome.
    /// Returns typed process, timeout and Lighthouse runtime errors. Node's timer attempts
    /// Chrome cleanup before the hard deadline; an unresponsive detached Chrome may require
    /// external cleanup if Node has to be forcibly terminated.
    pub fn run(&self) -> Result<LighthouseResult, Error> {
        if self.timeout.is_zero() || self.timeout.as_millis() > i32::MAX as u128 {
            return Err(Error::InvalidData(
                "timeout must be 1..=2147483 seconds".into(),
            ));
        }
        let mut input = tempfile::tempfile()?;
        serde_json::to_writer(&mut input, &self.lighthouse_script_arguments())?;
        input.flush()?;
        input.rewind()?;
        let mut stdout = tempfile::tempfile()?;
        let mut stderr = tempfile::tempfile()?;
        let mut command = Command::new(&self.node_binary);
        command
            .args([
                "--input-type=module",
                "--eval",
                include_str!("lighthouse.mjs"),
            ])
            .stdin(Stdio::from(input))
            .stdout(stdout.try_clone()?)
            .stderr(stderr.try_clone()?);
        if let Some(directory) = &self.working_directory {
            command.current_dir(directory);
        }
        let mut child = command.group_spawn().map_err(|error| {
            Error::CouldNotRunLighthouse(format!("could not start Node: {error}"))
        })?;
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() < self.timeout => {
                    thread::sleep(Duration::from_millis(10));
                }
                outcome => {
                    let killed = child.kill();
                    let reaped = child.wait();
                    killed?;
                    reaped?;
                    return match outcome {
                        Err(error) => Err(error.into()),
                        _ => Err(Error::Timeout(self.timeout)),
                    };
                }
            }
        };
        if status.code() == Some(124) {
            return Err(Error::Timeout(self.timeout));
        }
        if !status.success() {
            stderr.seek(SeekFrom::Start(0))?;
            let mut message = String::new();
            stderr.read_to_string(&mut message)?;
            return Err(Error::CouldNotRunLighthouse(message));
        }
        stdout.rewind()?;
        let raw = serde_json::from_reader(&mut stdout).map_err(|error| {
            Error::CouldNotRunLighthouse(format!("invalid runner output: {error}"))
        })?;
        LighthouseResult::from_raw_results(raw)
    }
}
