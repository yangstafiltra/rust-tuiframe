#!/usr/bin/env python3
"""Fix mechanical compile errors in component examples.

Handles the classes that make up the bulk of failures:
  1. render_widget(widget, area, f.buffer_mut()) -> f.render_widget(widget, area)
  2. Trailing widget-construction expression without a render call / semicolon
     (E0308 "expected (), found List/Paragraph") -> wrap in f.render_widget(expr, area)
  3. Table::new(rows).header(Row::new(h)).widths(&[..]) old API -> Table::new(rows, widths).header(...)
  4. canvas Line struct literal -> ratatui::widgets::canvas::Line
  5. [Constraint::Fill(1); n] non-const repeat -> vec![Constraint::Fill(1); n]
  6. Line::from(..).block(..) -> Paragraph::new(Line::from(..)).block(..)

Operates on the `example` and `snippet` fields of each TOML. Prints a summary.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
COMPONENTS = ROOT / "components"

WIDGET_CTORS = (
    "Paragraph", "List", "Table", "Gauge", "LineGauge", "BarChart", "Chart",
    "Sparkline", "Tabs", "Calendar", "Scrollbar", "Canvas", "Block", "Fill",
    "Clear", "PieChart", "BubbleChart", "ScatterPlot",
)


def read_failing(path):
    """Read failing component names from a file, one per line."""
    if path is None:
        return []
    return [l.strip() for l in path.read_text().splitlines() if l.strip()]


def extract_fields(content):
    """Extract example/snippet field bodies from raw TOML text."""
    out = {}
    for field in ("example", "snippet"):
        m = re.search(rf"^{field} = '''(.*?)'''", content, re.DOTALL | re.MULTILINE)
        out[field] = m.group(1) if m else None
    return out


def replace_field(content, field, old, new):
    """Replace within a single field's body, preserving TOML quotes."""
    m = re.search(rf"^{field} = '''(.*?)'''", content, re.DOTALL | re.MULTILINE)
    if not m:
        return content, 0
    body = m.group(1)
    if old not in body:
        return content, 0
    new_body = body.replace(old, new)
    new_content = content[: m.start(1)] + new_body + content[m.end(1):]
    return new_content, 1


def replace_field_regex(content, field, pattern, repl, count=0):
    m = re.search(rf"^{field} = '''(.*?)'''", content, re.DOTALL | re.MULTILINE)
    if not m:
        return content, 0
    body = m.group(1)
    new_body, n = re.subn(pattern, repl, body, count=count)
    if n == 0:
        return content, 0
    new_content = content[: m.start(1)] + new_body + content[m.end(1):]
    return new_content, n


def fix_render_widget(content, name):
    """render_widget(widget, area, f.buffer_mut()) -> f.render_widget(widget, area)"""
    n = 0
    for field in ("example", "snippet"):
        content, c = replace_field_regex(
            content, field,
            r"render_widget\(([A-Za-z_][A-Za-z0-9_]*), ([A-Za-z_][A-Za-z0-9_]*), f\.buffer_mut\(\)\)",
            r"f.render_widget(\1, \2)")
        n += c
    return content, n


def fix_trailing_expr(content, field, name):
    """Wrap a trailing widget-construction expression with f.render_widget(expr, area);"""
    fields = extract_fields(content)
    body = fields.get(field)
    if body is None:
        return content, 0
    stripped = body.strip()
    if not stripped:
        return content, 0
    if "render_widget" in stripped:
        return content, 0
    # Work on the raw body so the replacement matches the file exactly.
    last = body.rstrip()
    if last.endswith(";"):
        return content, 0
    # Trailing expression = everything after the last top-level `;`.
    semi = _find_top_level_semicolon(last)
    tail_start = semi + 1 if semi != -1 else 0
    tail = last[tail_start:].strip()
    if not tail or tail.endswith("}") or tail.endswith(";"):
        return content, 0
    if tail.startswith(tuple(WIDGET_CTORS)):
        new_tail = f"f.render_widget({tail}, area);"
    elif tail.startswith("Layout::"):
        new_tail = (f"let chunks = {tail};"
                    f" for c in chunks.iter() {{ f.render_widget(Block::bordered(), *c); }}")
    elif tail.startswith("Style::"):
        new_tail = f"f.render_widget(Block::default().borders(Borders::ALL).style({tail}), area);"
    elif tail.startswith("let "):
        new_tail = tail + ";"
    else:
        return content, 0
    prefix = last[:tail_start]
    new_body = (prefix.rstrip() + " " if prefix.strip() else "") + new_tail
    return content.replace(body, new_body, 1), 1


