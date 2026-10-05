//! Utility functions and structs, including persistence diagrams and matrix
//! anti-transposition.

// Keep the adapter and stripping implementation private to this crate.
#[cfg(all(feature = "python-module", not(any(doc, rust_analyzer))))]
pub(crate) use macro_rules_attribute::apply as macro_rules_apply;

// This macro removes inline Rust docstring comments (that begin with ///).
// Insert the macro before pyclass, pymethods, or pymodule directives to remove
// Rust docstrings, and also add a Python docstring using `#[doc =
// python_doc!(...)]`. That way PyO3 picks up the Python docstring but not the
// Rust docstring. This macro recurses through brace groups to also remove
// documentation on fields and methods. The stack rebuilds complete items
// instead of emitting macros in field positions.
#[cfg(all(feature = "python-module", not(any(doc, rust_analyzer))))]
macro_rules! strip_rust_docs {
	(@scan ($($out:tt)*) ()) => { $($out)* };
	(@scan ($($out:tt)*) (($($outer:tt)*) ($($rest:tt)*) $($stack:tt)*)) => {
		$crate::utils::strip_rust_docs!(@scan ($($outer)* { $($out)* }) ($($stack)*) $($rest)*);
	};
	(@scan ($($out:tt)*) ($($stack:tt)*) #[doc = $doc:literal] $($rest:tt)*) => {
		$crate::utils::strip_rust_docs!(@scan ($($out)*) ($($stack)*) $($rest)*);
	};
	(@scan ($($out:tt)*) ($($stack:tt)*) { $($inner:tt)* } $($rest:tt)*) => {
		$crate::utils::strip_rust_docs!(@scan () (($($out)*) ($($rest)*) $($stack)*) $($inner)*);
	};
	(@scan ($($out:tt)*) ($($stack:tt)*) $next:tt $($rest:tt)*) => {
		$crate::utils::strip_rust_docs!(@scan ($($out)* $next) ($($stack)*) $($rest)*);
	};
	($($item:tt)*) => { $crate::utils::strip_rust_docs!(@scan () () $($item)*); };
}

#[cfg(all(feature = "python-module", not(any(doc, rust_analyzer))))]
pub(crate) use strip_rust_docs;

// Rust documentation uses ordinary doc comments. Python documentation stays
// in raw strings so rustfmt preserves reStructuredText indentation.
#[cfg(any(doc, rust_analyzer, not(feature = "python-module")))]
macro_rules! python_doc {
	($doc:literal) => {
		""
	};
}

#[cfg(all(feature = "python-module", not(any(doc, rust_analyzer))))]
macro_rules! python_doc {
	($doc:literal) => {
		$doc
	};
}

pub(crate) use python_doc;

mod anti_transpose;
mod diagram;
#[cfg(feature = "serde")]
mod file_format;

pub use anti_transpose::anti_transpose;
pub use diagram::{ExtendedUsize, PersistenceDiagram};
#[cfg(feature = "serde")]
pub use file_format::{
	DecompositionFileFormat,
	clone_to_file_format,
	clone_to_veccolumn,
	serialize_algo,
};

use crate::columns::{Column, ColumnMode};

/// Helper function to set mode of both columns
pub(crate) fn set_mode_of_pair<C: Column>(column_pair: &mut (C, Option<C>), mode: ColumnMode) {
	column_pair.0.set_mode(mode);
	if let Some(c) = column_pair.1.as_mut() {
		c.set_mode(mode);
	}
}
