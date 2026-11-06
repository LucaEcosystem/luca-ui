//! Luca UI 1.0.2 Beta runnable example: built-in localization resource lookup
//! (`cargo run --example localize`). No resource file format exists yet —
//! tables are built programmatically; lookup is exact-locale with no
//! fallback chain (see docs/styling-theming.md).

use luca_ui::Resources;

fn main() {
    let mut resources = Resources::new("en");
    resources.add("en", "greeting", "Hello, Luca!");
    resources.add("en", "farewell", "Goodbye!");
    resources.add("de", "greeting", "Hallo, Luca!");

    for locale in ["en", "de"] {
        let greeting = resources.lookup(locale, "greeting").unwrap_or("(missing)");
        println!("{locale}: {greeting}");
    }
    match resources.lookup("de", "farewell") {
        Some(text) => println!("de farewell: {text}"),
        None => println!("de farewell: (missing — no fallback chain)"),
    }
}
