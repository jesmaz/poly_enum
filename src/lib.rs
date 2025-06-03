#![warn(missing_docs)]
//! Hierarchical polymorphism with enums
//!
//! This crate provides an easy way to create complex polymorphic hierarchies powered by enums.
//!
//! # Examples
//! ```
//! use poly_enum::PolyEnum;
//! #[derive(PolyEnum)]
//! enum Elements {
//!     #[poly_enum(NonMetal)]
//!     Carbon,
//!     #[poly_enum(Oxidizer, NonMetal)]
//!     Florine,
//!     #[poly_enum(Metal)]
//!     Iron,
//!     #[poly_enum(Alkali, Metal)]
//!     Lithium,
//!     #[poly_enum(Oxidizer, NonMetal)]
//!     Oxygen,
//!     #[poly_enum(Alkali, Metal)]
//!     Sodium,
//! }
//!
//! impl Alkali {
//!     fn explodes_in_water(&self) -> bool {true}
//! }
//!
//! let metal = Metal::Sodium;
//! let alkali: Alkali = metal.cast().unwrap();
//! assert!(alkali.explodes_in_water());
//! ```
//! This will generate 4 enums in addition to the base Elements enum: NonMetal, Oxidizer, Metal, and Alkali.
//!
//! # Limitations
//! - No dynamic dispatch other than the match expression
//! - Separate hierarchies are not cross compatible

use std::{rc::Rc, sync::Arc};

/// Generates a hierarchy from the provided enum. Will not work on structs or unions.
///
/// # #[poly_derive]
/// Pass through derive macros to sub-enums. These do not necessarily need to match the parent's derive attribute.
/// ```
/// use poly_enum::PolyEnum;
/// #[derive(Clone, Copy, Debug, PolyEnum)]
/// #[poly_derive(Clone, Copy)]
/// enum Value {
///     #[poly_enum(Half, Float)]
///     F32(f32),
///     #[poly_enum(Float)]
///     F64(f64),
///     #[poly_enum(Half, Int)]
///     I32(i32),
///     #[poly_enum(Int)]
///     I64(i64),
///     #[poly_enum(Int)]
///     U64(u64),
///     #[poly_enum(Pair)]
///     V2(u64, u64),
/// }
/// ```
///
/// # #[poly_enum]
/// Mark an enum variant as belonging to one or more sub-enums.
/// ```
/// use poly_enum::PolyEnum;
/// #[derive(PolyEnum)]
/// enum Value {
///     #[poly_enum(Half, Float)]
///     F32(f32),
///     #[poly_enum(Float)]
///     F64(f64),
///     #[poly_enum(Half, Int)]
///     I32(i32),
///     #[poly_enum(Int)]
///     I64(i64),
///     #[poly_enum(Int)]
///     U64(u64),
///     #[poly_enum(Pair)]
///     V2(u64, u64),
/// }
/// ```
pub use derive::PolyEnum;

/// The main trait for polymorphic enums.
/// This trait allows casting between different enums within a hierarchy.
pub trait PolyEnum<T> {
	/// Attempts to cast from one enum to another
	fn cast(self) -> Option<T>;
}

impl<T, U> PolyEnum<Arc<U>> for Arc<T> where T: Clone + PolyEnum<U> {
	fn cast(self) -> Option<Arc<U>> {
		Arc::unwrap_or_clone(self).cast().map(Arc::new)
	}
}

impl<T, U> PolyEnum<Box<U>> for Box<T> where T: PolyEnum<U> {
	fn cast(self) -> Option<Box<U>> {
		(*self).cast().map(Box::new)
	}
}

impl<T, U> PolyEnum<Rc<U>> for Rc<T> where T: Clone + PolyEnum<U> {
	fn cast(self) -> Option<Rc<U>> {
		Rc::unwrap_or_clone(self).cast().map(Rc::new)
	}
}

impl<T, U> PolyEnum<Vec<U>> for Vec<T> where T: PolyEnum<U> {
	fn cast(self) -> Option<Vec<U>> {
		self.into_iter().map(|e| e.cast()).collect::<Option<Vec<_>>>()
	}
}
