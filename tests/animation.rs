//! 0.8 Alpha: declarative transitions, first-class themes, system
//! appearance, and localization foundations.
//!
//! Timing defaults, theme token names, and resource formats are NOT locked
//! and are NOT defined here — entries/tokens/tables are structural data
//! only (see `docs/implementation-status.md`).

use luca_ui::{
    parse, Environment, Resources, SystemAppearance, ThemeMode,
};

fn example() -> luca_ui::Component {
    parse(&std::fs::read_to_string("examples/themes.lucu").unwrap()).unwrap()
}

#[test]
fn declares_transitions_as_data() {
    let tree = example();
    // Transitions are declared entries with no timing semantics attached.
    let header = tree.find_by_id("header").unwrap();
    let transitions = header.transitions.as_ref().expect("transitions block");
    assert_eq!(transitions.entries.len(), 1);
    assert_eq!(transitions.entries[0].key, "fade");
    // Empty blocks are allowed.
    let detail = tree.find_by_id("detail").unwrap();
    assert!(detail.transitions.as_ref().unwrap().is_empty());
    // Blocks are leaf attachments: tree shape and identity are unaffected.
    assert_eq!((tree.count(), tree.depth()), (4, 3));
    assert_eq!(tree.ids(), vec!["app", "header", "title", "detail"]);
}

#[test]
fn gates_transitions_on_reduced_motion() {
    let tree = example();
    let header = tree.find_by_id("header").unwrap();
    let transitions = header.transitions.as_ref().unwrap();
    // Motion allowed: declared transitions are eligible.
    assert_eq!(transitions.active(false).len(), 1);
    // Platform reduced-motion on: transitions resolve instant (suppressed),
    // preserving the core accessibility model.
    assert!(transitions.active(true).is_empty());
    let env = Environment::new(SystemAppearance::Light, true);
    assert!(env.reduced_motion);
    assert!(header.transitions.as_ref().unwrap().active(env.reduced_motion).is_empty());
}

#[test]
fn rejects_invalid_transition_declarations() {
    let cases = [
        // Entries only; at most one block; must belong to a component.
        (
            "App:\n  id = \"a\"\n  transitions:\n    Text:\n",
            "Only `key = value` entries are allowed inside `transitions`",
        ),
        (
            "App:\n  id = \"a\"\n  transitions:\n    fade = \"in\"\n  transitions:\n    fade = \"out\"\n",
            "Duplicate 'transitions' block",
        ),
        ("transitions:\n  fade = \"in\"\n", "`transitions` must belong to a component"),
        // Structural rules shared with every block kind.
        (
            "App:\n  id = \"a\"\n  transitions:\n    fade = \"in\"\n    fade = \"out\"\n",
            "Duplicate 'fade' in `transitions`",
        ),
        ("App:\n  id = \"a\"\n  transitions:\n    hide = true\n", "Imperative"),
    ];
    for (src, fragment) in cases {
        let msg = parse(src).unwrap_err().to_string();
        assert!(msg.contains(fragment), "src={src:?} got {msg}");
        assert!(msg.starts_with("Error at "), "src={src:?} got {msg}");
    }
}

#[test]
fn selects_themes_and_follows_system_appearance() {
    let tree = example();
    // Absent mode follows system appearance automatically (the default).
    let app_theme = tree.theme.as_ref().expect("root theme");
    assert_eq!(app_theme.name.as_deref(), Some("Demo"));
    assert_eq!(app_theme.mode, ThemeMode::System);
    assert_eq!(
        app_theme.effective(SystemAppearance::Light),
        SystemAppearance::Light
    );
    assert_eq!(
        app_theme.effective(SystemAppearance::Dark),
        SystemAppearance::Dark
    );
    // Pinned modes ignore the platform signal.
    let detail_theme = tree.find_by_id("detail").unwrap().theme.as_ref().unwrap();
    assert_eq!(detail_theme.mode, ThemeMode::Dark);
    assert_eq!(
        detail_theme.effective(SystemAppearance::Light),
        SystemAppearance::Dark
    );
    // Mode words work bare or quoted; the default needs no declaration.
    let tree = parse("App:\n  theme:\n    mode = light\n").unwrap();
    assert_eq!(tree.theme.as_ref().unwrap().mode, ThemeMode::Light);
    let tree = parse("App:\n  theme:\n    mode = \"dark\"\n").unwrap();
    assert_eq!(tree.theme.as_ref().unwrap().mode, ThemeMode::Dark);
    let tree = parse("App:\n  theme:\n").unwrap();
    let theme = tree.theme.as_ref().unwrap();
    assert_eq!(theme.mode, ThemeMode::System);
    assert_eq!(theme.name, None);
    // Unknown modes and non-string names are Luca errors at the entry.
    let msg = parse("App:\n  theme:\n    mode = sepia\n").unwrap_err().to_string();
    assert!(msg.contains("Unknown theme mode 'sepia'"), "{msg}");
    let msg = parse("App:\n  theme:\n    mode = 3\n").unwrap_err().to_string();
    assert!(msg.contains("Unknown theme mode"), "{msg}");
    let msg = parse("App:\n  theme:\n    name = 3\n").unwrap_err().to_string();
    assert!(msg.contains("Theme 'name' must be a string"), "{msg}");
}

#[test]
fn looks_up_localized_resources() {
    let mut resources = Resources::new("en");
    assert_eq!(resources.default_locale(), "en");
    assert!(resources.locales().is_empty());
    resources.add("en", "greeting", "Hello, Luca!");
    resources.add("en", "farewell", "Goodbye!");
    resources.add("de", "greeting", "Hallo, Luca!");
    assert_eq!(resources.locales(), vec!["de".to_owned(), "en".to_owned()]);
    // Exact-locale lookup per locale.
    assert_eq!(resources.lookup("en", "greeting"), Some("Hello, Luca!"));
    assert_eq!(resources.lookup("de", "greeting"), Some("Hallo, Luca!"));
    // Missing keys and unknown locales return None — no fallback chain.
    assert_eq!(resources.lookup("de", "farewell"), None);
    assert_eq!(resources.lookup("fr", "greeting"), None);
    // Overwriting a key replaces the text.
    resources.add("en", "greeting", "Hi!");
    assert_eq!(resources.lookup("en", "greeting"), Some("Hi!"));
}
