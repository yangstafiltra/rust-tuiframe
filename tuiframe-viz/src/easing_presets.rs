use crate::bezier::Bezier;

/// A named, built-in easing preset. Presets set the two inner control points
/// of a cubic bezier curve so developers can pick a rhythm instantly — the
/// same idea as AE plugin Flow's preset curves.
pub struct EasingPreset {
    pub name: &'static str,
    /// Bilingual description, e.g. "Gradual acceleration / 渐进加速".
    pub description: &'static str,
    pub p1: (f64, f64),
    pub p2: (f64, f64),
}

impl EasingPreset {
    pub fn bezier(&self) -> Bezier {
        Bezier::cubic((0.0, 0.0), self.p1, self.p2, (1.0, 1.0))
    }
}

/// The full catalog of built-in presets, in display order.
pub const PRESETS: [EasingPreset; 17] = [
    EasingPreset { name: "linear", description: "Constant speed / 匀速线性", p1: (0.0, 0.0), p2: (1.0, 1.0) },
    EasingPreset { name: "ease-in", description: "Gradual acceleration, then settle / 先慢后快，逐渐加速", p1: (0.42, 0.0), p2: (1.0, 1.0) },
    EasingPreset { name: "ease-out", description: "Fast start, gentle landing / 先快后慢，平缓收尾", p1: (0.0, 0.0), p2: (0.58, 1.0) },
    EasingPreset { name: "ease-in-out", description: "Slow start and end / 首尾慢、中间快", p1: (0.42, 0.0), p2: (0.58, 1.0) },
    EasingPreset { name: "ease-in-quad", description: "Quadratic ease-in / 二次方缓入", p1: (0.55, 0.0), p2: (0.85, 0.0) },
    EasingPreset { name: "ease-out-quad", description: "Quadratic ease-out / 二次方缓出", p1: (0.25, 1.0), p2: (0.45, 1.0) },
    EasingPreset { name: "ease-in-out-quad", description: "Quadratic ease-in-out / 二次方缓入缓出", p1: (0.45, 0.0), p2: (0.55, 1.0) },
    EasingPreset { name: "back", description: "Overshoots past the end / 越过终点再回弹", p1: (0.34, 1.56), p2: (0.64, 1.0) },
    EasingPreset { name: "back-out", description: "Pull back before starting / 起步前先回退", p1: (0.36, 0.0), p2: (0.66, -0.56) },
    EasingPreset { name: "anticipate", description: "Anticipate then accelerate / 先蓄力后冲刺", p1: (0.36, -0.3), p2: (0.64, 1.0) },
    EasingPreset { name: "elastic", description: "Springy elastic wobble / 弹性振荡", p1: (0.25, 1.25), p2: (0.5, 1.0) },
    EasingPreset { name: "bounce", description: "Bouncing landing / 落地弹跳", p1: (0.3, 1.7), p2: (0.6, 1.0) },
    EasingPreset { name: "overshoot", description: "Fast rise past the target / 快速越过目标", p1: (0.0, 1.4), p2: (1.0, 1.0) },
    EasingPreset { name: "undershoot", description: "Falls below before rising / 先下沉再回升", p1: (0.0, 0.0), p2: (1.0, -0.4) },
    EasingPreset { name: "soft-step", description: "Soft S-shaped curve / 柔和 S 形曲线", p1: (0.7, 0.0), p2: (0.3, 1.0) },
    EasingPreset { name: "hard-step", description: "Stiff near-instant switch / 接近瞬时的硬切换", p1: (0.1, 0.0), p2: (0.9, 1.0) },
    EasingPreset { name: "whip", description: "Snappy whip-like snap / 鞭打式骤停", p1: (0.5, 1.8), p2: (0.5, 0.0) },
];

/// Look up a preset by name (case-insensitive). Returns `None` if unknown.
pub fn by_name(name: &str) -> Option<&'static EasingPreset> {
    PRESETS.iter().find(|p| p.name.eq_ignore_ascii_case(name))
}

/// A category of presets is just the whole array here; expose a helper so the
/// editor can render the preset bar.
pub fn names() -> Vec<&'static str> {
    PRESETS.iter().map(|p| p.name).collect()
}

/// All presets as `(name, description)` pairs, in display order.
pub fn entries() -> Vec<(&'static str, &'static str)> {
    PRESETS.iter().map(|p| (p.name, p.description)).collect()
}
