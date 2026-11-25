//! Furniture: what it is, its height and the seats it offers.

use super::*;

/// How far (m) a mesh may be from a functional height (a seat, a top, a mattress) before the gate warns; and
/// how far two sources of one may differ before it calls them different.
const HEIGHT_OFF: f64 = 0.02;
const SAME: f64 = 0.005;
/// A seat's front edge (furniture.json `seats` `front`, `Sampler::front`): where its top has dropped this far (m)
/// below the seat's height (a cushion's rounded front), and how far the model's may be from what the table says.
const FRONT_DROP: f64 = 0.06;
const FRONT_OFF: f64 = 0.03;
/// How far (m) around the point a functional height is measured: the widest gap a top may have there and still be
/// the thing itself (a bench's slats, a table's planks, a cushion's seams). A single point falls between them and
/// reports nothing under a seat that is right.
const PROBE: f64 = 0.05;

/// A seat this low (m) is the ground (furniture.json `seats` `shade`: against a tree's trunk), not the model's to measure.
const GROUND: f64 = 0.02;

/// furniture.json: where residents sit and lie on each object (characters.ts `SEATS`, `BEDS`, `BUNKS` and their
/// defaults). An object in `seats` takes that seat over the content's `seat` (characters.ts `seatOf`).
pub struct Furniture {
    pub(super) seats: J,
    pub(super) default_seat: J,
    pub(super) beds: J,
    pub(super) default_bed: J,
    pub(super) bunks: J,
}

impl Furniture {
    /// Reads `paths.furniture`; a file that can't be read is an error naming it.
    pub fn read(paths: &Paths) -> Result<Furniture, String> {
        let f = read_json(&paths.furniture)
            .map_err(|e| format!("{}: {e}", relpath(&paths.furniture, &repo_root())))?;
        let part = |k: &str| f.get(k).cloned().unwrap_or(J::Obj(Vec::new()));
        Ok(Furniture {
            seats: part("seats"),
            default_seat: part("defaultSeat"),
            beds: part("beds"),
            default_bed: part("defaultBed"),
            bunks: part("bunks"),
        })
    }
}

/// A number of the content's (`d[k]`), or `default`.
fn number(d: Option<&J>, k: &str, default: f64) -> f64 {
    d.and_then(|d| d.get(k))
        .and_then(J::num)
        .map_or(default, Num::f)
}

/// A model as the game draws it, its parts where a pose puts them, to measure heights on: its faces (hidden
/// materials, and parts flattened to nothing at rest, left out) and the entry's `scale`.
pub(super) struct Sampler {
    tris: Vec<Tri>,
    scale: f64,
}

impl Sampler {
    /// None if its positions can't be read here (compressed).
    pub(super) fn new(g: &Glb, entry: &J, pose: Option<&Pose>) -> R<Option<Sampler>> {
        Self::without(g, entry, pose, &HashSet::new())
    }

    /// The model's own body at rest (`cargo review sizes`' top and table): what it carries (`aside_nodes`, a moving
    /// part too) left out.
    pub(super) fn own(g: &Glb, entry: &J) -> R<Option<Sampler>> {
        Self::without(g, entry, None, &aside_nodes(&g.gltf))
    }

    /// `new` with the parts `skip` (their nodes, or the moving parts they are on) left out.
    fn without(
        g: &Glb,
        entry: &J,
        pose: Option<&Pose>,
        skip: &HashSet<usize>,
    ) -> R<Option<Sampler>> {
        let materials = g.gltf.get("materials").and_then(J::list).unwrap_or(&[]);
        let no_overrides = J::Obj(Vec::new());
        let overrides = entry.get("materials").unwrap_or(&no_overrides);
        let shown = |i: Option<&J>| -> R<bool> {
            let name = match i {
                Some(i) => index(&J::Arr(materials.to_vec()), i)?
                    .get("name")
                    .map(J::key)
                    .unwrap_or_default(),
                None => String::new(),
            };
            let ov = override_of(overrides, &name);
            Ok(!(ov.truthy() && ov.get("hide").is_some_and(J::truthy)))
        };
        // (A part flattened to nothing: one of its axes under a hundredth of its longest, G.grow from 0.001.)
        let placed: Vec<Placed> = node_matrices(&g.gltf, pose, None)?
            .into_iter()
            .filter(|p| !skip.contains(&p.node) && !p.part.is_some_and(|q| skip.contains(&q)))
            .filter(|p| {
                let axis =
                    |k: usize| hypot(&[p.m[k * 4].f(), p.m[k * 4 + 1].f(), p.m[k * 4 + 2].f()]);
                let (short, long) = (0..3)
                    .map(axis)
                    .fold((f64::INFINITY, 0.0f64), |(s, l), a| (s.min(a), l.max(a)));
                short >= 0.01 * long
            })
            .collect();
        let fit = entry.get("fit").is_some_and(J::truthy);
        let scale = if fit {
            1.0
        } else {
            entry.get("scale").and_then(J::num).map_or(1.0, Num::f)
        };
        Ok(triangles(g, &placed, &shown)?.map(|tris| Sampler { tris, scale }))
    }

