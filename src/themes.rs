//! First-class themes and system appearance for 0.8 Alpha.
//!
//! Locked facts from `docs/SPEC.md`: themes are first-class and system
//! appearance is followed automatically by default. Theme token names are
//! NOT locked, so token entries are opaque data with no styling semantics
//! (see `docs/implementation-status.md`).
//!
//! Minimal assumption enabling the locked default-follow behavior: a theme
//! declares an optional `name` (display string) and an optional `mode` of
//! `system` (the default), `light`, or `dark`, written bare or quoted.
//! `mode` defaults to `System` so system appearance is followed
//! automatically unless pinned; unknown modes are Luca errors. The
//! `system`/`light`/`dark` vocabulary is the smallest appearance domain
//! consistent with platform behavior, not locked design. `Environment`
//! carries the platform signals adapters will supply (no adapter exists
//! yet); tests construct it explicitly — no defaults are invented.

use crate::{layout::check_block_entries, LucaError, Prop, PropValue};

/// Appearance a platform reports. Values beyond these two await the
/// platform adapters; nothing here names a platform or renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAppearance {
    Light,
    Dark,
}

/// Declared theme selection. `System` (the default) follows the platform;
/// `Light`/`Dark` pin the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    /// The declared mode word (`system`, `light`, or `dark`).
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    /// Resolve the selection against the reported platform appearance.
    pub fn effective(&self, system: SystemAppearance) -> SystemAppearance {
        match self {
            ThemeMode::System => system,
            ThemeMode::Light => SystemAppearance::Light,
            ThemeMode::Dark => SystemAppearance::Dark,
        }
    }
}

/// Platform signals for one render pass. Adapters supply these later;
/// callers construct them explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    pub system: SystemAppearance,
    pub reduced_motion: bool,
}

impl Environment {
    /// Declare the platform context explicitly (no invented defaults).
    pub fn new(system: SystemAppearance, reduced_motion: bool) -> Self {
        Self { system, reduced_motion }
    }
}

/// A first-class named theme attached to one component. `tokens` holds
/// opaque future style-token entries; no token names or meanings are
/// defined here.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: Option<String>,
    pub mode: ThemeMode,
    pub tokens: Vec<Prop>,
    pub line: usize,
    pub column: usize,
}

impl Theme {
    /// Validate raw entries harvested from a `theme:` block, recognizing
    /// only `name` (string) and `mode` (`system`/`light`/`dark`).
    pub(crate) fn from_entries(
        component: &str,
        entries: Vec<Prop>,
        line: usize,
        column: usize,
    ) -> Result<Self, LucaError> {
        check_block_entries("theme", component, &entries)?;
        let mut name: Option<String> = None;
        let mut mode = ThemeMode::System;
        let mut tokens = Vec::new();
        for entry in entries {
            match entry.key.as_str() {
                "name" => match &entry.value {
                    PropValue::Str(text) => name = Some(text.clone()),
                    _ => {
                        return Err(LucaError::new(
                            "Theme 'name' must be a string",
                            entry.line,
                            entry.column,
                        ));
                    }
                },
                "mode" => {
                    let text = match &entry.value {
                        PropValue::Ref(word) | PropValue::Str(word) => word.clone(),
                        _ => {
                            return Err(LucaError::new(
                                "Unknown theme mode; expected system, light, or dark",
                                entry.line,
                                entry.column,
                            ));
                        }
                    };
                    mode = match text.as_str() {
                        "system" => ThemeMode::System,
                        "light" => ThemeMode::Light,
                        "dark" => ThemeMode::Dark,
                        _ => {
                            return Err(LucaError::new(
                                format!(
                                    "Unknown theme mode '{text}'; expected system, light, or dark"
                                ),
                                entry.line,
                                entry.column,
                            ));
                        }
                    };
                }
                _ => tokens.push(entry),
            }
        }
        Ok(Self { name, mode, tokens, line, column })
    }

    /// Resolve this theme's selection against the platform appearance.
    pub fn effective(&self, system: SystemAppearance) -> SystemAppearance {
        self.mode.effective(system)
    }
}
