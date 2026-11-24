#![doc = include_str!("../README.md")]

mod error;
mod lighthouse;
mod result;
mod types;

pub use error::Error;
pub use lighthouse::Lighthouse;
pub use result::LighthouseResult;
pub use types::{Category, FormFactor, ScreenEmulation};

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
