//! `.lucu` declaration parser for 0.5 Alpha.
//!
//! Assumption syntax (NOT locked design):
//! ```text
//! App:
//!   id = "app"
//!   visible = true
//!   layout:
//!     gap = 2
//!   style:
//!     tone = "soft"
//!   Text:
//!     id = "greeting"
//!     text = "Hello, Luca!"
//! ```
//! - `--` starts a line comment (outside strings); `[[ ... ]]` is a
//!   multiline comment, matching Luca Code.
//! - Component lines are `Name:`; property lines are `key = value`.
//! - `layout:`/`style:`/`state:`/`events:` are reserved child blocks holding
//!   only `key = value` entries (at most one of each per component);
//!   layout/style entry names carry no locked meaning — see `crate::layout` /
//!   `crate::style`; `state:` entries must be literals — see `crate::state`;
//!   `events:` entries must be `event = handler` names — see `crate::events`.
//!   A bare-identifier value is a binding resolved against state at
//!   `crate::bindings::resolve_tree`.
//! - Indentation is spaces; tabs are rejected. Children/props must be
//!   indented relative to their parent; dedents must return to a known level.
//! - Values: `"string"`, `true`/`false`, `none`, numbers (`12`, `-3.50`),
//!   or a bare identifier reference (binding resolution deferred).
//! - A call-shaped line such as `hide()` is rejected as an imperative
//!   component operation per the locked declarative rules.

use crate::component::{Component, Prop, PropValue};
use crate::LucaError;

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if is_ident_start(c) => chars.all(is_ident_char),
        _ => false,
    }
}

/// Strip `[[ ... ]]` multiline comments, preserving line numbers.
/// Returns an error for an unterminated block at its start location.
fn strip_multiline(source: &str) -> Result<String, LucaError> {
    let mut out = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut line = 1usize;
    let mut col = 1usize;
    let mut in_string = false;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '\n' {
            out.push(c);
            line += 1;
            col = 1;
            i += 1;
            continue;
        }
        if c == '"' && (i == 0 || bytes[i - 1] as char != '\\') {
            in_string = !in_string;
            out.push(c);
            col += 1;
            i += 1;
            continue;
        }
        if !in_string && c == '[' && i + 1 < bytes.len() && bytes[i + 1] as char == '[' {
            let start_line = line;
            let start_col = col;
            i += 2;
            col += 2;
            let mut closed = false;
            while i < bytes.len() {
                let d = bytes[i] as char;
                if d == '\n' {
                    out.push(d);
                    line += 1;
                    col = 1;
                    i += 1;
                    continue;
                }
                if d == ']' && i + 1 < bytes.len() && bytes[i + 1] as char == ']' {
                    closed = true;
                    i += 2;
                    col += 2;
                    break;
                }
                // Swallow comment content; keep columns roughly aligned.
                col += 1;
                i += 1;
            }
            if !closed {
                return Err(LucaError::new("Unterminated multiline comment", start_line, start_col));
            }
            continue;
        }
        out.push(c);
        col += 1;
        i += 1;
    }
    Ok(out)
}

/// Strip a `--` comment starting outside a double-quoted string.
fn strip_line_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut prev: Option<char> = None;
    for (idx, c) in line.char_indices() {
        if c == '"' && prev != Some('\\') {
            in_string = !in_string;
        }
        if !in_string && c == '-' && line[idx..].starts_with("--") {
            return &line[..idx];
        }
        prev = Some(c);
    }
    line
}

fn count_indent(line: &str, line_no: usize) -> Result<(usize, &str), LucaError> {
    let mut width = 0usize;
    for c in line.chars() {
        match c {
            ' ' => width += 1,
            '\t' => {
                return Err(LucaError::new(
                    "Tabs are not allowed for indentation; use spaces",
                    line_no,
                    1,
                ));
            }
            _ => break,
        }
    }
    Ok((width, &line[width..]))
}

