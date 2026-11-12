use std::path::PathBuf;
use std::process;

use luca_ui::{
    build_visual, resolve_tree, seam, Size, State, SvgBackend, DEFAULT_HEIGHT, DEFAULT_WIDTH,
    PRODUCT_VERSION,
};

const USAGE: &str = "\
Usage:
  luca-ui <view.lucu>
  luca-ui check <view.lucu>
  luca-ui render <view.lucu> [--out <file.svg>] [--width <n>] [--height <n>]
  luca-ui inspect <view.lucu>            (debug builds only)

Validate a Luca UI file and print its component tree; check paired Luca Code
handlers and visual diagnostics; or render a static SVG snapshot.

Options:
  --help     Show this help and exit
  --version  Show the Luca UI version and exit

Examples:
  luca-ui hello.lucu
  luca-ui check hello.lucu
  luca-ui render hello.lucu --out hello.svg --width 480 --height 320
  luca-ui inspect hello.lucu";

fn main() {
    let mut raw = std::env::args();
    let _exe = raw.next();
    let action = match raw.next() {
        Some(action) => action,
        None => {
            eprintln!("{USAGE}");
            process::exit(2);
        }
    };
    match action.as_str() {
        "--help" | "-h" => {
            println!("{USAGE}");
        }
        "--version" | "-V" => {
            println!("luca-ui {PRODUCT_VERSION}");
        }
        "inspect" => {
            let target = require_target(&mut raw);
            run_inspect(&target);
        }
        "check" => {
            let target = require_target(&mut raw);
            run_check(&target);
        }
        "render" => {
            let target = require_target(&mut raw);
            run_render(&target, raw.collect());
        }
        path => match seam::load_file(&PathBuf::from(path)) {
            Ok(tree) => print_tree(&tree, 0),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        },
    }
}

fn require_target(args: &mut impl Iterator<Item = String>) -> String {
    match args.next() {
        Some(target) => target,
        None => {
            eprintln!("{USAGE}");
            process::exit(2);
        }
    }
}

/// Validate parsing, state, bindings, the documented visual assumptions, and
/// (when a sibling `.lucc` file exists) declared event handlers.
fn run_check(target: &str) {
    let path = PathBuf::from(target);
    let tree = match seam::load_file(&path) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    let state = match State::from_tree(&tree) {
        Ok(state) => state,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    let resolved = match resolve_tree(&tree, &state) {
        Ok(resolved) => resolved,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    let visual = match build_visual(&resolved, Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)) {
        Ok(visual) => visual,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    let mut failed = false;
    for diagnostic in &visual.diagnostics {
        println!("{diagnostic}");
    }

    let logic_path = seam::paired_logic_path(&path);
    if logic_path.exists() {
        match seam::load_logic(&logic_path).and_then(|source| seam::check_handlers(&tree, &source))
        {
            Ok(check) if check.verified => {
                for missing in &check.missing {
                    eprintln!("{}", missing.to_error());
                    failed = true;
                }
            }
            Ok(check) => {
                for missing in &check.missing {
                    println!(
                        "Note: handler '{}' on component '{}' was not found locally; the paired source imports modules, so it is unverified",
                        missing.handler, missing.component
                    );
                }
            }
            Err(error) => {
                eprintln!("{error}");
                failed = true;
            }
        }
    } else {
        println!(
            "Note: no paired Luca Code source at {}; handlers were not checked",
            logic_path.display()
        );
    }

    if failed {
        process::exit(1);
    }
    println!("OK: {target}");
}

struct RenderOptions {
    out: Option<PathBuf>,
    width: u32,
    height: u32,
}

fn parse_render_options(args: Vec<String>) -> RenderOptions {
    let mut options = RenderOptions {
        out: None,
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" | "-o" => {
                options.out = match args.next() {
                    Some(path) => Some(PathBuf::from(path)),
                    None => usage_error("--out requires a file path"),
                };
            }
            "--width" => {
                options.width = parse_dimension(args.next(), "--width");
            }
            "--height" => {
                options.height = parse_dimension(args.next(), "--height");
            }
            other => usage_error(&format!("unknown render option '{other}'")),
        }
    }
    options
}

fn parse_dimension(value: Option<String>, flag: &str) -> u32 {
    let Some(value) = value else {
        usage_error(&format!("{flag} requires a positive integer"));
    };
    match value.parse::<u32>() {
        Ok(number) if number > 0 => number,
        _ => usage_error(&format!("{flag} requires a positive integer")),
    }
}

fn usage_error(message: &str) -> ! {
    eprintln!("{message}\n\n{USAGE}");
    process::exit(2);
}

/// Render a static SVG snapshot of the resolved view.
fn run_render(target: &str, args: Vec<String>) {
    let options = parse_render_options(args);
    let path = PathBuf::from(target);
    let tree = match seam::load_file(&path) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    let state = match State::from_tree(&tree) {
        Ok(state) => state,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    let resolved = match resolve_tree(&tree, &state) {
        Ok(resolved) => resolved,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    let visual = match build_visual(&resolved, Size::new(options.width, options.height)) {
        Ok(visual) => visual,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    for diagnostic in &visual.diagnostics {
        eprintln!("{diagnostic}");
    }
    let mut backend = SvgBackend::new();
    if let Err(error) = backend.render_visual(&visual) {
        eprintln!("{error}");
        process::exit(1);
    }
    let svg = backend.output();
    match options.out {
        Some(output) => {
            if let Err(error) = std::fs::write(&output, svg) {
                eprintln!("Could not write {}: {error}", output.display());
                process::exit(1);
            }
        }
        None => println!("{svg}"),
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
        println!(
            "{pad}  transitions ({}:{})",
            transitions.line, transitions.column
        );
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