def fix_trailing_expr_both(content, name):
    n = 0
    for field in ("example", "snippet"):
        content, c = fix_trailing_expr(content, field, name)
        n += c
    return content, n


def _find_top_level_semicolon(s):
    """Return index of last `;` at paren/bracket depth zero, skipping string literals."""
    depth = 0
    idx = -1
    i = 0
    n = len(s)
    while i < n:
        ch = s[i]
        if ch == '"':
            i += 1
            while i < n and s[i] != '"':
                if s[i] == "\\":
                    i += 1
                i += 1
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == ";" and depth == 0:
            idx = i
        i += 1
    return idx


def fix_multirender(content, name):
    """f.render_widget(A; B, area) (broken multi-widget) -> two render_widget calls."""
    def repl(m):
        inner = m.group(1)
        semi = _find_top_level_semicolon(inner)
        if semi == -1:
            return m.group(0)
        a = inner[:semi]
        b = inner[semi + 1:]
        return f"f.render_widget({a}, area);\n            f.render_widget({b}, area);"
    n = 0
    for field in ("example", "snippet"):
        content, c = replace_field_regex(
            content, field,
            r"f\.render_widget\(([^;]*;.*?),\s*area\)",
            repl)
        n += c
    return content, n


def fix_block_area_paren(content, name):
    """.block(Block::bordered().., area)) -> .block(Block::bordered()..), area)
    Fixes cases where `, area` ended up inside the block's parens."""
    n = 0
    for field in ("example", "snippet"):
        content, c = replace_field_regex(content, field, r"\),\s*area\)\)", r")), area)")
        n += c
    return content, n


def _has_top_level_comma(s):
    """True if a comma appears at bracket/paren depth zero."""
    depth = 0
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "," and depth == 0:
            return True
    return False


def _balanced_end(s, start):
    """Return index just past the matching close paren for s[start] == '('."""
    depth = 0
    for i in range(start, len(s)):
        if s[i] == "(":
            depth += 1
        elif s[i] == ")":
            depth -= 1
            if depth == 0:
                return i + 1
    return None


def fix_table_api(content, name):
    """Table::new(rows).header(Row::new(h)).block(..).widths(&[..]) old API
    -> Table::new(rows.map(Row::new), widths).header(Row::new(h)).block(..)"""
    n = 0
    for field in ("example", "snippet"):
        fields = extract_fields(content)
        body = fields.get(field)
        if body is None:
            continue
        new_body = body
        i = 0
        while True:
            idx = new_body.find("Table::new(", i)
            if idx == -1:
                break
            open_idx = idx + len("Table::new(")
            end = _balanced_end(new_body, open_idx - 1)
            if end is None:
                break
            rows = new_body[open_idx:end - 1]
            if _has_top_level_comma(rows):
                # Already two-arg form; leave alone.
                i = end
                continue
            j = end
            chain = {}
            order = []
            ok = True
            while j < len(new_body) and new_body[j] == ".":
                k = new_body.index("(", j)
                mname = new_body[j + 1:k]
                e = _balanced_end(new_body, k)
                margs = new_body[k + 1:e - 1]
                chain[mname] = margs
                order.append(mname)
                j = e
            widths = chain.get("widths")
            if widths is None:
                i = end
                continue
            # Rebuild: Table::new(rows, widths).header(..).block(..) in original order.
            parts = ["Table::new(%s, %s)" % (rows, widths)]
            for mname in order:
                if mname == "widths":
                    continue
                parts.append(".%s(%s)" % (mname, chain[mname]))
            if isinstance(rows, str) and "map(Row::new)" not in rows:
                parts[0] = parts[0].replace(rows + ",", rows + ".map(Row::new),", 1)
            rebuilt = "".join(parts)
            new_body = new_body[:idx] + rebuilt + new_body[j:]
            n += 1
            i = idx + len(rebuilt)
        if n and new_body != body:
            content = content.replace(body, new_body, 1)
    return content, n