fn parse_value(text: &str, line: usize, column: usize) -> Result<PropValue, LucaError> {
    let t = text.trim();
    if t.is_empty() {
        return Err(LucaError::new("Expected a value after '='", line, column));
    }
    if t.starts_with('"') {
        if t.len() < 2 || !t.ends_with('"') {
            return Err(LucaError::new("Unterminated string literal", line, column));
        }
        let inner = &t[1..t.len() - 1];
        if inner.contains('\n') {
            return Err(LucaError::new("Unterminated string literal", line, column));
        }
        let mut value = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some(other) => {
                        return Err(LucaError::new(
                            format!("Unknown escape '\\{other}' in string"),
                            line,
                            column,
                        ));
                    }
                    None => {
                        return Err(LucaError::new("Unterminated string literal", line, column));
                    }
                }
            } else {
                value.push(c);
            }
        }
        return Ok(PropValue::Str(value));
    }
    match t {
        "true" => return Ok(PropValue::Bool(true)),
        "false" => return Ok(PropValue::Bool(false)),
        "none" => return Ok(PropValue::None),
        _ => {}
    }
    if is_number(t) {
        return Ok(PropValue::Number(t.to_owned()));
    }
    if is_ident(t) {
        // Bare reference; binding resolution is deferred past 0.1.
        // Luca keywords used as values are still accepted as refs here.
        return Ok(PropValue::Ref(t.to_owned()));
    }
    if t.contains('(') || t.contains(')') {
        return Err(LucaError::new(
            "Imperative component operations are not supported; update state and allow the UI to react",
            line,
            column,
        ));
    }
    Err(LucaError::new(format!("Invalid value '{t}'"), line, column))
}

