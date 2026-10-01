//! 0.1 Alpha foundation tests: parsing, validation, errors, seam boundary.

use luca_ui::{parse, run_paired, seam};

#[test]
fn accepts_documented_example() {
    let src = std::fs::read_to_string("examples/hello.lucu").unwrap();
    let tree = parse(&src).unwrap();
    assert_eq!(tree.name, "App");
    assert_eq!(tree.id(), Some("app"));
    assert_eq!(tree.count(), 2);
    let child = tree.find_by_id("greeting").unwrap();
    assert_eq!(child.name, "Text");
}

#[test]
fn rejects_imperative_operations() {
    for src in [
        "App:\n  hide()\n",
        "App:\n  show()\n",
        "App:\n  hide = true\n",
        "App:\n  set_text = \"hi\"\n",
        "App:\n  text = greet(\"hi\")\n",
    ] {
        let err = parse(src).unwrap_err();
        assert!(err.message.contains("Imperative"), "unexpected: {err}");
        assert!(format!("{err}").starts_with("Error at "), "location: {err}");
    }
}

#[test]
fn validates_locked_property_rules() {
    // visible/disabled must be bool.
    let err = parse("App:\n  visible = \"yes\"\n").unwrap_err();
    assert!(err.message.contains("must be bool"), "{err}");
    let err = parse("App:\n  disabled = 1\n").unwrap_err();
    assert!(err.message.contains("must be bool"), "{err}");
    // id must be a non-empty string and unique.
    let err = parse("App:\n  id = 3\n").unwrap_err();
    assert!(err.message.contains("'id'"), "{err}");
    let err = parse("App:\n  id = \"a\"\n  Text:\n    id = \"a\"\n").unwrap_err();
    assert!(err.message.contains("Duplicate id"), "{err}");
    // Duplicate props rejected.
    let err = parse("App:\n  visible = true\n  visible = false\n").unwrap_err();
    assert!(err.message.contains("Duplicate property"), "{err}");
}

#[test]
fn reports_source_locations() {
    let err = parse("App:\n  visible = \"yes\"\n").unwrap_err();
    assert_eq!((err.line, err.column), (2, 3));
    let err = parse("App:\n\tvisible = true\n").unwrap_err();
    assert!(err.message.contains("Tabs"), "{err}");
    let err = parse("").unwrap_err();
    assert!(err.message.contains("Expected a component"), "{err}");
    let err = parse("App:\nApp:\n").unwrap_err();
    assert!(err.message.contains("single root"), "{err}");
}

#[test]
fn enforces_lucu_extension_and_never_touches_lucc() {
    let err = seam::load_file(std::path::Path::new("examples/hello.lucc")).unwrap_err();
    assert!(err.message.contains(".lucu"), "{err}");
    let tree = seam::load_file(std::path::Path::new("examples/hello.lucu")).unwrap();
    assert_eq!(tree.name, "App");
}

#[test]
fn seam_runs_paired_luca_code_without_copying_it() {
    let lucu = std::fs::read_to_string("examples/hello.lucu").unwrap();
    let lucc = std::fs::read_to_string("examples/hello.lucc").unwrap();
    let (tree, output) = run_paired(&lucu, &lucc).unwrap();
    assert_eq!(tree.name, "App");
    assert_eq!(output, vec!["Hello, Luca!".to_owned()]);
    // Logic errors surface as ordinary Luca errors through the seam.
    let bad = "print(undefined_var_xyz)";
    let err = seam::run_logic(bad).unwrap_err();
    assert!(format!("{err}").starts_with("Error at "), "{err}");
}
