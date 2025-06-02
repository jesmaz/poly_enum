use std::{error::Error, fmt::Display};

pub use derive::PolyEnum;

#[derive(Clone, Copy, Debug)]
pub struct NoSuchVariantError;

pub trait PolyEnum<T> {
	fn cast(self) -> Option<T>;
}

impl Display for NoSuchVariantError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("No such variant")
	}
}

impl Error for NoSuchVariantError {}
