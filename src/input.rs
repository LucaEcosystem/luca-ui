//! Hit testing over a computed layout tree.
//!
//! Hit testing is static and geometric: the deepest visible node whose
//! border box contains the point wins, searching children in reverse paint
//! order (last painted is topmost). It reports component identity only; it
//! does not execute handlers, know about platforms, or synthesize events.

use crate::geometry::Point;
use crate::layout_engine::LayoutNode;

/// The deepest laid-out node containing `point`, if any.
pub fn hit_test(layout: &LayoutNode, point: Point) -> Option<&LayoutNode> {
    if !layout.rect.contains(point) {
        return None;
    }
    for child in layout.children.iter().rev() {
        if let Some(hit) = hit_test(child, point) {
            return Some(hit);
        }
    }
    Some(layout)
}

/// The `id` of the deepest hit node, if that node declares one.
pub fn hit_test_id(layout: &LayoutNode, point: Point) -> Option<&str> {
    hit_test(layout, point).and_then(|node| node.id.as_deref())
}

/// The component id a static click at `point` should target: the nearest
/// identified node on the path from the root to the deepest hit node.
///
/// Container nodes are frequently anonymous; routing an event wants the
/// closest identified ancestor (or the hit node itself).
pub fn click_target(layout: &LayoutNode, point: Point) -> Option<&str> {
    fn walk<'a>(node: &'a LayoutNode, point: Point, current: Option<&'a str>) -> Option<&'a str> {
        if !node.rect.contains(point) {
            return None;
        }
        let current = node.id.as_deref().or(current);
        for child in node.children.iter().rev() {
            if let Some(target) = walk(child, point, current) {
                return Some(target);
            }
        }
        current
    }
    walk(layout, point, None)
}
