//! Seam execution safety: guest `System.exit` must return through the
//! actual Luca UI seam as data instead of terminating the host process.

#[test]
fn guest_exit_through_run_logic_returns_exit_marker() {
    let err = luca_ui::seam::run_logic("import System\nSystem.exit(3)\n").unwrap_err();
    assert!(err.is_exit(), "exit must stay detectable: {err}");
    assert_eq!(err.to_string(), "Error at 1:1: System.exit(3)");
    // The test process is alive: the guest exit did not terminate the host.
    assert_eq!(
        luca_ui::seam::run_logic("print(\"alive\")\n").unwrap(),
        ["alive"]
    );
}

#[test]
fn guest_exit_through_run_logic_with_input_returns_exit_marker() {
    let err = luca_ui::seam::run_logic_with_input(
        "import System\nSystem.exit(5)\n",
        vec!["unused".to_owned()],
    )
    .unwrap_err();
    assert!(err.is_exit(), "exit must stay detectable: {err}");
    assert_eq!(err.to_string(), "Error at 1:1: System.exit(5)");
    assert_eq!(
        luca_ui::seam::run_logic("print(\"alive\")\n").unwrap(),
        ["alive"]
    );
}

#[test]
fn logic_runtime_errors_propagate_with_locations() {
    let err = luca_ui::seam::run_logic("print(10 / 0)\n").unwrap_err();
    assert!(!err.is_exit());
    assert_eq!(err.to_string(), "Error at 1:10: Division by zero");
}

#[test]
fn logic_syntax_errors_propagate_with_locations() {
    let err = luca_ui::seam::run_logic("print(undefined_var)\n").unwrap_err();
    assert!(!err.is_exit());
    assert!(err.to_string().contains("is not declared"), "{err}");
}

#[test]
fn scripted_input_reaches_ask_through_the_seam() {
    let output = luca_ui::seam::run_logic_with_input(
        "def const name (str) = ask \"Name?\"\nprint(\"hi {name}\")\n",
        vec!["Luca".to_owned()],
    )
    .unwrap();
    assert_eq!(output, ["Name?", "hi Luca"]);
}

const APP: &str = "App:\n  Button:\n    id = \"visit\"\n    text = \"Visit\"\n    events:\n      tap = record_visit\n";

const LOGIC: &str = "def func record_visit(n (int)) (int):\n    return n + 1\n";

#[test]
fn declared_handlers_are_read_from_luca_code_ast() {
    let declarations = luca_ui::seam::declared_handlers(LOGIC).unwrap();
    assert_eq!(declarations.functions, ["record_visit"]);
    assert!(!declarations.uses_imports);
    assert!(luca_ui::seam::declared_handlers("print(1)\n")
        .unwrap()
        .functions
        .is_empty());
}

#[test]
fn handler_check_confirms_a_declared_handler() {
    let tree = luca_ui::parse(APP).unwrap();
    let check = luca_ui::seam::check_handlers(&tree, LOGIC).unwrap();
    assert!(check.verified);
    assert!(check.is_clean());
}

#[test]
fn handler_check_reports_missing_declarations_with_locations() {
    let tree = luca_ui::parse(APP).unwrap();
    let check = luca_ui::seam::check_handlers(&tree, "print(1)\n").unwrap();
    assert!(check.verified);
    assert_eq!(check.missing.len(), 1);
    let missing = &check.missing[0];
    assert_eq!(missing.component_id.as_deref(), Some("visit"));
    assert_eq!(missing.event, "tap");
    assert_eq!(missing.handler, "record_visit");
    assert_eq!((missing.line, missing.column), (6, 7));
    assert_eq!(
        missing.to_error().to_string(),
        "Error at 6:7: Event 'tap' on component 'visit' names handler 'record_visit', which is not defined in the paired Luca Code source"
    );
}

#[test]
fn handler_check_does_not_claim_verification_when_imports_are_used() {
    let tree = luca_ui::parse(APP).unwrap();
    let logic = format!("import SomeModule\n{LOGIC}");
    let check = luca_ui::seam::check_handlers(&tree, &logic).unwrap();
    assert!(!check.verified);
    assert!(check.is_clean());
}

#[test]
fn paired_paths_and_file_loading_follow_the_pairing_contract() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("luca-ui-seam-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let view = dir.join("view.lucu");
    let logic = dir.join("view.lucc");
    std::fs::write(
        &view,
        "App:\n  state:\n    visits = 0\n  Text:\n    id = \"count\"\n    text = visits\n",
    )
    .unwrap();
    std::fs::write(&logic, "print(\"paired\")\n").unwrap();

    assert_eq!(luca_ui::seam::paired_logic_path(&view), logic);
    let (tree, output) = luca_ui::seam::load_paired(&view).unwrap();
    assert_eq!(tree.name, "App");
    assert_eq!(output, ["paired"]);
    assert_eq!(
        luca_ui::seam::load_logic(&logic).unwrap(),
        "print(\"paired\")\n"
    );

    let wrong = dir.join("view.txt");
    assert_eq!(
        luca_ui::seam::load_logic(&wrong).unwrap_err().to_string(),
        "Error at 1:1: Luca Code source files must use the .lucc extension"
    );
    assert!(luca_ui::seam::load_file(&wrong)
        .unwrap_err()
        .to_string()
        .contains("must use the .lucu extension"));

    std::fs::remove_file(&logic).unwrap();
    assert!(luca_ui::seam::load_paired(&view)
        .unwrap_err()
        .to_string()
        .contains("No paired Luca Code source found"));
    let _ = std::fs::remove_dir_all(&dir);
}
