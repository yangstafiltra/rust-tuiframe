/// Bezier curve evaluation and adaptive sampling.
///
/// A bezier easing curve is defined by an ordered list of control points where
/// the first point is anchored at `(0,0)` and the last at `(1,1)` (a
/// monotonic-ish easing curve). A standard cubic has four points; the editor
/// lets developers drag the inner handles or insert extra control points for
/// more elaborate shapes. `sample(x)` maps a time `x in [0,1]` to progress
/// `y in [0,1]` by solving the polynomial for `x` then evaluating `y`.
#[derive(Clone, Debug, PartialEq)]
pub struct Bezier {
    pub points: Vec<(f64, f64)>,
}

impl Default for Bezier {
    fn default() -> Self {
        Bezier::linear()
    }
}

impl Bezier {
    /// Cubic from its four control points.
    pub fn cubic(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> Self {
        Bezier { points: vec![p0, p1, p2, p3] }
    }

    /// Linear identity curve.
    pub fn linear() -> Self {
        Bezier::cubic((0.0, 0.0), (0.0, 0.0), (1.0, 1.0), (1.0, 1.0))
    }

    /// The curve anchored at (0,0)/(1,1) — callers must keep the endpoints sane.
    pub fn anchor() -> Self {
        Bezier::linear()
    }

    pub fn p0(&self) -> (f64, f64) {
        self.points[0]
    }

    pub fn p3(&self) -> (f64, f64) {
        *self.points.last().unwrap()
    }

    /// First interior handle (second control point).
    pub fn p1(&self) -> (f64, f64) {
        *self.points.get(1).unwrap_or(&self.points[0])
    }

    /// Last interior handle (penultimate control point).
    pub fn p2(&self) -> (f64, f64) {
        *self.points.get(self.points.len().saturating_sub(2)).unwrap_or(&self.points[0])
    }

    /// The draggable handles: every point except the two anchors.
    pub fn handles(&self) -> impl Iterator<Item = (usize, (f64, f64))> + '_ {
        self.points
            .iter()
            .enumerate()
            .skip(1)
            .take(self.points.len().saturating_sub(2))
            .map(|(i, p)| (i, *p))
    }

    /// Insert a control point at `idx` (clamped to [1, len-1] so the anchors stay).
    pub fn insert_point(&mut self, idx: usize, xy: (f64, f64)) {
        let idx = idx.clamp(1, self.points.len().saturating_sub(1));
        self.points.insert(idx, xy);
    }

    pub fn remove_point(&mut self, idx: usize) {
        if self.points.len() <= 2 {
            return;
        }
        let idx = idx.clamp(1, self.points.len().saturating_sub(2));
        self.points.remove(idx);
    }

    /// Bezier point at parameter `t in [0,1]` via de Casteljau's algorithm.
    pub fn point(&self, t: f64) -> (f64, f64) {
        let mut pts: Vec<(f64, f64)> = self.points.clone();
        let n = pts.len();
        for k in 1..n {
            for i in 0..n - k {
                let (a0, a1) = pts[i];
                let (b0, b1) = pts[i + 1];
                pts[i] = (a0 + (b0 - a0) * t, a1 + (b1 - a1) * t);
            }
        }
        pts[0]
    }

