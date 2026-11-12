//! Visual foundation tests: the documented ◐ layout/style assumptions,
//! display-list paint order, static hit testing, and the SVG snapshot backend.
//!
//! These test observable behavior, not implementation details: rectangles in
//! abstract units, diagnostic presence/wording for recognized-but-invalid
//! entries, deterministic SVG output, and XML escaping.

use luca_ui::{
    build_visual, click_target, hit_test, parse, resolve_tree, text_metrics, Color, PaintCommand,
    Point, Rect, ResolvedComponent, Size, State, SvgBackend, Visual,
};

fn resolved(source: &str) -> ResolvedComponent {
    let tree = parse(source).unwrap();
    let state = State::from_tree(&tree).unwrap();
    resolve_tree(&tree, &state).unwrap()
}

fn visual(source: &str, width: u32, height: u32) -> Visual {
    build_visual(&resolved(source), Size::new(width, height)).unwrap()
}

#[test]
fn column_stacks_children_with_padding_and_gap() {
    let visual = visual(
        "App:\n  layout:\n    padding = 10\n    gap = 5\n  Text:\n    id = \"a\"\n    text = \"aa\"\n  Text:\n    id = \"b\"\n    text = \"b\"\n",
        200,
        100,
    );
    // The root fills the viewport when it declares no size.
    assert_eq!(visual.layout.rect, Rect::new(0, 0, 200, 100));
    // 16px font: 10-unit cells, 20-unit line height.
    assert_eq!(
        visual.layout.find_by_id("a").unwrap().rect,
        Rect::new(10, 10, 20, 20)
    );
    assert_eq!(
        visual.layout.find_by_id("b").unwrap().rect,
        Rect::new(10, 35, 10, 20)
    );
}

#[test]
fn row_direction_and_center_alignment() {
    let visual = visual(
        "App:\n  layout:\n    direction = \"row\"\n    padding = 5\n    gap = 4\n    align = \"center\"\n  Text:\n    id = \"a\"\n    text = \"aa\"\n  Text:\n    id = \"b\"\n    text = \"bb\"\n",
        200,
        100,
    );
    let a = visual.layout.find_by_id("a").unwrap();
    let b = visual.layout.find_by_id("b").unwrap();
    assert_eq!(a.rect, Rect::new(5, 40, 20, 20));
    assert_eq!(b.rect, Rect::new(29, 40, 20, 20));
}

#[test]
fn stretch_fills_the_cross_axis_unless_explicitly_sized() {
    let visual = visual(
        "App:\n  layout:\n    padding = 10\n    align = \"stretch\"\n  Text:\n    id = \"a\"\n    text = \"a\"\n  Text:\n    id = \"fixed\"\n    text = \"a\"\n    layout:\n      width = 25\n",
        200,
        100,
    );
    assert_eq!(visual.layout.find_by_id("a").unwrap().rect.width, 180);
    // Explicit cross-axis sizes win over stretch.
    assert_eq!(visual.layout.find_by_id("fixed").unwrap().rect.width, 25);
}

#[test]
fn explicit_root_size_overrides_the_viewport() {
    let visual = visual(
        "App:\n  layout:\n    width = 120\n    height = 80\n  Text:\n    id = \"a\"\n    text = \"a\"\n",
        300,
        200,
    );
    assert_eq!(visual.layout.rect, Rect::new(0, 0, 120, 80));
}

#[test]
fn hidden_subtrees_are_excluded_from_layout_and_paint() {
    let visual = visual(
        "App:\n  layout:\n    padding = 10\n  Text:\n    id = \"hide\"\n    text = \"hidden\"\n    visible = false\n  Text:\n    id = \"show\"\n    text = \"shown\"\n",
        200,
        100,
    );
    assert!(visual.layout.find_by_id("hide").is_none());
    assert_eq!(
        visual.layout.find_by_id("show").unwrap().rect,
        Rect::new(10, 10, 50, 20)
    );
    assert!(!format!("{:?}", visual.display_list).contains("hidden"));
}

