//! Rendering backends.
//!
//! The text backend lives in [`crate::rendering`]; visual backends live here.
//! Backends depend on the platform-independent model only and must never be
//! depended on by Luca Code (the seam stays one-way).

pub mod svg;

pub use svg::SvgBackend;
