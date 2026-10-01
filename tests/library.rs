//! 0.9 Alpha: core component library — registered App/Text/Button with
//! the shared locked API, strict unknown-prop rejection on registered
//! names, open vocabulary elsewhere, zero platform concepts.

use luca_ui::{lookup, parse, CORE_COMPONENTS};

#[test]
fn registry_holds_the_minimal_cross_platform_core() {
    assert_eq!(CORE_COMPONENTS.len(), 3);
    let names: Vec<&str> = CORE_COMPONENTS.iter().map(|c| c.name).collect();
    assert_eq!(names, vec!["App", "Button", "Text"]);
    for core in CORE_COMPONENTS {
        // Shared locked API on every core component.
        for key in ["id", "visible", "disabled", "label"] {
            assert!(core.supports(key), "{} supports {key}", core.name);
        }
        assert!(!core.description.is_empty(), "{}", core.name);
        // Cross-platform purity: no platform, renderer, or toolkit names.
        let haystack = format!("{} {}", core.name, core.description).to_lowercase();
        for token in ["ios", "android", "macos", "windows", "web", "native", "gpu", "cocoa", "uikit", "dom"] {
            assert!(!haystack.contains(token), "{} mentions {token}", core.name);
        }
    }
    assert!(lookup("App").is_some());
    assert!(lookup("Missing").is_none());
    assert_eq!(
        lookup("App").unwrap().supports_list(),
        "disabled, id, label, visible"
    );
    assert_eq!(
        lookup("Text").unwrap().supports_list(),
        "disabled, id, label, text, visible"
    );
}

#[test]
fn enforces_registered_props_and_keeps_open_vocabulary() {
    // Registered: unknown props rejected with the supported list.
    let err = parse("Button:\n  id = \"b\"\n  tap = 1\n").unwrap_err().to_string();
    assert_eq!(
        err,
        "Error at 3:3: Unknown property 'tap' on component 'Button' (supports: disabled, id, label, text, visible)"
    );
    // Registered: listed props (locked + generic) accepted.
    let tree = parse("Button:\n  id = \"b\"\n  text = \"Go\"\n  disabled = false\n").unwrap();
    assert_eq!(tree.id(), Some("b"));
    // Unregistered: generic props still accepted (open vocabulary).
    let tree = parse("Panel:\n  id = \"p\"\n  anything = 1\n").unwrap();
    assert_eq!(tree.id(), Some("p"));
    // Locked rules still precede library rules everywhere.
    let err = parse("Text:\n  hide = true\n").unwrap_err().to_string();
    assert!(err.contains("Imperative"), "{err}");
}

#[test]
fn every_core_component_renders_cross_platform() {
    // The text backend presents each core component without interpreting
    // names: adaptation evidence for the whole registered set.
    let tree = parse(
        "App:\n  id = \"app\"\n  Text:\n    id = \"t\"\n    text = \"Hi\"\n  Button:\n    id = \"b\"\n    text = \"Go\"\n",
    )
    .unwrap();
    let state = luca_ui::State::from_tree(&tree).unwrap();
    let output = luca_ui::TextBackend::render_to_string(&luca_ui::resolve_tree(&tree, &state).unwrap())
        .unwrap();
    for id in ["app", "t", "b"] {
        assert!(output.contains(&format!("id=\"{id}\"")), "{output}");
    }
}
