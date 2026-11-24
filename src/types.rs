use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::Error;

/// The four audit categories supported by current Lighthouse releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// Loading speed and responsiveness.
    Performance,
    /// Accessibility of the page.
    Accessibility,
    /// Web development best practices.
    BestPractices,
    /// Search engine optimization.
    Seo,
}

impl Category {
    /// All categories, in the same order as the PHP package.
    pub const ALL: [Self; 4] = [
        Self::Performance,
        Self::Accessibility,
        Self::BestPractices,
        Self::Seo,
    ];

    /// The Lighthouse category identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Performance => "performance",
            Self::Accessibility => "accessibility",
            Self::BestPractices => "best-practices",
            Self::Seo => "seo",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Category {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == value)
            .ok_or_else(|| Error::InvalidEnumValue {
                kind: "category",
                value: value.into(),
            })
    }
}

/// The browser profile to use when running audits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormFactor {
    /// A mobile profile.
    Mobile,
    /// A desktop profile (the default).
    Desktop,
}

impl FormFactor {
    /// The Lighthouse form factor identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mobile => "mobile",
            Self::Desktop => "desktop",
        }
    }
}

impl fmt::Display for FormFactor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for FormFactor {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mobile" => Ok(Self::Mobile),
            "desktop" => Ok(Self::Desktop),
            _ => Err(Error::InvalidEnumValue {
                kind: "form factor",
                value: value.into(),
            }),
        }
    }
}

/// Screen emulation settings; absent dimensions retain Lighthouse's defaults.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenEmulation {
    /// Whether to emulate a mobile screen.
    pub mobile: bool,
    /// Whether screen emulation is disabled.
    pub disabled: bool,
    /// Viewport width in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Viewport height in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Physical pixels per CSS pixel.
    #[serde(rename = "deviceScaleFactor", skip_serializing_if = "Option::is_none")]
    pub device_scale_ratio: Option<f64>,
}
