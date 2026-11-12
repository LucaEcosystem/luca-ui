//! Luca UI 1.0.2 Beta runnable example: resolve `examples/visual.lucu`,
//! compute the documented layout, and emit a static SVG snapshot
//! (`cargo run --example visual`).
//!
//! This is a snapshot, not an interactive surface: no hit testing, events,
//! fonts, or platform integration are exercised here.

use luca_ui::{build_visual, parse, resolve_tree, Size, State, SvgBackend};

fn main() {
    let source = std::fs::read_to_string("examples/visual.lucu").unwrap_or_else(|error| {
        eprintln!("Could not read examples/visual.lucu: {error}");
        std::process::exit(1);
    });
    let tree = parse(&source).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    let state = State::from_tree(&tree).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    let resolved = resolve_tree(&tree, &state).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    let visual = build_visual(&resolved, Size::new(320, 200)).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    for diagnostic in &visual.diagnostics {
        eprintln!("{diagnostic}");
    }
    let mut backend = SvgBackend::new();
    backend.render_visual(&visual).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    print!("{}", backend.output());
}
