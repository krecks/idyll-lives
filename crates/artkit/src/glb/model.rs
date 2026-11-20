//! A model, and sampling its animation.

use super::*;

impl Model {
    /// The animation named `clip`.
    pub fn clip(&self, clip: &str) -> Option<usize> {
        self.animations.iter().position(|a| a.name == clip)
    }

    /// Node `n`'s translation, rotation and scale in `pose` (its rest where the clip doesn't move it).
    pub fn local(&self, n: usize, pose: Pose) -> ([f32; 3], [f32; 4], [f32; 3]) {
        let node = &self.nodes[n];
        let (mut t, mut r, mut s) = (node.translation, node.rotation, node.scale);
        if let Some((a, time)) = pose
            && let Some(anim) = self.animations.get(a)
        {
            for c in anim.channels.iter().filter(|c| c.node == n) {
                let v = sample(c, time);
                match c.target {
                    Target::Translation => t = [v[0], v[1], v[2]],
                    Target::Rotation => r = [v[0], v[1], v[2], v[3]],
                    Target::Scale => s = [v[0], v[1], v[2]],
                }
            }
        }
        (t, r, s)
    }

    /// The nodes the scene draws, parents first.
    pub fn drawn(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut seen = vec![false; self.nodes.len()];
        let mut todo: Vec<usize> = self.roots.iter().rev().copied().collect();
        while let Some(n) = todo.pop() {
            if n >= self.nodes.len() || seen[n] {
                continue;
            }
            seen[n] = true;
            out.push(n);
            todo.extend(self.nodes[n].children.iter().rev());
        }
        out
    }

    /// Each node's place in the model in `pose` (identity for one the scene doesn't reach).
    pub fn world(&self, pose: Pose) -> Vec<Affine> {
        let mut out = vec![IDENTITY; self.nodes.len()];
        let mut parent = vec![None; self.nodes.len()];
        for (n, node) in self.nodes.iter().enumerate() {
            for &c in &node.children {
                parent[c] = Some(n);
            }
        }
        for n in self.drawn() {
            let (t, r, s) = self.local(n, pose);
            let own = trs(t, r, s);
            out[n] = match parent[n] {
                Some(p) => compose(&out[p], &own),
                None => own,
            };
        }
        out
    }
}

/// A channel's value at `time` (s): held before its first key and after its last; between two keys a straight
/// line (a turn: the quaternions' normalised blend, the keys being a frame apart).
pub fn sample(c: &Channel, time: f32) -> Vec<f32> {
    let w = c.target.width();
    let (stride, at) = match c.interpolation {
        Interpolation::CubicSpline => (3 * w, w),
        _ => (w, 0),
    };
    let key = |i: usize| &c.values[i * stride + at..i * stride + at + w];
    let n = c.times.len().min(c.values.len() / stride);
    if n == 0 {
        return vec![0.0; w];
    }
    if time <= c.times[0] {
        return key(0).to_vec();
    }
    if time >= c.times[n - 1] {
        return key(n - 1).to_vec();
    }
    let i = c.times[..n].partition_point(|&t| t <= time) - 1;
    let (a, b) = (key(i), key(i + 1));
    if c.interpolation == Interpolation::Step {
        return a.to_vec();
    }
    let u = (time - c.times[i]) / (c.times[i + 1] - c.times[i]);
    if c.target == Target::Rotation {
        let dot: f32 = (0..4).map(|k| a[k] * b[k]).sum();
        let sign = if dot < 0.0 { -1.0 } else { 1.0 };
        let q: Vec<f32> = (0..4).map(|k| a[k] + (b[k] * sign - a[k]) * u).collect();
        let len = q.iter().map(|x| x * x).sum::<f32>().sqrt();
        return q.iter().map(|x| x / len).collect();
    }
    (0..w).map(|k| a[k] + (b[k] - a[k]) * u).collect()
}