#[test]
fn invalid_known_entries_produce_diagnostics_and_unknown_entries_stay_opaque() {
    let visual = visual(
        "App:\n  layout:\n    direction = \"grid\"\n    gap = -1\n    padding = 1.5\n    mystery = 7\n  style:\n    background = \"red\"\n    font_size = -2\n    tone = \"soft\"\n",
        100,
        100,
    );
    assert_eq!(visual.diagnostics.len(), 5, "{:?}", visual.diagnostics);
    for diagnostic in &visual.diagnostics {
        assert_eq!(diagnostic.component.as_deref(), Some("App"));
    }
    let messages: Vec<&str> = visual
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(messages[0].contains("Unknown layout direction \"grid\""));
    assert!(messages[1].contains("'gap' must be a non-negative integer"));
    assert!(messages[2].contains("'padding' must be a non-negative integer"));
    assert!(messages[3].contains("'background' must be a #rrggbb color"));
    assert!(messages[4].contains("'font_size' must be a non-negative integer"));
    assert!(!messages.iter().any(|message| message.contains("mystery")));
    assert!(!messages.iter().any(|message| message.contains("tone")));
}

#[test]
fn display_list_paints_parent_rect_then_text_then_children() {
    let visual = visual(
        "App:\n  style:\n    background = \"#000000\"\n  Text:\n    id = \"t\"\n    text = \"x\"\n    style:\n      background = \"#ffffff\"\n      border = \"#ff0000\"\n",
        100,
        100,
    );
    assert_eq!(visual.display_list.commands.len(), 3);
    match &visual.display_list.commands[0] {
        PaintCommand::Rect { rect, fill, .. } => {
            assert_eq!(*rect, Rect::new(0, 0, 100, 100));
            assert_eq!(*fill, Some(Color::parse("#000000").unwrap()));
        }
        other => panic!("expected root rect, found {other:?}"),
    }
    match &visual.display_list.commands[1] {
        PaintCommand::Rect { rect, stroke, .. } => {
            assert_eq!(*rect, Rect::new(0, 0, 10, 20));
            assert_eq!(*stroke, Some(Color::parse("#ff0000").unwrap()));
        }
        other => panic!("expected child rect, found {other:?}"),
    }
    match &visual.display_list.commands[2] {
        PaintCommand::Text { text, .. } => assert_eq!(text, "x"),
        other => panic!("expected text, found {other:?}"),
    }
}

#[test]
fn text_metrics_use_the_documented_cell_model() {
    assert_eq!(text_metrics("ab\nc", 16), Size::new(20, 40));
    assert_eq!(text_metrics("", 16), Size::new(0, 20));
}

#[test]
fn svg_output_is_deterministic_escaped_and_self_contained() {
    let source = "App:\n  style:\n    background = \"#000000\"\n    radius = 4\n  Text:\n    id = \"t\"\n    text = \"a<&>\\\"'\\nsecond\"\n    style:\n      color = \"#ffffff\"\n      font_size = 12\n";
    let visual = visual(source, 100, 60);
    let mut first = SvgBackend::new();
    first.render_visual(&visual).unwrap();
    let mut second = SvgBackend::new();
    second.render_visual(&visual).unwrap();
    assert_eq!(first.output(), second.output());

    let svg = first.output();
    assert!(svg.starts_with("<svg"), "{svg}");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("width=\"100\""));
    assert!(svg.contains("height=\"60\""));
    assert!(svg.contains("viewBox=\"0 0 100 60\""));
    assert!(svg.contains("fill=\"#000000\""));
    assert!(svg.contains("rx=\"4\""));
    assert!(svg.contains("font-size=\"12\""));
    assert!(svg.contains("fill=\"#ffffff\""));
    assert!(svg.contains("a&lt;&amp;&gt;&quot;&apos;"));
    assert!(svg.contains("<tspan"));
    assert!(svg.contains("second"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn static_hit_testing_finds_the_deepest_node_and_nearest_id() {
    let visual = visual(
        "App:\n  layout:\n    padding = 10\n  Panel:\n    id = \"panel\"\n    layout:\n      padding = 5\n    Text:\n      id = \"inner\"\n      text = \"x\"\n",
        200,
        100,
    );
    let inner = visual.layout.find_by_id("inner").unwrap();
    let point = Point::new(inner.rect.x + 1, inner.rect.y + 1);
    assert_eq!(hit_test(&visual.layout, point).unwrap().name, "Text");
    assert_eq!(click_target(&visual.layout, point), Some("inner"));

    // Inside the panel's padding but outside the child: nearest id is the
    // panel, not the child.
    let padding_point = Point::new(11, 11);
    assert_eq!(
        hit_test(&visual.layout, padding_point).unwrap().name,
        "Panel"
    );
    assert_eq!(click_target(&visual.layout, padding_point), Some("panel"));

    // Outside every rectangle.
    assert!(hit_test(&visual.layout, Point::new(250, 250)).is_none());
    assert!(click_target(&visual.layout, Point::new(250, 250)).is_none());
}
