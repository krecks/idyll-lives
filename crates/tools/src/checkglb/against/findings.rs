//! Findings, their classes, and the numbers a comparison keeps.

use super::*;

/// One row of `METRICS` that is over its limit.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub metric: &'static str,
    pub text: String,
}

pub(crate) fn found(metric: &'static str, text: String) -> Finding {
    debug_assert!(METRICS.iter().any(|(m, _)| *m == metric), "{metric}");
    Finding { metric, text }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Class {
    Same,
    Near,
    Changed,
}

/// The largest differences measured between two files that are not the same, for a `near` line.
#[derive(Clone, Debug, Default)]
pub struct Numbers {
    /// Which parts of the digest differ (`digest::Parts::differing`).
    pub differs: Vec<&'static str>,
    /// The worst part's mean and largest distance (m).
    pub surface_mean: f64,
    pub surface_max: f64,
    /// The worst part's share of turned normals, of texture coordinates off, and mean shading difference (greys).
    pub turned: f64,
    pub uv_off: f64,
    pub shade: f64,
    /// The material whose triangle count changed most, as a share (signed).
    pub triangles: f64,
}

impl Numbers {
    pub(crate) fn text(&self) -> String {
        let mut out = Vec::new();
        if !self.differs.is_empty() {
            out.push(format!("differs in {}", self.differs.join(", ")));
        }
        out.push(format!(
            "surface {} mean, {} at most",
            mm(self.surface_mean),
            mm(self.surface_max)
        ));
        if self.turned > 0.0 {
            out.push(format!("normals {} turned", share(self.turned)));
        }
        if self.uv_off > 0.0 {
            out.push(format!("texture {} off", share(self.uv_off)));
        }
        if self.shade > 0.0 {
            out.push(format!("shading {:.2} greys", self.shade));
        }
        if self.triangles != 0.0 {
            out.push(format!("triangles {:+.1} %", self.triangles * 100.0));
        }
        out.join("; ")
    }
}

pub(crate) fn mm(v: f64) -> String {
    if v.is_finite() {
        format!("{:.3} mm", v * 1000.0)
    } else {
        "no surface".into()
    }
}

pub(crate) fn share(v: f64) -> String {
    format!("{:.2} %", v * 100.0)
}

pub(crate) fn point(p: P3) -> String {
    format!("({:.4}, {:.4}, {:.4})", p[0], p[1], p[2])
}

/// Two files compared: whether they are the same, else what is over its limit and the numbers.
#[derive(Debug, Default)]
pub struct Compared {
    pub same: bool,
    pub findings: Vec<Finding>,
    pub numbers: Numbers,
}
