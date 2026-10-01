//! Localization foundations for 0.8 Alpha.
//!
//! Locked facts from `docs/SPEC.md`: localization support is built in.
//! Resource formats are NOT locked, so this module defines no file format
//! and no `.lucu` resource syntax — tables are built programmatically and
//! `.lucu` linkage awaits the resource-format design record
//! (see `docs/implementation-status.md`).
//!
//! Minimal foundation: `Resources` maps locales to key/text tables with
//! exact-locale lookup. Missing keys and unknown locales return `None`
//! (the caller decides presentation); no fallback chains are defined here.

use std::collections::HashMap;

/// Built-in localization resource tables: locale → key → text.
#[derive(Debug, Clone, Default)]
pub struct Resources {
    default_locale: String,
    tables: HashMap<String, HashMap<String, String>>,
}

impl Resources {
    /// Create empty resources with an explicit default locale.
    pub fn new(default_locale: &str) -> Self {
        Self { default_locale: default_locale.to_owned(), tables: HashMap::new() }
    }

    /// The default locale chosen at construction.
    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }

    /// Add one string to one locale's table (overwrites the same key).
    pub fn add(&mut self, locale: &str, key: &str, text: &str) {
        self.tables
            .entry(locale.to_owned())
            .or_default()
            .insert(key.to_owned(), text.to_owned());
    }

    /// Locales with at least one string, sorted.
    pub fn locales(&self) -> Vec<String> {
        let mut locales: Vec<String> = self.tables.keys().cloned().collect();
        locales.sort();
        locales
    }

    /// Exact-locale lookup: `Some` text when the locale table holds the key,
    /// else `None`. No fallback is applied here.
    pub fn lookup(&self, locale: &str, key: &str) -> Option<&str> {
        self.tables.get(locale)?.get(key).map(String::as_str)
    }
}
