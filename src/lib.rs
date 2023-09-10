#![no_std]
mod constants;
pub mod ov5640;

pub use ov5640::{Format, Ov5640, RawOrder, Resolution, Rgb565Order, SccbError};

#[cfg(feature = "defmt-03")]
use defmt_03 as defmt;
