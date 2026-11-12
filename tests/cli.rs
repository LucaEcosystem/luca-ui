//! CLI integration tests for `check` and `render`.
//!
//! These drive the built binary so argument handling, exit codes, paired
//! handler policy, and file output are verified as a user sees them.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VIEW: &str =
    "App:\n  id = \"app\"\n  layout:\n    gap = 2\n  Text:\n    id = \"t\"\n    text = \"hi\"\n";

const EVENT_VIEW: &str = "App:\n  id = \"app\"\n  Button:\n    id = \"go\"\n    text = \"Go\"\n    events:\n      tap = go\n";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_luca-ui")
}

fn temp_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("luca-ui-cli-{}-{stamp}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn check_succeeds_for_a_valid_view_and_reports_visual_diagnostics() {
    let dir = temp_dir();
    let view = write(&dir, "view.lucu", VIEW);
    let output = Command::new(bin())
        .arg("check")
        .arg(&view)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OK:"), "{stdout}");
    assert!(stdout.contains("no paired Luca Code source"), "{stdout}");

    // Recognized-but-invalid assumption entries warn but do not fail.
    let warned = write(
        &dir,
        "warned.lucu",
        "App:\n  layout:\n    direction = \"grid\"\n  style:\n    background = \"red\"\n",
    );
    let output = Command::new(bin())
        .arg("check")
        .arg(&warned)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Render warning"), "{stdout}");
    assert!(stdout.contains("Unknown layout direction"), "{stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_enforces_missing_handlers_only_when_locally_verifiable() {
    let dir = temp_dir();
    let view = write(&dir, "event.lucu", EVENT_VIEW);

    write(&dir, "event.lucc", "print(1)\n");
    let output = Command::new(bin())
        .arg("check")
        .arg(&view)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("names handler 'go'"), "{stderr}");
    assert!(stderr.contains("not defined in the paired"), "{stderr}");

    write(
        &dir,
        "event.lucc",
        "def func go(n (int)) (int):\n    return n\n",
    );
    let output = Command::new(bin())
        .arg("check")
        .arg(&view)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");

    // Imports make local verification impossible: report, do not fail.
    write(&dir, "event.lucc", "import SomeModule\n");
    let output = Command::new(bin())
        .arg("check")
        .arg(&view)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unverified"), "{stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn render_writes_a_deterministic_svg_file_or_stdout() {
    let dir = temp_dir();
    let view = write(&dir, "view.lucu", VIEW);
    let out = dir.join("view.svg");
    let output = Command::new(bin())
        .arg("render")
        .arg(&view)
        .arg("--out")
        .arg(&out)
        .args(["--width", "120", "--height", "80"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let svg = std::fs::read_to_string(&out).unwrap();
    assert!(svg.starts_with("<svg"), "{svg}");
    assert!(svg.contains("width=\"120\""), "{svg}");
    assert!(svg.contains("height=\"80\""), "{svg}");
    assert!(svg.contains(">hi</text>"), "{svg}");

    let output = Command::new(bin())
        .arg("render")
        .arg(&view)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("<svg"), "{stdout}");

    let output = Command::new(bin())
        .arg("render")
        .arg(&view)
        .arg("--width")
        .arg("0")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
