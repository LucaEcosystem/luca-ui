//! Rendering abstraction for 0.7 Alpha (`runtime/rendering` in ARCHITECTURE).
//!
//! Locked facts from `docs/SPEC.md`: the UI targets a platform-independent
//! API backed by a rendering abstraction; native, custom/GPU, and web are
//! intended backend *options*; the core vocabulary carries no
//! platform-specific components. Renderer-specific behavior is explicitly
//! open, so backend output shapes are backend-defined — except for the
//! contract below, which every backend MUST honor:
//!
//! 1. A backend receives the resolved root (`ResolvedComponent`, i.e. UI
//!    output already derived from state) exactly once per `render` call.
//! 2. It visits visible nodes depth-first, parents before children.
//! 3. It skips `visible == false` subtrees entirely (visibility is an
//!    ordinary declarative property; there is no show/hide API).
//! 4. It reads only the platform-independent model (`ResolvedComponent`,
//!    `Description`, `Value`); it introduces no platform components and
//!    names no platform, renderer, or unit concepts.
//! 5. It is deterministic: the same resolved input renders the same way.
//!
//! Absent `visible` means shown: the property applies where applicable, and
//! nothing else in the locked model distinguishes hidden output. This and
//! the line format of `TextBackend` are minimal assumptions, not design
//! (see `docs/implementation-status.md`).

use luca_code::types::Value;

use crate::{describe_resolved, LucaError, ResolvedComponent};

/// The renderer contract. Backends implement presentation only; all
/// validation, state, and binding work happens before `render` is called.
pub trait Renderer {
    /// Backend name for diagnostics and backend selection (e.g. `"text"`).
    fn name(&self) -> &'static str;

    /// Present one resolved tree according to the contract above.
    fn render(&mut self, root: &ResolvedComponent) -> Result<(), LucaError>;
}

/// Contract helper: true unless the node declares `visible = false`.
/// Resolution already enforces bool-typed `visible`, so only the absent
/// case defaults here (shown).
pub fn is_visible(node: &ResolvedComponent) -> bool {
    match node.prop("visible") {
        Some(Value::Bool(shown)) => *shown,
        _ => true,
    }
}

/// Format one resolved value deterministically. Strings render quoted
/// (debug-escaped); every other scalar renders with Luca Code `display`
/// semantics, so exact decimal scale is preserved (`2.50`, not `2.5`).
pub fn format_value(value: &Value) -> String {
    match value {
        Value::Str(text) => format!("{text:?}"),
        _ => value.display(),
    }
}

/// The first concrete backend: a headless, dependency-free text backend.
/// It renders the resolved tree as indented structural lines — component
/// name, `id`, accessible label, then resolved props and layout/style
/// entries — and interprets no component names, so no component inventory
/// or platform vocabulary leaks in. Its deterministic string output also
/// serves the UI testing API foundation. Native, custom/GPU, and web
/// backends are NOT implemented in 0.7.
#[derive(Debug, Default)]
pub struct TextBackend {
    lines: Vec<String>,
}

impl TextBackend {
    /// Empty backend with no rendered lines.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render and return the whole output as one string (lines joined with
    /// `\n`, no trailing newline).
    pub fn render_to_string(root: &ResolvedComponent) -> Result<String, LucaError> {
        let mut backend = Self::new();
        backend.render(root)?;
        Ok(backend.output())
    }

    /// The rendered lines collected so far.
    pub fn output(&self) -> String {
        self.lines.join("\n")
    }

    fn emit(&mut self, node: &ResolvedComponent, depth: usize) {
        if !is_visible(node) {
            return;
        }
        let pad = "  ".repeat(depth);
        let mut head = node.name.clone();
        if let Some(id) = &node.id {
            head.push_str(&format!(" id={id:?}"));
        }
        let description = describe_resolved(node);
        if let Some(label) = description.label {
            head.push_str(&format!(" label={label:?}"));
        }
        self.lines.push(format!("{pad}{head}"));
        for prop in &node.props {
            self.lines.push(format!(
                "{pad}  .{} = {}",
                prop.key,
                format_value(&prop.value)
            ));
        }
        self.emit_entries(depth, "layout", &node.layout);
        self.emit_entries(depth, "style", &node.style);
        for child in &node.children {
            self.emit(child, depth + 1);
        }
    }

    fn emit_entries(
        &mut self,
        depth: usize,
        section: &str,
        entries: &[crate::ResolvedProp],
    ) {
        if entries.is_empty() {
            return;
        }
        let pad = "  ".repeat(depth);
        self.lines.push(format!("{pad}  {section}:"));
        for entry in entries {
            self.lines.push(format!(
                "{pad}    .{} = {}",
                entry.key,
                format_value(&entry.value)
            ));
        }
    }
}

impl Renderer for TextBackend {
    fn name(&self) -> &'static str {
        "text"
    }

    fn render(&mut self, root: &ResolvedComponent) -> Result<(), LucaError> {
        self.emit(root, 0);
        Ok(())
    }
}