def fix_canvas_line(content, name):
    n = 0
    for field in ("example", "snippet"):
        content, c = replace_field_regex(content, field, r"(&Line\s*\{)", r"&ratatui::widgets::canvas::Line {")
        n += c
    return content, n


def fix_vec_repeat(content, name):
    n = 0
    for field in ("example", "snippet"):
        content, c = replace_field_regex(content, field, r"(?<!\!)\[(Constraint::Fill\(1\)); n\]", r"vec![\1; n]")
        n += c
    return content, n


def fix_line_block(content, name):
    """Line::from(..).block(..) -> Paragraph::new(Line::from(..)).block(..)"""
    n = 0
    for field in ("example", "snippet"):
        content, c = replace_field_regex(
            content, field,
            r"(?<!Paragraph::new\()Line::from\(([^)]*)\)\.block\((\n?[^;]*?)\)",
            lambda m: f"Paragraph::new(Line::from({m.group(1)})).block({m.group(2)})")
        n += c
    return content, n


def fix_missing_render(content, name):
    """Example builds a widget via `let X = Widget::new(...)...;` but never renders it.
    Append `f.render_widget(X, area);` after the last statement."""
    n = 0
    for field in ("example", "snippet"):
        m = re.search(rf"^{field} = '''(.*?)'''", content, re.DOTALL | re.MULTILINE)
        if not m:
            continue
        body = m.group(1)
        stripped = body.strip()
        if not stripped or "render_widget" in stripped or "f.render" in stripped:
            continue
        lines = [l.rstrip() for l in body.splitlines()]
        non_empty_idx = [i for i, l in enumerate(lines) if l.strip()]
        if not non_empty_idx:
            continue
        last_idx = non_empty_idx[-1]
        last = lines[last_idx].strip()
        mm = re.match(r"let\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*", last)
        if not mm:
            continue
        var = mm.group(1)
        if var == "area":
            continue
        if last.startswith("let layout"):
            continue
        if "::new(" not in last and "::default(" not in last:
            continue
        new_body = body + f"\nf.render_widget({var}, area);"
        content = content[: m.start(1)] + new_body + content[m.end(1):]
        n += 1
    return content, n


def main():
    failing = read_failing(Path(sys.argv[1])) if len(sys.argv) > 1 else []
    fixers = [
        fix_render_widget,
        fix_table_api,
        fix_canvas_line,
        fix_vec_repeat,
        fix_line_block,
        fix_trailing_expr_both,
        fix_multirender,
        fix_block_area_paren,
        fix_missing_render,
    ]
    total_fixed = 0
    stats = {}
    for name in failing:
        toml = next(COMPONENTS.rglob(f"{name}.toml"), None)
        if toml is None:
            print(f"MISSING: {name}")
            continue
        content = toml.read_text()
        orig = content
        for fix in fixers:
            content, c = fix(content, name)
            if c:
                stats.setdefault(fix.__name__, 0)
                stats[fix.__name__] += 1
                total_fixed += c
        if content != orig:
            toml.write_text(content)
            print(f"FIXED: {name}")
        else:
            print(f"UNCHANGED: {name}")
    print(f"\nTotal edits: {total_fixed}")
    for k, v in stats.items():
        print(f"  {k}: {v}")


if __name__ == "__main__":
    main()
