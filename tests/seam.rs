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