    /// Solve for the parameter `t` that yields a given `x`, then return `y`.
    /// Falls back to bisection on the x-coordinate which is robust for
    /// non-monotonic control points too.
    pub fn sample(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        // Newton-Raphson on x(t), with bisection fallback.
        let mut t = x;
        for _ in 0..6 {
            let (cx, cy) = self.point(t);
            let dx = cx - x;
            if dx.abs() < 1e-6 {
                return cy;
            }
            let dt = 0.001f64.max(t * 1e-3);
            let (nx, _) = self.point((t + dt).min(1.0));
            let deriv = (nx - cx) / dt;
            if deriv.abs() < 1e-9 {
                break;
            }
            t = (t - dx / deriv).clamp(0.0, 1.0);
        }
        // Bisection fallback for robustness.
        let mut lo = 0.0f64;
        let mut hi = 1.0f64;
        for _ in 0..30 {
            let mid = (lo + hi) / 2.0;
            let (cx, _) = self.point(mid);
            if cx < x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.point((lo + hi) / 2.0).1
    }

    /// Sample the curve as an array of (x, y) points.
    /// `density` controls how many samples are used; higher density yields
    /// smoother rendering on high-resolution terminals.
    pub fn samples(&self, density: usize) -> Vec<(f64, f64)> {
        let n = density.max(2);
        (0..=n)
            .map(|i| self.point(i as f64 / n as f64))
            .collect()
    }
}

/// Recommended sample density for a terminal of the given width.
///
/// The density adapts to the available horizontal resolution so the curve
/// stays smooth without over-sampling on small terminals.
pub fn density_for(width: u16) -> usize {
    (width as usize / 2).clamp(24, 256)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_samples_identity() {
        let b = Bezier::linear();
        assert!(b.sample(0.0).abs() < 1e-6);
        assert!((b.sample(0.25) - 0.25).abs() < 1e-3);
        assert!((b.sample(1.0) - 1.0).abs() < 1e-6);
        let pts = b.samples(10);
        assert_eq!(pts.len(), 11);
        let (x, y) = pts[5];
        assert!((x - 0.5).abs() < 1e-9);
        assert!((y - 0.5).abs() < 1e-9);
    }

    #[test]
    fn ease_in_out_reaches_endpoints() {
        let b = crate::easing_presets::PRESETS
            .iter()
            .find(|p| p.name == "ease-in-out")
            .unwrap()
            .bezier();
        assert!((b.sample(0.0)).abs() < 1e-6);
        assert!((b.sample(1.0) - 1.0).abs() < 1e-6);
        // midpoint should be exactly 0.5
        assert!((b.sample(0.5) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn ease_in_is_slow_start() {
        let b = crate::easing_presets::PRESETS
            .iter()
            .find(|p| p.name == "ease-in")
            .unwrap()
            .bezier();
        // at x=0.3, ease-in should be well below linear's 0.3
        assert!(b.sample(0.3) < 0.15);
    }

    #[test]
    fn ease_out_is_fast_start() {
        let b = crate::easing_presets::PRESETS
            .iter()
            .find(|p| p.name == "ease-out")
            .unwrap()
            .bezier();
        // ease-out reaches y≈0.44 by x=0.3 — well above linear's 0.3
        assert!(b.sample(0.3) > 0.4);
    }

    #[test]
    fn overshoot_curves_can_exceed_one() {
        for name in ["back", "elastic", "bounce"] {
            let b = crate::easing_presets::PRESETS
                .iter()
                .find(|p| p.name == name)
                .unwrap()
                .bezier();
            let max_y = (0..=100).map(|i| b.sample(i as f64 / 100.0)).fold(0.0_f64, f64::max);
            assert!(max_y > 1.01, "{name} should overshoot");
        }
    }

    #[test]
    fn undershoot_curves_dip_below_zero() {
        for name in ["back-out", "anticipate"] {
            let b = crate::easing_presets::PRESETS
                .iter()
                .find(|p| p.name == name)
                .unwrap()
                .bezier();
            let min_y = (0..=100).map(|i| b.sample(i as f64 / 100.0)).fold(0.0_f64, f64::min);
            assert!(min_y < -0.01, "{name} should undershoot");
        }
    }

    #[test]
    fn bounce_returns_near_zero_late() {
        let b = crate::easing_presets::PRESETS
            .iter()
            .find(|p| p.name == "bounce")
            .unwrap()
            .bezier();
        // bounce ends settled at y≈1
        assert!((b.sample(0.98) - 1.0).abs() < 0.15);
        assert!((b.sample(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn density_adapts_to_width() {
        assert_eq!(density_for(40), 24);
        assert_eq!(density_for(96), 48);
        assert_eq!(density_for(300), 150);
        assert_eq!(density_for(600), 256);
        assert_eq!(density_for(100), 50);
    }
}
