//! 0.7 Alpha runnable example: resolve `examples/render.lucu` against its
//! state and present it with the text backend (`cargo run --example render`).

use luca_ui::{parse, resolve_tree, State, TextBackend};

fn main() {
    let source = std::fs::read_to_string("examples/render.lucu").unwrap_or_else(|error| {
        eprintln!("Could not read examples/render.lucu: {error}");
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
    match TextBackend::render_to_string(&resolved) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