fn is_number(text: &str) -> bool {
    let t = text.strip_prefix('-').unwrap_or(text);
    if t.is_empty() {
        return false;
    }
    let mut parts = t.split('.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next();
    if parts.next().is_some() {
        return false;
    }
    if int_part.is_empty() || !int_part.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    match frac_part {
        None => true,
        Some(f) => !f.is_empty() && f.chars().all(|c| c.is_ascii_digit()),
    }
}

/// Parse and validate a `.lucu` source string.
pub fn parse(source: &str) -> Result<Component, LucaError> {
    let cleaned = strip_multiline(source)?;
    // Raw nodes: (indent, kind) in source order.
    enum Node {
        Comp { name: String, line: usize, column: usize },
        PropNode { key: String, value: PropValue, line: usize, column: usize },
    }
    struct Item {
        indent: usize,
        node: Node,
    }
    let mut items: Vec<Item> = Vec::new();

    for (idx, raw) in cleaned.lines().enumerate() {
        let line_no = idx + 1;
        let without_comment = strip_line_comment(raw);
        if without_comment.trim().is_empty() {
            continue;
        }
        let (indent, content) = count_indent(without_comment, line_no)?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            continue;
        }
        let column = indent + 1;
        if let Some(stripped) = trimmed.strip_suffix(':') {
            let name = stripped.trim();
            if name.is_empty() {
                return Err(LucaError::new("Expected a component name before ':'", line_no, column));
            }
            if name.contains('=') || name.contains(' ') || name.contains('(') || name.contains(')') {
                if name.contains('(') || name.contains(')') {
                    return Err(LucaError::new(
                        "Imperative component operations are not supported; update state and allow the UI to react",
                        line_no,
                        column,
                    ));
                }
                return Err(LucaError::new(
                    format!("Invalid component name '{name}'"),
                    line_no,
                    column,
                ));
            }
            if !is_ident(name) {
                return Err(LucaError::new(
                    format!("Invalid component name '{name}'"),
                    line_no,
                    column,
                ));
            }
            // Reject a second colon inside the name portion.
            if name.contains(':') {
                return Err(LucaError::new(
                    format!("Invalid component name '{name}'"),
                    line_no,
                    column,
                ));
            }
            items.push(Item { indent, node: Node::Comp { name: name.to_owned(), line: line_no, column } });
        } else if trimmed.contains('=') {
            let eq = trimmed.find('=').expect("contains =");
            let key = trimmed[..eq].trim();
            let value_text = trimmed[eq + 1..].trim();
            if !is_ident(key) {
                return Err(LucaError::new(
                    format!("Invalid property name '{key}'"),
                    line_no,
                    column,
                ));
            }
            if ["hide", "show", "set_text"].contains(&key) {
                return Err(LucaError::new(
                    format!(
                        "Imperative component operation '{key}' is not supported; update state and allow the UI to react"
                    ),
                    line_no,
                    column,
                ));
            }
            let value_column = column + (trimmed[..eq + 1].len());
            let value = parse_value(value_text, line_no, value_column)?;
            items.push(Item {
                indent,
                node: Node::PropNode { key: key.to_owned(), value, line: line_no, column },
            });
        } else {
            // Bare call-shaped lines are imperative operations.
            if trimmed.contains('(') || trimmed.contains(')') {
                return Err(LucaError::new(
                    "Imperative component operations are not supported; update state and allow the UI to react",
                    line_no,
                    column,
                ));
            }
            return Err(LucaError::new(
                format!("Expected 'Name:' or 'key = value', found '{trimmed}'"),
                line_no,
                column,
            ));
        }
    }

    if items.is_empty() {
        return Err(LucaError::new("Expected a component, found empty file", 1, 1));
    }
    if items[0].indent != 0 {
        let line = match &items[0].node {
            Node::Comp { line, .. } => *line,
            Node::PropNode { line, .. } => *line,
        };
        return Err(LucaError::new("Root component must start at column 1", line, items[0].indent + 1));
    }
    // First item must be a component.
    match &items[0].node {
        Node::Comp { .. } => {}
        Node::PropNode { line, column, .. } => {
            return Err(LucaError::new("Property outside a component", *line, *column));
        }
    }

    // Build the tree with an indent stack of raw pointers via indices.
    // We materialize with a recursive builder over positions.
    let mut roots: Vec<Component> = Vec::new();
    // Stack of (indent, index-path into roots tree).
    // Instead of borrowing, track component positions as paths.
    struct Frame {
        indent: usize,
        path: Vec<usize>,
    }
    let mut stack: Vec<Frame> = Vec::new();
    // Known indent levels for dedent validation.
    let mut levels: Vec<usize> = vec![0];

    // Helper to reach a mutable component by path.
    fn at_path<'a>(roots: &'a mut [Component], path: &[usize]) -> &'a mut Component {
        let mut node = &mut roots[path[0]];
        for step in &path[1..] {
            node = &mut node.children[*step];
        }
        node
    }

    for item in items {
        match item.node {
            Node::Comp { name, line, column } => {
                if item.indent == 0 {
                    if !roots.is_empty() {
                        return Err(LucaError::new(
                            "Expected a single root component per .lucu file",
                            line,
                            column,
                        ));
                    }
                    roots.push(Component::new(name, line, column));
                    stack.clear();
                    stack.push(Frame { indent: 0, path: vec![roots.len() - 1] });
                    levels = vec![0];
                } else {
                    // Pop until a parent with smaller indent is found.
                    while stack.last().is_some_and(|f| f.indent >= item.indent) {
                        stack.pop();
                    }
                    let parent = stack.last().ok_or_else(|| {
                        LucaError::new("Component outside a parent component", line, column)
                    })?;
                    if !levels.contains(&item.indent) {
                        // New deeper level must be strictly deeper than parent.
                        if item.indent <= parent.indent {
                            return Err(LucaError::new(
                                "Inconsistent indentation; dedent must return to a known level",
                                line,
                                column,
                            ));
                        }
                        levels.push(item.indent);
                    }
                    let parent_path = parent.path.clone();
                    let parent_node = at_path(&mut roots, &parent_path);
                    parent_node.children.push(Component::new(name, line, column));
                    let child_idx = parent_node.children.len() - 1;
                    let mut path = parent_path;
                    path.push(child_idx);
                    stack.push(Frame { indent: item.indent, path });
                }
            }
            Node::PropNode { key, value, line, column } => {
                while stack.last().is_some_and(|f| f.indent >= item.indent) {
                    stack.pop();
                }
                let owner = stack.last().ok_or_else(|| {
                    LucaError::new("Property outside a component", line, column)
                })?;
                if item.indent <= owner.indent {
                    return Err(LucaError::new(
                        "Inconsistent indentation; dedent must return to a known level",
                        line,
                        column,
                    ));
                }
                if !levels.contains(&item.indent) {
                    levels.push(item.indent);
                }
                let path = owner.path.clone();
                at_path(&mut roots, &path).props.push(Prop { key, value, line, column });
            }
        }
    }

    debug_assert_eq!(roots.len(), 1);
    let mut root = roots.into_iter().next().expect("single root");
    if root.name == "layout"
        || root.name == "style"
        || root.name == "state"
        || root.name == "events"
        || root.name == "transitions"
        || root.name == "theme"
    {
        return Err(LucaError::new(
            format!("`{}` must belong to a component", root.name),
            root.line,
            root.column,
        ));
    }
    root.extract_layout_style()?;
    root.validate()?;
    Ok(root)
}
