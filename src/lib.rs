//! Luca UI 1.0.2 Beta (the 1.0 Beta product line).
//!
//! Scope: every locked 1.0 requirement with test evidence — minimal core
//! library (App/Text/Button) with per-component validation, deterministic
//! `TestHarness` testing API, frozen diagnostics catalog, a headless text
//! backend, a static SVG visual backend with geometry/layout/paint/
//! hit-testing, and the development-only inspector — over the unchanged
//! 0.1–0.8 machinery. Source-located diagnostics via
//! `luca_code::error::LucaError`; visual-assumption warnings via
//! [`RenderDiagnostic`].
//!
//! The syntax remains a *minimal compatible assumption*, not locked design:
//! generic indented `Name:` components with `key = value` props (strict on
//! registered core names, generic elsewhere), generic `layout:`/`style:`
//! entry blocks, `state:` literal blocks, `event = handler` entries,
//! `transitions:` data blocks, `theme:` selection blocks, and
//! bare-identifier bindings resolved against state. The full component
//! inventory and every remaining open item stay explicitly open in this
//! release; nothing beyond the shipped behavior is claimed.

pub mod accessibility;
pub mod animation;
pub mod backends;
pub mod bindings;
pub mod component;
pub mod events;
pub mod geometry;
pub mod input;
pub mod inspector;
pub mod layout;
pub mod layout_engine;
pub mod library;
pub mod localization;
pub mod navigation;
pub mod paint;
pub mod parser;
pub mod pipeline;
pub mod rendering;
pub mod runtime;
pub mod seam;
pub mod state;
pub mod style;
pub mod testing;
pub mod themes;

pub use accessibility::{describe, describe_resolved, Description};
pub use animation::TransitionSet;
pub use backends::SvgBackend;
pub use bindings::{resolve_tree, ResolvedComponent, ResolvedProp};
pub use component::{Component, Prop, PropValue};
pub use events::{dispatch, Dispatch, EventBlock, EventDecl};
pub use geometry::{Point, Rect, Size};
pub use input::{click_target, hit_test, hit_test_id};
pub use inspector::{Inspector, Snapshot};
pub use layout::{Align, Direction, Layout, LayoutSpec};
pub use layout_engine::{compute_layout, text_metrics, LayoutNode};
pub use library::{lookup, CoreComponent, CORE_COMPONENTS};
pub use localization::Resources;
pub use luca_code::error::LucaError;
pub use navigation::Navigator;
pub use paint::{build_display_list, DisplayList, PaintCommand};
pub use pipeline::{build_visual, Visual, DEFAULT_HEIGHT, DEFAULT_WIDTH};
pub use rendering::{
    format_value, is_visible, RenderDiagnostic, Renderer, TextBackend, VisualBackend,
};
pub use runtime::{LifecycleDiff, Runtime, ScopedTask};
pub use seam::{HandlerCheck, HandlerDeclarations, MissingHandler};
pub use state::{Notification, State, StateBlock, StateEntry};
pub use style::{Color, Style, StyleSpec};
pub use testing::TestHarness;
pub use themes::{Environment, SystemAppearance, Theme, ThemeMode};

/// Package release label: `1.0.2 Beta` (the 1.0 Beta product line).
pub const PRODUCT_VERSION: &str = "1.0.2 Beta";

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
