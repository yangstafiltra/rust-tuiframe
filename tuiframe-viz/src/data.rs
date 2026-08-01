use std::collections::HashMap;

/// Unified data model shared by all charts.
///
/// Most charts interpret this as: `labels` = categories / x-axis ticks,
/// `series` = one or more value series. Hierarchical and graph charts
/// additionally use `tree` / `edges`.
/// Axis range (x_min, x_max, y_min, y_max) used to map data to pixels.
pub type Axis = (f64, f64, f64, f64);

#[derive(Clone, Debug)]
pub struct ChartData {
    pub title: String,
    pub labels: Vec<String>,
    pub series: Vec<Series>,
    pub tree: Vec<TreeNode>,
    pub edges: Vec<Edge>,
    /// Animated data scale stamped by the engine each frame so the axis
    /// glides instead of snapping when the dataset's nice max jumps.
    pub scale: Option<Axis>,
    /// Animated grid/axis scale that advances on a slower clock than `scale`
    /// (parallax: far grid lags near data).
    pub grid_scale: Option<Axis>,
}

impl Default for ChartData {
    fn default() -> Self {
        ChartData {
            title: String::new(),
            labels: Vec::new(),
            series: Vec::new(),
            tree: Vec::new(),
            edges: Vec::new(),
            scale: None,
            grid_scale: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Series {
    pub name: String,
    pub values: Vec<f64>,
}

impl Default for Series {
    fn default() -> Self {
        Series { name: String::new(), values: Vec::new() }
    }
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub name: String,
    pub value: f64,
    pub children: Vec<TreeNode>,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub weight: f64,
}

impl ChartData {
    pub fn single(values: Vec<f64>) -> Self {
        ChartData {
            title: String::new(),
            labels: Vec::new(),
            series: vec![Series { name: String::new(), values }],
            tree: Vec::new(),
            edges: Vec::new(),
            scale: None,
            grid_scale: None,
        }
    }

    pub fn label_n(values: Vec<f64>) -> Self {
        let labels = (0..values.len()).map(|i| (i + 1).to_string()).collect();
        let mut d = Self::single(values);
        d.labels = labels;
        d
    }

    pub fn len(&self) -> usize {
        self.series.first().map(|s| s.values.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Produce a "zeroed" version of this dataset: same labels / series count,
    /// but all values reset to 0. Used as the start state for entrance
    /// animations so charts grow in smoothly from nothing. Tree leaf values
    /// and edge weights are zeroed too so graph/tree charts also grow in.
    pub fn zeroed(&self) -> ChartData {
        let series = self
            .series
            .iter()
            .map(|s| Series { name: s.name.clone(), values: vec![0.0; s.values.len()] })
            .collect::<Vec<_>>();
        let tree = self.tree.iter().map(zero_tree).collect::<Vec<_>>();
        let edges = self
            .edges
            .iter()
            .map(|e| Edge { from: e.from.clone(), to: e.to.clone(), weight: 0.0 })
            .collect::<Vec<_>>();
        ChartData {
            title: self.title.clone(),
            labels: self.labels.clone(),
            series,
            tree,
            edges,
            scale: None,
            grid_scale: None,
        }
    }
}

fn zero_tree(node: &TreeNode) -> TreeNode {
    TreeNode {
        name: node.name.clone(),
        value: 0.0,
        children: node.children.iter().map(zero_tree).collect(),
    }
}

/// Parse user input text into ChartData.
///
/// Accepted formats:
///   - `1, 2, 3`                      one unnamed series
///   - `Series A: 1,2,3`              one named series
///   - multiple `Name: 1,2,3` lines   several series
///   - `@cat1,@cat2` lines give labels for the columns
///   - indented lines build a tree (treemap / sunburst)
///     `Root\n  A: 40\n    B: 10`
pub fn parse_data(text: &str) -> Result<ChartData, String> {
    let mut series: Vec<Series> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    let mut tree: Vec<TreeNode> = Vec::new();
    let mut tree_mode = false;

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let indent = raw_line.len() - raw_line.trim_start().len();

        if trimmed.starts_with('@') {
            labels = trimmed[1..]
                .split([',', ';'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            continue;
        }

        // A line that looks like `Name: v1, v2, ...` is a series line.
        if let Some((name, rest)) = split_name(trimmed) {
            let values = parse_values(rest)?;
            if !values.is_empty() {
                series.push(Series { name: name.to_string(), values });
            }
            continue;
        }

        // Otherwise numbers-only line(s): unnamed series.
        let values = parse_values(trimmed)?;
        if !values.is_empty() {
            let name = format!("Series {}", series.len() + 1);
            series.push(Series { name, values });
        } else if indent == 0 && series.is_empty() {
            // Root tree label line: `Root` with no values.
            if let Some((n, _)) = split_name(trimmed) {
                tree = vec![TreeNode { name: n.to_string(), value: 0.0, children: Vec::new() }];
                tree_mode = true;
            }
        }
    }

    if tree_mode && !tree.is_empty() {
        return Ok(ChartData { title: String::new(), labels, series, tree, edges: Vec::new(), scale: None, grid_scale: None });
    }
    if series.is_empty() {
        return Err("no numeric data found".to_string());
    }
    Ok(ChartData { title: String::new(), labels, series, tree: Vec::new(), edges: Vec::new(), scale: None, grid_scale: None })
}

/// Parse edges format for graph charts: `A - B: 3` per line.
pub fn parse_edges(text: &str) -> Result<ChartData, String> {
    let mut edges = Vec::new();
    let mut nodes: HashMap<String, f64> = HashMap::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('@') {
            continue;
        }
        if let Some((head, weight)) = t.split_once(':') {
            let w: f64 = weight.trim().parse().map_err(|_| format!("bad weight: {weight}"))?;
            if let Some((from, to)) = head.trim().split_once('-') {
                let (from, to) = (from.trim(), to.trim());
                *nodes.entry(from.to_string()).or_insert(0.0) += w;
                *nodes.entry(to.to_string()).or_insert(0.0) += w;
                edges.push(Edge { from: from.to_string(), to: to.to_string(), weight: w });
            }
        }
    }
    if edges.is_empty() {
        return Err("no edges (use `A - B: 3`) found".to_string());
    }
    let series = vec![Series { name: "node".into(), values: nodes.values().copied().collect() }];
    Ok(ChartData { title: String::new(), labels: Vec::new(), series, tree: Vec::new(), edges, scale: None, grid_scale: None })
}

fn split_name(s: &str) -> Option<(&str, &str)> {
    // `Name: values` or `Name: value`
    if let Some(idx) = s.find(':') {
        let name = s[..idx].trim();
        let rest = s[idx + 1..].trim();
        if !name.is_empty() && (rest.contains(',') || rest.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')) {
            return Some((name, rest));
        }
    }
    None
}

fn parse_values(s: &str) -> Result<Vec<f64>, String> {
    let mut out = Vec::new();
    for part in s.split([',', ';', ' ', '\t']) {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        match p.parse::<f64>() {
            Ok(v) => out.push(v),
            Err(_) => return Err(format!("bad number: {p:?}")),
        }
    }
    Ok(out)
}

/// Interpolate between two datasets. `t` in [0,1], eased by `ease(t)`.
/// Values are length-normalized so series of different lengths animate, and
/// labels/series are matched by index to avoid shape "teleporting". Tree
/// leaf values and edge weights are matched by path / endpoint and lerped so
/// hierarchical and graph charts also grow smoothly.
pub fn interpolate(from: &ChartData, to: &ChartData, t: f64, ease: impl Fn(f64) -> f64) -> ChartData {
    let e = ease(t);
    let n = to.series.len().max(from.series.len());
    let mut series = Vec::with_capacity(n);
    for i in 0..n {
        let a = from.series.get(i).cloned().unwrap_or_default();
        let b = to.series.get(i).cloned().unwrap_or_default();
        let name = if !b.name.is_empty() { b.name.clone() } else { a.name.clone() };
        let len = b.values.len().max(a.values.len());
        let mut values = Vec::with_capacity(len);
        for j in 0..len {
            let av = a.values.get(j).copied().unwrap_or(0.0);
            let bv = b.values.get(j).copied().unwrap_or(0.0);
            values.push(av + (bv - av) * e);
        }
        series.push(Series { name, values });
    }
    ChartData {
        title: if to.title.is_empty() { from.title.clone() } else { to.title.clone() },
        labels: if to.labels.is_empty() { from.labels.clone() } else { to.labels.clone() },
        series,
        tree: interp_trees(&from.tree, &to.tree, e),
        edges: interp_edges(&from.edges, &to.edges, e),
        scale: to.scale.or(from.scale),
        grid_scale: to.grid_scale.or(from.grid_scale),
    }
}

/// Lerp tree leaf values keyed by slash-path. Structure follows `to`; values
/// for paths missing in `from` grow from 0 so trees animate in smoothly.
fn interp_trees(from: &[TreeNode], to: &[TreeNode], e: f64) -> Vec<TreeNode> {
    if to.is_empty() {
        return from.to_vec();
    }
    let from_vals: std::collections::HashMap<String, f64> = {
        let mut m = std::collections::HashMap::new();
        for node in from {
            collect_paths(node, String::new(), &mut m);
        }
        m
    };
    let to_vals: std::collections::HashMap<String, f64> = {
        let mut m = std::collections::HashMap::new();
        for node in to {
            collect_paths(node, String::new(), &mut m);
        }
        m
    };
    let mut out = Vec::new();
    for node in to {
        out.push(rebuild_tree(node, String::new(), &from_vals, &to_vals, e));
    }
    let _ = to_vals;
    out
}

fn collect_paths(node: &TreeNode, prefix: String, out: &mut std::collections::HashMap<String, f64>) {
    let path = if prefix.is_empty() { node.name.clone() } else { format!("{prefix}/{}", node.name) };
    if node.children.is_empty() {
        out.insert(path, node.value);
    } else {
        for c in &node.children {
            collect_paths(c, path.clone(), out);
        }
    }
}

fn rebuild_tree(
    node: &TreeNode,
    prefix: String,
    from_vals: &std::collections::HashMap<String, f64>,
    _to_vals: &std::collections::HashMap<String, f64>,
    e: f64,
) -> TreeNode {
    let path = if prefix.is_empty() { node.name.clone() } else { format!("{prefix}/{}", node.name) };
    let children = node
        .children
        .iter()
        .map(|c| rebuild_tree(c, path.clone(), from_vals, _to_vals, e))
        .collect::<Vec<_>>();
    let target = node.value;
    let start = from_vals.get(&path).copied().unwrap_or(0.0);
    TreeNode {
        name: node.name.clone(),
        value: if children.is_empty() { start + (target - start) * e } else { node.value },
        children,
    }
}

/// Lerp edge weights matched by (from, to). Structure follows `to`; weights
/// for new edges grow from 0.
fn interp_edges(from: &[Edge], to: &[Edge], e: f64) -> Vec<Edge> {
    if to.is_empty() {
        return from.to_vec();
    }
    let from_w: std::collections::HashMap<(String, String), f64> = from
        .iter()
        .map(|ed| ((ed.from.clone(), ed.to.clone()), ed.weight))
        .collect();
    to.iter()
        .map(|ed| {
            let start = from_w.get(&(ed.from.clone(), ed.to.clone())).copied().unwrap_or(0.0);
            Edge { from: ed.from.clone(), to: ed.to.clone(), weight: start + (ed.weight - start) * e }
        })
        .collect()
}

pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Convenience: build a dataset from labeled series.
pub fn ds(title: &str, labels: &[&str], series: &[(&str, &[f64])]) -> ChartData {
    ChartData {
        title: title.to_string(),
        labels: labels.iter().map(|s| s.to_string()).collect(),
        series: series
            .iter()
            .map(|(name, vals)| Series { name: name.to_string(), values: vals.to_vec() })
            .collect(),
        tree: Vec::new(),
        edges: Vec::new(),
        scale: None,
        grid_scale: None,
    }
}

/// Convenience: single-series dataset with category labels.
pub fn single(title: &str, labels: &[&str], values: &[f64]) -> ChartData {
    ds(title, labels, &[("", values)])
}

/// Build a tree from `parent-child: value` depth indentation.
pub fn tree_from_flat(flat: &[(&str, f64)]) -> Vec<TreeNode> {
    let mut root = Vec::new();
    let mut stack: Vec<(usize, TreeNode)> = Vec::new();
    for &(path, value) in flat {
        let parts: Vec<&str> = path.split('/').collect();
        while stack.len() > parts.len() {
            let (_, node) = stack.pop().unwrap();
            if let Some((_, parent)) = stack.last_mut() {
                parent.children.push(node);
            } else {
                root.push(node);
            }
        }
        let node = TreeNode { name: parts.last().unwrap().to_string(), value, children: Vec::new() };
        if stack.is_empty() {
            stack.push((parts.len(), node));
        } else {
            let (lvl, _) = stack.last().unwrap().clone();
            if lvl < parts.len() {
                stack.push((parts.len(), node));
            } else {
                stack.pop();
                if let Some((_, p)) = stack.last_mut() {
                    p.children.push(node);
                } else {
                    root.push(node);
                }
            }
        }
    }
    while let Some((_, node)) = stack.pop() {
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.push(node);
        } else {
            root.push(node);
        }
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroed_keeps_shape_zeroes_values() {
        let d = ds("t", &["a", "b"], &[("s", &[3.0, 7.0])]);
        let z = d.zeroed();
        assert_eq!(z.series[0].values, vec![0.0, 0.0]);
        assert_eq!(z.series[0].name, "s");
        assert_eq!(z.labels, d.labels);
    }

    #[test]
    fn interpolate_eases_series_values() {
        let from = ChartData::single(vec![0.0, 0.0]);
        let to = ChartData::single(vec![10.0, 20.0]);
        let mid = interpolate(&from, &to, 0.5, ease_in_out_cubic);
        assert_eq!(mid.series[0].values, vec![5.0, 10.0]);
        let end = interpolate(&from, &to, 1.0, ease_in_out_cubic);
        assert_eq!(end.series[0].values, vec![10.0, 20.0]);
    }

    #[test]
    fn interpolate_normalizes_series_length() {
        let from = ChartData::single(vec![0.0, 0.0, 0.0]);
        let to = ChartData::single(vec![9.0, 18.0]);
        let mid = interpolate(&from, &to, 0.5, ease_in_out_cubic);
        assert_eq!(mid.series[0].values, vec![4.5, 9.0, 0.0]);
    }

    #[test]
    fn zeroed_zeroes_tree_and_edges() {
        let tree = vec![TreeNode {
            name: "A".into(),
            value: 5.0,
            children: vec![TreeNode { name: "C".into(), value: 7.0, children: vec![] }],
        }];
        let d = ChartData {
            title: "t".into(),
            labels: vec![],
            series: vec![],
            tree: tree.clone(),
            edges: vec![Edge { from: "x".into(), to: "y".into(), weight: 4.0 }],
            scale: None,
            grid_scale: None,
        };
        let z = d.zeroed();
        assert_eq!(z.tree[0].value, 0.0);
        assert_eq!(z.tree[0].children[0].value, 0.0);
        assert_eq!(z.edges[0].weight, 0.0);
    }

    #[test]
    fn interpolate_grows_tree_from_zero() {
        let to_tree = vec![TreeNode {
            name: "A".into(),
            value: 0.0,
            children: vec![TreeNode { name: "C".into(), value: 20.0, children: vec![] }],
        }];
        let to = ChartData { title: "t".into(), labels: vec![], series: vec![], tree: to_tree, edges: vec![], scale: None, grid_scale: None };
        let from = to.zeroed();
        let mid = interpolate(&from, &to, 0.5, ease_in_out_cubic);
        assert_eq!(mid.tree[0].children[0].value, 10.0);
        let end = interpolate(&from, &to, 1.0, ease_in_out_cubic);
        assert_eq!(end.tree[0].children[0].value, 20.0);
    }

    #[test]
    fn interpolate_grows_edges_from_zero() {
        let to_edges = vec![Edge { from: "x".into(), to: "y".into(), weight: 8.0 }];
        let to = ChartData { title: "t".into(), labels: vec![], series: vec![], tree: vec![], edges: to_edges, scale: None, grid_scale: None };
        let from = to.zeroed();
        let mid = interpolate(&from, &to, 0.5, ease_in_out_cubic);
        assert_eq!(mid.edges[0].weight, 4.0);
        let end = interpolate(&from, &to, 1.0, ease_in_out_cubic);
        assert_eq!(end.edges[0].weight, 8.0);
    }
}