    /// The heights (m) of the faces on the vertical line at (x, depth) (game axes, m from the model's origin):
    /// those looking up, or every face.
    fn line(&self, x: f64, z: f64, up: bool) -> Vec<f64> {
        // glTF x -> game -x; the game's model is the file's times `scale`.
        let (p, d) = ([-x / self.scale, 0.0, z / self.scale], [0.0, -1.0, 0.0]);
        hits(&p, &d, &self.tris)
            .filter(|(_, n)| !up || n[1] > 0.0)
            .map(|(t, _)| -t * self.scale)
            .collect()
    }

    /// The top (m) at (x, depth) nearest `near`: of the faces looking up there, the one nearest that height (what
    /// lies on a top, a vase or a pillow, and what is under it aren't its top). If none looks up (inside out),
    /// any face. None: nothing there.
    pub(super) fn top(&self, x: f64, z: f64, near: f64) -> Option<f64> {
        self.top_in(x, z, [near, near])
    }

    /// `top` over the `PROBE` strip around (x, depth): the faces looking up at the point itself and on a ring round
    /// it, the one nearest `near` of those (a gap between slats has nothing on the line but its neighbours do).
    fn top_around(&self, x: f64, z: f64, near: f64) -> Option<f64> {
        // The point first, then the ring: a tie keeps the point's own face.
        let mut best = self.top(x, z, near);
        for (dx, dz) in [
            (1.0, 0.0),
            (-1.0, 0.0),
            (0.0, 1.0),
            (0.0, -1.0),
            (0.7, 0.7),
            (-0.7, 0.7),
            (0.7, -0.7),
            (-0.7, -0.7),
        ] {
            let found = self.top(x + dx * PROBE, z + dz * PROBE, near);
            let off = |m: Option<f64>| m.map_or(f64::INFINITY, |m| (m - near).abs());
            if found.is_some() && off(found) < off(best) {
                best = found;
            }
        }
        best
    }

