use std::{path::PathBuf, process};

use luca_ui::{seam, PRODUCT_VERSION};

const USAGE: &str = "Usage: luca-ui <view.lucu> | luca-ui inspect <view.lucu>\n\nValidate a Luca UI file and print its component tree, or (debug builds\nonly) inspect tree, state, and layout declarations.\n\nOptions:\n  --help     Show this help and exit\n  --version  Show the Luca UI version and exit\n\nExamples:\n  luca-ui hello.lucu\n  luca-ui inspect hello.lucu";

fn main() {
    let mut raw = std::env::args();
    let _exe = raw.next();
    let path = match raw.next() {
        Some(p) => p,
        None => {
            eprintln!("{USAGE}");
            process::exit(2);
        }
    };
    if path == "--help" || path == "-h" {
        println!("{USAGE}");
        return;
    }
    if path == "--version" || path == "-V" {
        println!("luca-ui {PRODUCT_VERSION}");
        return;
    }
    if path == "inspect" {
        let target = match raw.next() {
            Some(target) => target,
            None => {
                eprintln!("{USAGE}");
                process::exit(2);
            }
        };
        run_inspect(&target);
        return;
    }
    match seam::load_file(&PathBuf::from(&path)) {
        Ok(tree) => {
            print_tree(&tree, 0);
        }
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

/// Development-only inspection: debug builds snapshot tree, state, and
/// layout declarations; release builds refuse (dev tool, not a product
/// surface).
#[cfg(debug_assertions)]
fn run_inspect(target: &str) {
    let path = PathBuf::from(target);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not read {}: {error}", path.display());
            process::exit(1);
        }
    };
    if !target.ends_with(".lucu") {
        eprintln!("Luca UI source files must use the .lucu extension");
        process::exit(2);
    }
    match luca_ui::inspector::inspect_source(&source) {
        Ok(snapshot) => println!("{snapshot}"),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

#[cfg(not(debug_assertions))]
fn run_inspect(_target: &str) {
    eprintln!("The inspector is a development-only tool and is unavailable in release builds");
    process::exit(2);
}

fn print_tree(c: &luca_ui::Component, depth: usize) {
    let pad = "  ".repeat(depth);
    let id = c.id().map(|s| format!(" id={s:?}")).unwrap_or_default();
    println!("{pad}{} ({}:{}){id}", c.name, c.line, c.column);
    for p in &c.props {
        println!("{pad}  .{} = {}", p.key, value_text(&p.value));
    }
    if let Some(layout) = &c.layout {
        println!("{pad}  layout ({}:{})", layout.line, layout.column);
        for p in &layout.entries {
            println!("{pad}    .{} = {}", p.key, value_text(&p.value));
        }
    }
    if let Some(style) = &c.style {
        println!("{pad}  style ({}:{})", style.line, style.column);
        for p in &style.entries {
            println!("{pad}    .{} = {}", p.key, value_text(&p.value));
        }
    }
    if let Some(state) = &c.state {
        println!("{pad}  state ({}:{})", state.line, state.column);
        for p in &state.entries {
            println!("{pad}    .{} = {}", p.key, value_text(&p.value));
        }
    }
    if let Some(events) = &c.events {
        println!("{pad}  events ({}:{})", events.line, events.column);
        for d in &events.events {
            println!("{pad}    .{} -> {}", d.event, d.handler);
        }
    }
    if let Some(transitions) = &c.transitions {
        println!("{pad}  transitions ({}:{})", transitions.line, transitions.column);
        for p in &transitions.entries {
            println!("{pad}    .{} = {}", p.key, value_text(&p.value));
        }
    }
    if let Some(theme) = &c.theme {
        println!("{pad}  theme ({}:{})", theme.line, theme.column);
        if let Some(name) = &theme.name {
            println!("{pad}    .name = {name:?}");
        }
        println!("{pad}    .mode = {}", theme.mode.as_str());
        for p in &theme.tokens {
            println!("{pad}    .{} = {}", p.key, value_text(&p.value));
        }
    }
    for child in &c.children {
        print_tree(child, depth + 1);
    }
}

fn value_text(v: &luca_ui::PropValue) -> String {
    v.display_text()
}
