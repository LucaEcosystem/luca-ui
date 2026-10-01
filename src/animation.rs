//! Declarative animation foundations for 0.8 Alpha.
//!
//! Locked facts from `docs/SPEC.md`: animation is part of the core API and
//! transitions are declarative. Transition syntax and timing defaults are
//! NOT locked, so the `transitions:` block below is a minimal compatible
//! assumption carrying generic entries with no timing or property semantics
//! (see `docs/implementation-status.md`).
//!
//! What this module does implement from locked-adjacent behavior:
//! transitions are declaration data, never imperative start/stop calls
//! (consistent with no-imperative-manipulation), and they gate on platform
//! reduced-motion: `active` returns no transitions when reduced motion is
//! on, preserving the core accessibility model. Executors, timing, and
//! interpolation arrive with the animation design record.

use crate::{layout::check_block_entries, LucaError, Prop};

/// Validated transition declarations belonging to one component. Entries
/// carry no locked meaning; timing and interpolation are undefined here.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionSet {
    pub entries: Vec<Prop>,
    pub line: usize,
    pub column: usize,
}

impl TransitionSet {
    /// Validate raw entries harvested from a `transitions:` block.
    pub(crate) fn from_entries(
        component: &str,
        entries: Vec<Prop>,
        line: usize,
        column: usize,
    ) -> Result<Self, LucaError> {
        check_block_entries("transitions", component, &entries)?;
        Ok(Self { entries, line, column })
    }

    /// True when the block declares no transitions.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Transitions eligible to run: all declared entries, or none when
    /// platform reduced-motion is on (accessibility preservation).
    pub fn active(&self, reduced_motion: bool) -> &[Prop] {
        if reduced_motion {
            &[]
        } else {
            &self.entries
        }
    }
}