    /// `top`, nearest the range `[lo, hi]` (of those in it, nearest its middle).
    pub(super) fn top_in(&self, x: f64, z: f64, [lo, hi]: [f64; 2]) -> Option<f64> {
        let off = |h: f64| ((lo - h).max(h - hi).max(0.0), (h - (lo + hi) / 2.0).abs());
        self.tops_at(x, z).into_iter().min_by(|a, b| {
            off(*a)
                .partial_cmp(&off(*b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// The heights `top` chooses from at (x, depth): the faces looking up, or every face if none does.
    pub(super) fn tops_at(&self, x: f64, z: f64) -> Vec<f64> {
        let found = self.line(x, z, true);
        if found.is_empty() {
            self.line(x, z, false)
        } else {
            found
        }
    }

    /// Where a seat ends in front (m forward of the footprint's middle): from the hips' point (x, depth) forward a
    /// centimetre at a time while the model reaches up to within `FRONT_DROP` of the seat's height `h` there (a
    /// cushion's top, a bean bag's bulge), the last such depth; None: nothing that high under the hips.
    pub(super) fn front(&self, x: f64, z: f64, h: f64) -> Option<f64> {
        let on = |z: f64| self.span(x, z).is_some_and(|(_, hi)| hi >= h - FRONT_DROP);
        if !on(z) {
            return None;
        }
        let mut at = z;
        while at < z + 1.5 && on(at + 0.01) {
            at += 0.01;
        }
        Some(at)
    }

    /// The lowest and the highest face (m) on the vertical line at (x, depth); None: nothing there.
    fn span(&self, x: f64, z: f64) -> Option<(f64, f64)> {
        let found = self.line(x, z, false);
        let lo = found.iter().copied().reduce(f64::min)?;
        Some((lo, found.into_iter().fold(lo, f64::max)))
    }

    /// A face's corners (m, game axes).
    fn corners(&self, t: &Tri) -> [[f64; 3]; 3] {
        let game = |v: [f64; 3]| [-v[0] * self.scale, v[1] * self.scale, v[2] * self.scale];
        let at = |e: &[f64; 3]| game([t.a[0] + e[0], t.a[1] + e[1], t.a[2] + e[2]]);
        [game(t.a), at(&t.e1), at(&t.e2)]
    }

    /// The box of its faces (m, game axes): (lowest corner, highest); None: no face.
    pub(super) fn extent(&self) -> Option<([f64; 3], [f64; 3])> {
        let mut corners = self.tris.iter().flat_map(|t| self.corners(t));
        let first = corners.next()?;
        Some(corners.fold((first, first), |(lo, hi), c| {
            (
                [0, 1, 2].map(|k| lo[k].min(c[k])),
                [0, 1, 2].map(|k| hi[k].max(c[k])),
            )
        }))
    }

    /// How high it reaches (m) without what stands beside its top `top` higher than it (a dining set's chairs'
    /// backs): the highest of its faces that are no more than `OVER_TOP` over the top, or whose middle, seen from
    /// above, is over the top (the box of the flat faces looking up within 1 cm of it: a vase, a screen on it).
    /// None: no face looks up there.
    pub(super) fn height_by(&self, top: f64) -> Option<f64> {
        const OVER_TOP: f64 = 0.02;
        let flat_at = |t: &Tri, c: &[[f64; 3]; 3]| {
            t.nrm[1] > 0.9 * hypot(&t.nrm) && c.iter().all(|v| (v[1] - top).abs() <= 0.01)
        };
        let mut area: Option<([f64; 2], [f64; 2])> = None;
        for t in &self.tris {
            let c = self.corners(t);
            if flat_at(t, &c) {
                for v in c {
                    let (lo, hi) = area.get_or_insert(([v[0], v[2]], [v[0], v[2]]));
                    *lo = [lo[0].min(v[0]), lo[1].min(v[2])];
                    *hi = [hi[0].max(v[0]), hi[1].max(v[2])];
                }
            }
        }
        let (lo, hi) = area?;
        let over = |c: &[[f64; 3]; 3]| {
            let (x, z) = (
                (c[0][0] + c[1][0] + c[2][0]) / 3.0,
                (c[0][2] + c[1][2] + c[2][2]) / 3.0,
            );
            lo[0] <= x && x <= hi[0] && lo[1] <= z && z <= hi[1]
        };
        self.tris
            .iter()
            .map(|t| self.corners(t))
            .filter(|c| c.iter().all(|v| v[1] <= top + OVER_TOP) || over(c))
            .flat_map(|c| c.map(|v| v[1]))
            .reduce(f64::max)
    }

    /// A stair's steps: on the line up its middle (across), front to back every centimetre, the highest face
    /// looking up (a tread, not the handrails at its sides); the heights of those 2 cm or more up, a centimetre
    /// apart or more, are its steps. (The top step's height, how many); None: none.
    pub(super) fn steps(&self) -> Option<(f64, usize)> {
        let (lo, hi) = self.extent()?;
        let x = (lo[0] + hi[0]) / 2.0;
        let mut found: Vec<f64> = (0..=((hi[2] - lo[2]) * 100.0) as usize)
            .filter_map(|k| {
                self.line(x, lo[2] + k as f64 / 100.0, true)
                    .into_iter()
                    .reduce(f64::max)
            })
            .filter(|h| *h >= 0.02)
            .collect();
        found.sort_by(f64::total_cmp);
        found.dedup_by(|a, b| *a - *b < 0.01);
        Some((*found.last()?, found.len()))
    }
}

/// Where the clip `clip` leaves the model's parts: each channel's last key (a bed pulled out or folded down,
/// held open: `motions.open`).
pub(super) fn open_pose(g: &Glb, clip: &J) -> R<Pose> {
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut pose: Pose = HashMap::new();
    for a in g
        .gltf
        .get("animations")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .filter(|a| a.get("name") == Some(clip))
    {
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let (Some(node), Some(path)) =
                (target.opt("node"), target.get("path").and_then(J::str))
            else {
                continue;
            };
            let Some(path) = ["translation", "rotation", "scale"]
                .into_iter()
                .find(|p| *p == path)
            else {
                continue;
            };
            if let Some(last) = first_value(g, key(sampler(a, c)?, "output")?, true)? {
                pose.insert(
                    (node_index(nodes, node)?, path),
                    last.into_iter().map(Num::F).collect(),
                );
            }
        }
    }
    Ok(pose)
}

/// A functional height of a content object (where residents sit or lie on it, where things stand on its top,
/// where its lamp shines from) and what its model has there (`heights`).
#[derive(Default)]
pub struct Height {
    /// `seat`, `surface`, `mattress`, `lower bunk`, `upper bunk` or `light`.
    pub what: &'static str,
    /// The height the game uses (m), and what says so (content `seat`, furniture.json `seats`, a default).
    pub expected: f64,
    pub source: String,
    /// Where it was measured: m to the right of the footprint's middle and forward of it (game axes).
    pub at: [f64; 2],
    /// The clip it was measured held open at (`motions.open`: a bed pulled out), if any.
    pub open: Option<String>,
    /// The model's top there (m; None: nothing there); for a light, its lowest and highest faces there.
    pub mesh: Option<f64>,
    pub span: Option<(f64, f64)>,
    /// What a second source of it says, when it says otherwise (content `seat` against furniture.json `seats`).
    pub disagrees: Option<String>,
}

impl Height {
    /// How far (m) the model is from it (for a light, how far outside the lamp it shines from); None: nothing there.
    pub fn off(&self) -> Option<f64> {
        if let Some((lo, hi)) = self.span {
            return Some(if self.expected < lo {
                lo - self.expected
            } else {
                f64::max(0.0, self.expected - hi)
            });
        }
        self.mesh.map(|m| (m - self.expected).abs())
    }

    /// Whether the model is within `HEIGHT_OFF` of it.
    pub fn fits(&self) -> bool {
        self.off().is_some_and(|off| off <= HEIGHT_OFF)
    }

    /// The gate's warnings about it: the model off it, and a second source that says otherwise.
    pub(super) fn warnings(&self) -> Vec<String> {
        let mut out = Vec::new();
        let open = self
            .open
            .as_ref()
            .map(|c| format!(", held open ({})", repr_str(c)))
            .unwrap_or_default();
        let at = format!("x {}, depth {}", fixed(self.at[0], 2), fixed(self.at[1], 2));
        if self.what == "seat front" {
            if self.off().is_none_or(|off| off > FRONT_OFF) {
                out.push(match self.mesh {
                    Some(m) => format!(
                        "seat front {} m forward ({}), the model's seat ends {} m forward there ({at}): {} cm {}",
                        fixed(self.expected, 2),
                        self.source,
                        fixed(m, 2),
                        fixed((m - self.expected).abs() * 100.0, 1),
                        if m < self.expected { "further back" } else { "further forward" }
                    ),
                    None => format!("seat front {} m forward ({}): no seat under the hips ({at})", fixed(self.expected, 2), self.source),
                });
            }
            return out;
        }
        if !self.fits() {
            out.push(match (self.mesh, self.span) {
                (_, Some((lo, hi))) => format!(
                    "{} {} m ({}), outside the lamp there (its model {} to {} m at {at})",
                    self.what,
                    fixed(self.expected, 2),
                    self.source,
                    fixed(lo, 2),
                    fixed(hi, 2)
                ),
                (Some(m), _) => format!(
                    "{} {} m ({}{open}), the model {} m there ({at}): {} cm {}",
                    self.what,
                    fixed(self.expected, 2),
                    self.source,
                    fixed(m, 3),
                    fixed((m - self.expected).abs() * 100.0, 1),
                    if m < self.expected { "lower" } else { "higher" }
                ),
                (None, None) => format!(
                    "{} {} m ({}{open}): no model under it ({at})",
                    self.what,
                    fixed(self.expected, 2),
                    self.source
                ),
            });
        }
        if let Some(d) = &self.disagrees {
            out.push(format!("{}: {d}", self.what));
        }
        out
    }
}

/// Where residents sit on a content object (`seat_of`): the seat's height (m), what says so, what a second source
/// says when it says otherwise, and the hips' points (m, game axes).
pub(super) struct Seat {
    h: f64,
    source: &'static str,
    disagrees: Option<String>,
    pub(super) points: Vec<[f64; 2]>,
    /// Where its top ends in front (furniture.json `seats` `front`, m forward of the middle), if the table says.
    front: Option<f64>,
}

/// Whether any of the object's uses is a baby's (content `baby`).
fn for_baby(object: &J) -> bool {
    object
        .get("interactions")
        .and_then(J::list)
        .is_some_and(|l| l.iter().any(|i| i.get("baby").is_some_and(J::truthy)))
}

/// Each slot's place of content object `object` (sim-core `slot_position`, `slot_yaw`), `fwd` m ahead of it the
/// way its user faces (m right of the footprint's middle and forward of it): its `places` (but those stood at,
/// `pose: "stand"`: the masseur's beside a massage table is no seat or mattress), else spread across its width,
/// facing its front.
fn slot_points(object: &J, fwd: f64) -> Vec<[f64; 2]> {
    if let Some(places) = object.get("places").and_then(J::list) {
        return places
            .iter()
            .filter(|p| p.get("pose").and_then(J::str) != Some("stand"))
            .map(|p| {
                let at = |k| {
                    p.get("at")
                        .and_then(|a| a.at(k))
                        .and_then(J::num)
                        .map_or(0.0, Num::f)
                };
                let facing = number(Some(p), "facing", 0.0).to_radians();
                [at(0) + facing.sin() * fwd, at(1) + facing.cos() * fwd]
            })
            .collect();
    }
    // A baby's place is the middle, however many slots it has (sim-core `place_at`: a baby is put at the object's
    // centre, not spread across its width), so a crib's mattress is one mattress and a high chair one seat.
    if for_baby(object) {
        return vec![[0.0, fwd]];
    }
    let n = object
        .get("slots")
        .and_then(J::num)
        .map_or(1, |n| n.f().max(1.0) as usize);
    let width = object
        .get("footprint")
        .and_then(|f| f.at(0))
        .and_then(J::num)
        .map_or(1.0, Num::f);
    (0..n)
        .map(|k| [(k as f64 - (n as f64 - 1.0) / 2.0) * width / n as f64, fwd])
        .collect()
}

/// The seat of content object `object` (furniture.json `seats`, else the content's `seat`, else `defaultSeat` for
/// one sat on), the hips' point `fwd` ahead of each slot's place (`slot_points`; along the object for a bath, `along`).
/// None: no one sits on it.
pub(super) fn seat_of(object: &J, furniture: &Furniture) -> Option<Seat> {
    // A thing that stands on a top (content `layer: "surface"`) has no seat of its own: a `sit` use of the mugs or
    // the books is the resident sitting at the table they stand on, not on them.
    if object.get("layer").is_some_and(|l| l.key() == "surface") {
        return None;
    }
    let id = object.get("id").map(J::key).unwrap_or_default();
    let own = object.opt("seat");
    let table = furniture.seats.get(&id);
    let sits = || {
        object
            .get("interactions")
            .and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .any(|i| i.get("pose").is_some_and(|p| p.key() == "sit"))
    };
    if own.is_none() && table.is_none() && !sits() {
        return None;
    }
    let (seat, source) = match (table, own) {
        (Some(t), _) => (
            (number(Some(t), "h", 0.45), number(Some(t), "fwd", 0.0)),
            "furniture.json `seats`",
        ),
        (None, Some(o)) => (
            (
                number(Some(o), "height", 0.45),
                number(Some(o), "forward", 0.0),
            ),
            "content `seat`",
        ),
        (None, None) => (
            (
                number(Some(&furniture.default_seat), "h", 0.45),
                number(Some(&furniture.default_seat), "fwd", 0.0),
            ),
            "furniture.json `defaultSeat`",
        ),
    };
    let disagrees = match (table, own) {
        (Some(_), Some(o)) => {
            let theirs = (
                number(Some(o), "height", 0.0),
                number(Some(o), "forward", 0.0),
            );
            ((theirs.0 - seat.0).abs() > SAME || (theirs.1 - seat.1).abs() > SAME).then(|| {
                format!(
                    "content `seat` says {} m, {} forward; furniture.json `seats` says {} m, {} forward (furniture.json wins: characters.ts `seatOf`)",
                    fixed(theirs.0, 2),
                    fixed(theirs.1, 2),
                    fixed(seat.0, 2),
                    fixed(seat.1, 3)
                )
            })
        }
        _ => None,
    };
    let along = table
        .and_then(|t| t.get("along"))
        .and_then(J::num)
        .map(Num::f);
    let points = match along {
        Some(a) => vec![[a * seat.1, 0.0]],
        None => slot_points(object, seat.1),
    };
    let front = table
        .and_then(|t| t.get("front"))
        .and_then(J::num)
        .map(Num::f);
    Some(Seat {
        h: seat.0,
        source,
        disagrees,
        points,
        front,
    })
}

/// The functional heights of content object `object` (`Facts::object`) drawn by the model `g` (its `entry`):
/// - its seat (`seat_of`), at each of its hips' points (the worst reported);
/// - its top (content `surface`: at its `at`, else the middle);
/// - its mattress (furniture.json `bunks`, `beds`, else `defaultBed` for one lain on; a hammock's sling at its
///   low point), `fwd` forward, at each slot (for one lain on across, `fwd` along it from its middle at the slot's depth, but a baby's at its slot), held open if it opens;
///   with `every_slot` false at slot 0 only (`cargo review sizes`: a crib's slot 1 is its changing table);
/// - its lamp's light (content `light`, only with an `at`: the shade).
pub(super) fn heights(
    g: &Glb,
    entry: &J,
    object: &J,
    furniture: &Furniture,
    every_slot: bool,
) -> R<Vec<Height>> {
    let mut out = Vec::new();
    let id = object.get("id").map(J::key).unwrap_or_default();
    let poses: HashSet<String> = object
        .get("interactions")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|i| i.get("pose").map(J::key))
        .collect();
    let seat = seat_of(object, furniture);
    let lain = furniture.beds.get(&id).is_some()
        || furniture.bunks.get(&id).is_some()
        || poses.contains("lie");
    let lamp = object.opt("light").is_some_and(|l| l.opt("at").is_some());
    if !(seat.is_some() || lain || lamp || object.opt("surface").is_some()) {
        return Ok(out);
    }
    // At rest (not `grown`: a tub's water grown to full would be its seat; so a yoga mat, rolled up at rest, has
    // nothing under its seat).
    let rest = Sampler::new(g, entry, None)?;
    // The worst of the points: the farthest off, nothing there worst of all.
    let worst = |s: &Sampler, points: &[[f64; 2]], near: f64| -> ([f64; 2], Option<f64>) {
        let mut best: Option<([f64; 2], Option<f64>)> = None;
        for p in points {
            let m = s.top_around(p[0], p[1], near);
            let off = |m: Option<f64>| m.map_or(f64::INFINITY, |m| (m - near).abs());
            if best.is_none_or(|(_, b)| off(m) > off(b)) {
                best = Some((*p, m));
            }
        }
        best.unwrap_or(([0.0, 0.0], None))
    };
    let mut height = |what: &'static str,
                      expected: f64,
                      source: String,
                      points: &[[f64; 2]],
                      s: &Sampler,
                      open: Option<String>,
                      disagrees: Option<String>| {
        let (at, mesh) = worst(s, points, expected);
        out.push(Height {
            what,
            expected,
            source,
            at,
            open,
            mesh,
            span: None,
            disagrees,
        });
    };

    // Seated (not on the ground: against a tree's trunk, nothing of the model's is sat on).
    let mut seat_front = None;
    if let Some(seat) = seat
        && let Some(s) = &rest
        && seat.h > GROUND
    {
        height(
            "seat",
            seat.h,
            seat.source.into(),
            &seat.points,
            s,
            None,
            seat.disagrees,
        );
        // Where its top ends in front (the shins clear of a deep cushion: characters.ts `perch`), the worst point.
        if let Some(front) = seat.front {
            let mut worst: Option<([f64; 2], Option<f64>)> = None;
            for p in &seat.points {
                let m = s.front(p[0], p[1], seat.h);
                let off = |m: Option<f64>| m.map_or(f64::INFINITY, |m| (m - front).abs());
                if worst.is_none_or(|(_, w)| off(m) > off(w)) {
                    worst = Some((*p, m));
                }
            }
            let (at, mesh) = worst.unwrap_or(([0.0, 0.0], None));
            seat_front = Some(Height {
                what: "seat front",
                expected: front,
                source: "furniture.json `seats`".into(),
                at,
                open: None,
                mesh,
                span: None,
                disagrees: None,
            });
        }
    }

    // On a top.
    if let Some(top) = object.opt("surface")
        && let Some(s) = &rest
    {
        let point = match top.get("at").and_then(J::list) {
            Some([x, z, ..]) => [x.num().map_or(0.0, Num::f), z.num().map_or(0.0, Num::f)],
            _ => [0.0, 0.0],
        };
        height(
            "surface",
            number(Some(top), "height", 0.0),
            "content `surface`".into(),
            &[point],
            s,
            None,
            None,
        );
    }

    // Lain on: held open if it opens (a sofa bed's mattress pulled out, a wall bed folded down).
    let bed = furniture.beds.get(&id);
    let bunk = furniture.bunks.get(&id).and_then(J::list);
    if lain {
        let none = J::Obj(Vec::new());
        let open = entry
            .get("motions")
            .filter(|m| m.truthy())
            .unwrap_or(&none)
            .opt("open")
            .cloned();
        let opened = match &open {
            Some(clip) => Sampler::new(g, entry, Some(&open_pose(g, clip)?))?,
            None => None,
        };
        let s = match &opened {
            Some(s) => Some(s),
            None => rest.as_ref(),
        };
        if let Some(s) = s {
            let fwd = number(bed, "fwd", 0.0);
            let open = open.map(|c| c.key());
            if let Some(tops) = bunk {
                for (what, h) in ["lower bunk", "upper bunk"].into_iter().zip(tops) {
                    height(
                        what,
                        h.num().map_or(0.0, Num::f),
                        "furniture.json `bunks`".into(),
                        &[[0.0, fwd]],
                        s,
                        open.clone(),
                        None,
                    );
                }
            } else {
                let sling = bed.and_then(|b| b.get("sling"));
                let (h, source) = match (bed, sling) {
                    (Some(_), Some(sl)) => (
                        number(Some(sl), "low", 0.5),
                        "furniture.json `beds`, its sling's low point",
                    ),
                    (Some(b), None) => (number(Some(b), "h", 0.5), "furniture.json `beds`"),
                    (None, _) => (
                        number(Some(&furniture.default_bed), "h", 0.5),
                        "furniture.json `defaultBed`",
                    ),
                };
                // (A baby lies at its slot, on its back along the bed, even on one lain on across: characters.ts.)
                let across =
                    !for_baby(object) && bed.and_then(|b| b.get("across")).is_some_and(J::truthy);
                let slots = slot_points(object, fwd);
                let n = if every_slot { slots.len() } else { 1 };
                let points: Vec<[f64; 2]> = if across {
                    // Lain on across, along it: moved to its middle from the slot, then `fwd` along it, at the
                    // slot's depth (residents.ts `bedFwd`).
                    let depth = slot_points(object, 0.0).first().map_or(0.0, |p| p[1]);
                    vec![[fwd, depth]]
                } else {
                    slots.into_iter().take(n).collect()
                };
                height("mattress", h, source.into(), &points, s, open, None);
            }
        }
    }

    // A lamp's light, at its shade.
    if let Some(light) = object.opt("light").filter(|_| lamp)
        && let Some([x, z, ..]) = light.get("at").and_then(J::list)
        && let Some(s) = &rest
    {
        let at = [x.num().map_or(0.0, Num::f), z.num().map_or(0.0, Num::f)];
        let span = s.span(at[0], at[1]);
        out.push(Height {
            what: "light",
            expected: number(Some(light), "height", 1.55),
            source: "content `light`".into(),
            at,
            open: None,
            mesh: None,
            span,
            disagrees: None,
        });
    }
    out.extend(seat_front);
    Ok(out)
}
