//! Core cross-platform component library for 0.9 Alpha (`std/components`
//! in ARCHITECTURE).
//!
//! Locked facts from `docs/SPEC.md` (§Components): core components are
//! cross-platform; they adapt to the target platform while retaining the
//! shared API; platform-specific APIs live outside the core vocabulary.
//! The component *inventory* is NOT locked (it must follow the complete
//! design record), so this registry holds the 0.9 minimal core only — the
//! container, display, and action surface exercised by the shipped examples
//! and tests — and is explicitly extensible
//! (see `docs/implementation-status.md`).
//!
//! Every core component shares the locked API (`id`, `visible`, `disabled`,
//! `label`) plus its listed generic props. Validation rejects unknown
//! props on *registered* components with the supported list; unregistered
//! names stay generic (open vocabulary preserved). No entry names a
//! platform, renderer, or native toolkit: adaptation happens in backends
//! (the text backend presents every core component structurally).

/// One core component: shared locked API plus listed generic props.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreComponent {
    /// Component name as written in `.lucu` (`Name:`).
    pub name: &'static str,
    /// What the component is for, in platform-neutral terms.
    pub description: &'static str,
    /// Generic props beyond the locked set, sorted alphabetically.
    pub extra_props: &'static [&'static str],
}

impl CoreComponent {
    /// Locked keys every core component supports, sorted alphabetically.
    pub const LOCKED_PROPS: &'static [&'static str] =
        &["disabled", "id", "label", "visible"];

    /// True for locked keys and listed generic props alike.
    pub fn supports(&self, key: &str) -> bool {
        Self::LOCKED_PROPS.contains(&key) || self.extra_props.contains(&key)
    }

    /// Deterministic supported-key list for diagnostics.
    pub fn supports_list(&self) -> String {
        let mut keys: Vec<&str> = Self::LOCKED_PROPS.to_vec();
        keys.extend(self.extra_props.iter().copied());
        keys.sort();
        keys.join(", ")
    }
}

/// The 0.9 minimal core: application root, labeled text display, and
/// labeled action with routed events. Cross-platform by construction —
/// no entry carries platform meaning; backends adapt presentation.
pub const CORE_COMPONENTS: &[CoreComponent] = &[
    CoreComponent {
        name: "App",
        description: "Application root container",
        extra_props: &[],
    },
    CoreComponent {
        name: "Button",
        description: "Labeled action with routed events",
        extra_props: &["text"],
    },
    CoreComponent {
        name: "Text",
        description: "Labeled text display",
        extra_props: &["text"],
    },
];

/// Look up a registered core component by name (`None` = generic, open
/// vocabulary — accepted, validated by locked rules only).
pub fn lookup(name: &str) -> Option<&'static CoreComponent> {
    CORE_COMPONENTS.iter().find(|c| c.name == name)
}
