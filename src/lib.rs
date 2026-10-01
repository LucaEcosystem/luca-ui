//! Luca UI 1.0 Beta — publication Beta (not stable 1.0).
//!
//! Scope: every locked 1.0 requirement with test evidence — minimal core
//! library (App/Text/Button) with per-component validation, deterministic
//! `TestHarness` testing API, frozen diagnostics catalog, and the
//! development-only inspector — over the unchanged 0.1–0.8 machinery.
//! Source-located diagnostics via `luca_code::error::LucaError`.
//!
//! The syntax remains a *minimal compatible assumption*, not locked design:
//! generic indented `Name:` components with `key = value` props (strict on
//! registered core names, generic elsewhere), generic `layout:`/`style:`
//! entry blocks, `state:` literal blocks, `event = handler` entries,
//! `transitions:` data blocks, `theme:` selection blocks, and
//! bare-identifier bindings resolved against state. The full inventory and
//! remaining open items are listed in `docs/implementation-status.md` §16–17.

pub mod accessibility;
pub mod animation;
pub mod bindings;
pub mod component;
pub mod events;
pub mod inspector;
pub mod layout;
pub mod library;
pub mod localization;
pub mod navigation;
pub mod parser;
pub mod rendering;
pub mod runtime;
pub mod seam;
pub mod state;
pub mod style;
pub mod testing;
pub mod themes;

pub use accessibility::{describe, describe_resolved, Description};
pub use animation::TransitionSet;
pub use bindings::{resolve_tree, ResolvedComponent, ResolvedProp};
pub use component::{Component, Prop, PropValue};
pub use events::{dispatch, Dispatch, EventBlock, EventDecl};
pub use inspector::{Inspector, Snapshot};
pub use layout::Layout;
pub use library::{lookup, CoreComponent, CORE_COMPONENTS};
pub use localization::Resources;
pub use luca_code::error::LucaError;
pub use navigation::Navigator;
pub use runtime::{LifecycleDiff, Runtime, ScopedTask};
pub use rendering::{format_value, is_visible, Renderer, TextBackend};
pub use state::{Notification, State, StateBlock, StateEntry};
pub use testing::TestHarness;
pub use style::Style;
pub use themes::{Environment, SystemAppearance, Theme, ThemeMode};

/// Product release label: `1.0 Beta` publication Beta. Not stable 1.0.
pub const PRODUCT_VERSION: &str = "1.0 Beta";

/// Parse and validate a `.lucu` source string into a component tree.
pub fn parse(source: &str) -> Result<Component, LucaError> {
    parser::parse(source)
}

/// Parse, validate, and run paired Luca Code (`.lucc`) logic.
///
/// Returns the UI tree plus the collected `print` output of the logic.
/// Proves the one-way seam: Luca UI reuses Luca Code; Luca Code never
/// depends on Luca UI.
pub fn run_paired(
    lucu_source: &str,
    lucc_source: &str,
) -> Result<(Component, Vec<String>), LucaError> {
    let tree = parser::parse(lucu_source)?;
    let output = seam::run_logic(lucc_source)?;
    Ok((tree, output))
}
