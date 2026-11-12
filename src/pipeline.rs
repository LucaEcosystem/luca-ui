//! Static visual pipeline: resolved tree → layout → display list.
//!
//! The pipeline is the single place that turns UI output into paint commands.
//! It collects the non-fatal diagnostics produced by the documented ◐
//! layout/style assumptions so callers can surface them alongside the visual
//! result.

use crate::bindings::ResolvedComponent;
use crate::geometry::Size;
use crate::layout_engine::{compute_layout, LayoutNode};
use crate::paint::{build_display_list, DisplayList};
use crate::rendering::RenderDiagnostic;
use crate::LucaError;

/// Default document size used by CLI and example rendering when no viewport
/// is supplied. ◐ assumption, not a platform requirement.
pub const DEFAULT_WIDTH: u32 = 320;
/// Default document height (see [`DEFAULT_WIDTH`]).
pub const DEFAULT_HEIGHT: u32 = 240;

/// One complete static visual: layout tree, display list, and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visual {
    pub layout: LayoutNode,
    pub display_list: DisplayList,
    pub diagnostics: Vec<RenderDiagnostic>,
}

impl Visual {
    /// Document size in abstract units.
    pub fn size(&self) -> Size {
        self.display_list.size
    }
}

/// Build a static visual for `root` inside `viewport`.
///
/// This performs no I/O and never mutates state. Layout/style assumption
/// warnings are returned in [`Visual::diagnostics`]; structural errors were
/// already reported by parsing and resolution.
pub fn build_visual(root: &ResolvedComponent, viewport: Size) -> Result<Visual, LucaError> {
    let (layout, diagnostics) = compute_layout(root, viewport)?;
    let display_list = build_display_list(&layout);
    Ok(Visual {
        layout,
        display_list,
        diagnostics,
    })
}
