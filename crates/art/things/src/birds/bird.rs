//! One bird built from its numbers, part by part, keeping the points the parts meet at. Built in a side view: f
//! forward (Blender -y), u up (z), x to the bird's left (Blender +x, the game's -x).

use super::common::{
    BODY_K, BODY_T, STATIONS, body_ring, ease, head_ring, interp, loft, mid, octahedron, p, reach,
    side, toe, tube,
};
use super::species::Species;
use artkit::materials::{Mat, Palette};
use artkit::math::{PI, V3, cos, radians, sin, sqrt, v3};
use artkit::parts::{join, new_parts};
use artkit::scene::{self, Obj};

/// `perch_height`: how far the body's middle stands over the feet (the entry's `bird.perchHeight`).
pub fn perch_height(s: &Species) -> f64 {
    s.leg + s.depth * 0.47
}

/// `Bird`: one bird of `S` (a species' numbers, metres and degrees) in its palette.
pub struct Bird<'a> {
    s: &'a Species,
    m: Palette,
    /// (x, f, u): the body's axis, rump to breast
    a: V3,
    /// square to it, up
    n: V3,
    /// the body's middle over the feet
    c: V3,
    /// points round the body, the head, a spread wing
    rings: (usize, usize, usize),
    tail_len: f64,
    tail_dir: V3,
    tail_up: V3,
    tail_base: V3,
    shoulder: V3,
    head_centre: V3,
    bill_tip: V3,
}

impl<'a> Bird<'a> {
    pub fn new(s: &'a Species, m: Palette) -> Bird<'a> {
        let pitch = radians(s.pitch as f64);
        Bird {
            s,
            m,
            a: v3(0.0, cos(pitch), sin(pitch)),
            n: v3(0.0, -sin(pitch), cos(pitch)),
            c: v3(0.0, s.body_at, perch_height(s)),
            rings: s.rings,
            tail_len: 0.0,
            tail_dir: V3::ZERO,
            tail_up: V3::ZERO,
            tail_base: V3::ZERO,
            shoulder: V3::ZERO,
            head_centre: V3::ZERO,
            bill_tip: V3::ZERO,
        }
    }

    fn mat(&self, zone: &str) -> Mat {
        self.m.get(zone)
    }

    /// (x, f, u) -> Blender
    fn b(v: V3) -> V3 {
        p(v.x, v.y, v.z)
    }

    /// A point on the body's surface at `t` (rump 0 .. neck 1) and `deg` round from the top, `out` metres off it;
    /// `smooth`: its fullness eased between the stations (the surface), not straight (the joints, as they were).
    fn body_point(&self, t: f64, deg: f64, out: f64, smooth: bool) -> V3 {
        let s = self.s;
        let k = if smooth {
            ease(&BODY_T, &BODY_K, t)
        } else {
            interp(&BODY_T, &BODY_K, t)
        };
        let th = radians(deg);
        let centre = self.c + self.a * (s.body * (t - 0.5));
        let (hw, hh) = (s.width / 2.0 * k, s.depth / 2.0 * k);
        // a fuller breast and belly
        let sag = -self.n * hh * 0.1 * sin(PI * 1.0f64.min(0.0f64.max(t)));
        let normal = (self.n * (cos(th) / hh.max(1e-6)) + v3(1, 0, 0) * (sin(th) / hw.max(1e-6)))
            .normalized();
        centre + sag + self.n * (hh * cos(th)) + v3(hw * sin(th), 0.0, 0.0) + normal * out
    }

    /// The torso: rings eased from the rump round to the neck, every face the zone it is in (`zones.body`).
    fn torso(&self) -> Obj {
        let s = self.s;
        let ts: &[f64] = s.stations.unwrap_or(&STATIONS);
        let ring = body_ring(self.rings.0);
        let n = ring.len();
        let rings: Vec<Vec<V3>> = ts
            .iter()
            .map(|&t| {
                ring.iter()
                    .map(|&d| Bird::b(self.body_point(t, d, 0.0, true)))
                    .collect()
            })
            .collect();
        let at = |i: i64| ts[0.max(i.min(ts.len() as i64 - 1)) as usize];
        let zone = |i: i64, j: usize| {
            let t = (at(i) + at(i + 1)) / 2.0;
            let deg = side(mid(ring[j], ring[(j + 1) % n]));
            self.mat((s.zones.body)(t, deg))
        };
        let rump = Bird::b(self.c + self.a * (-s.body * 0.52));
        loft("torso", &rings, zone, Some(rump), None)
    }

    /// A wing folded on the body's upper side, from the shoulder back over the rump to its tip over the tail: a lens
    /// of five points round (its lower edge, two on its outer face, its upper edge, one against the body), coloured
    /// band by band from the shoulder (`zones.folded`: the outer face's colour and its lower edge's).
    fn folded_wing(&self, side_: i32) -> Obj {
        let s = self.s;
        let st = [
            (0.78, 60.0, 0.12),
            (0.68, 63.0, 0.25),
            (0.58, 62.0, 0.29),
            (0.5, 61.0, 0.29),
            (0.38, 57.0, 0.27),
            (0.18, 45.0, 0.18),
        ];
        let sf = side_ as f64;
        let mut rings: Vec<Vec<V3>> = Vec::new();
        for (t, deg, w) in st {
            let centre = self.body_point(t, deg, s.depth * 0.03, false);
            let outward = (self.body_point(t, deg, 1.0, false)
                - self.body_point(t, deg, 0.0, false))
            .normalized();
            let along = (self.body_point(t, deg + 8.0, 0.0, false)
                - self.body_point(t, deg - 8.0, 0.0, false))
            .normalized();
            let (half, thick) = (s.depth * w, s.depth * 0.06);
            let ring = [
                centre - along * half,
                centre - along * half * 0.45 + outward * thick,
                centre + along * half * 0.4 + outward * thick,
                centre + along * half,
                centre - outward * thick * 0.5,
            ];
            rings.push(
                ring.iter()
                    .map(|v| Bird::b(v3(v.x as f64 * sf, v.y, v.z)))
                    .collect(),
            );
        }
        let tip = self.tail_base
            + self.tail_dir * (self.tail_len * s.tip_over)
            + self.tail_up * s.width * 0.09;
        let tip = Bird::b(v3(sf * s.width * 0.1, tip.y, tip.z));
        // (outer face, lower edge) per band: shoulder .. tip
        let bands = &s.zones.folded;
        let zone = |i: i64, j: usize| {
            let (main, edge) = bands[0.max(i.min(bands.len() as i64 - 1)) as usize];
            self.mat(if j == 0 { edge } else { main })
        };
        loft(&format!("folded{side_}"), &rings, zone, None, Some(tip))
    }

    /// Each leg: a three-sided shank down to the foot, three toes spread forward and one back (to grip).
    fn legs(&self) -> Vec<Obj> {
        let s = self.s;
        let mut parts = Vec::new();
        for side_ in [-1, 1] {
            let x = side_ as f64 * s.width * 0.17;
            let r = s.leg_r;
            let top = Bird::b(v3(x, 0.0, s.leg + s.depth * 0.25));
            let foot = Bird::b(v3(x, 0.0, r));
            parts.push(tube(
                &format!("leg{side_}"),
                &[top, foot],
                r,
                r * 0.9,
                self.mat("leg"),
            ));
            let toe_ = s.toe;
            let heel = v3(x, 0.0, r);
            for (k, (yaw, reach_)) in [(-24, 0.85), (0, 1.0), (24, 0.85), (180, 0.6)]
                .into_iter()
                .enumerate()
            {
                let a = radians((yaw * side_) as f64);
                let d = v3(sin(a), cos(a), 0);
                parts.push(toe(
                    &format!("toe{side_}{k}"),
                    Bird::b(heel),
                    Bird::b(heel + d * toe_ * reach_ - v3(0.0, 0.0, r * 0.4)),
                    r * 0.85,
                    self.mat("leg"),
                ));
            }
        }
        parts
    }

    /// The head (a lofted ball), its bill and its eyes; returns them and the neck joint (Blender).
    fn head(&mut self) -> (Vec<Obj>, V3) {
        let s = self.s;
        let big_r = s.head;
        let neck = self.c + self.a * (s.body * 0.5);
        let up = radians(s.neck_up);
        let hc = neck + v3(0.0, cos(up), sin(up)) * big_r * s.neck;
        self.head_centre = hc;
        let ss = [-0.78, -0.42, -0.02, 0.38, 0.7];
        let ring = head_ring(self.rings.1);
        let n = ring.len();
        let at = |sv: f64, deg: f64, out: f64| {
            let q = sqrt(0.04f64.max(1.0 - sv * sv));
            let th = radians(deg);
            // (a little flatter than round, over and under: the crown as high as the old eight-sided head's)
            let (hw, hh) = (big_r * 0.9 * q, big_r * q * 0.86);
            hc + v3(
                hw * sin(th) * (1.0 + out),
                sv * big_r,
                hh * cos(th) * (1.0 + out),
            )
        };
        let rings: Vec<Vec<V3>> = ss
            .iter()
            .map(|&sv| ring.iter().map(|&d| Bird::b(at(sv, d, 0.0))).collect())
            .collect();
        let back = Bird::b(hc + v3(0.0, -big_r, 0.05 * big_r));
        let front = Bird::b(hc + v3(0.0, big_r * 0.96, -0.1 * big_r));
        let mut zs = vec![(-1.0 + ss[0]) / 2.0];
        zs.extend((0..ss.len() - 1).map(|i| (ss[i] + ss[i + 1]) / 2.0));
        zs.push((ss[ss.len() - 1] + 1.0) / 2.0);
        let zone = |i: i64, j: usize| {
            let deg = side(mid(ring[j], ring[(j + 1) % n]));
            self.mat((s.zones.head)(zs[(i + 1) as usize], deg))
        };
        let mut parts = vec![loft("skull", &rings, zone, Some(back), Some(front))];
        // The bill: from the face to its point, tipped down a little, its ridge (the culmen) curving down to the tip.
        let base = hc + v3(0.0, big_r * 0.78, -big_r * s.bill_low);
        let drop = radians(s.bill_droop);
        let fwd = v3(0.0, cos(drop), -sin(drop));
        let dn = v3(0.0, sin(drop), cos(drop));
        let tipv = base + fwd * (s.bill + big_r * 0.2);
        let (bw, bh) = (s.bill_w / 2.0, s.bill_h / 2.0);
        let mid_ = base + fwd * ((s.bill + big_r * 0.2) * 0.45) + dn * bh * 0.08;
        let bill_ring = |c: V3, f: f64| -> Vec<V3> {
            [
                c + v3(bw * f, 0.0, -bh * 0.15 * f),
                c + dn * (bh * f) + v3(bw * 0.55 * f, 0.0, 0.0),
                c + dn * (bh * 1.08 * f),
                c + dn * (bh * f) - v3(bw * 0.55 * f, 0.0, 0.0),
                c - v3(bw * f, 0.0, bh * 0.15 * f),
                c - dn * (bh * 0.85 * f),
            ]
            .iter()
            .map(|&v| Bird::b(v))
            .collect()
        };
        let bill = self.mat("bill");
        parts.push(loft(
            "bill",
            &[bill_ring(base, 1.0), bill_ring(mid_, 0.58)],
            |_, _| bill,
            None,
            Some(Bird::b(tipv)),
        ));
        self.bill_tip = tipv;
        // The eyes, on the band through them (a blackbird's in a yellow ring).
        for side_ in [-1, 1] {
            let sf = side_ as f64;
            let e = at(s.eye_at, 88.0, 0.0);
            let e = v3(sf * e.x as f64, e.y, e.z);
            let out = v3(sf, 0.25, 0.1).normalized();
            let r = big_r * s.eye;
            if s.ring {
                parts.push(octahedron(
                    &format!("ring{side_}"),
                    Bird::b(e - out * r * 0.55),
                    r * 1.5,
                    self.mat("ring"),
                    0.6,
                ));
            }
            parts.push(octahedron(
                &format!("eye{side_}"),
                Bird::b(e - out * r * 0.25),
                r,
                self.mat("eye"),
                0.6,
            ));
        }
        (parts, Bird::b(neck))
    }

    /// The tail from under the rump, back and down; returns it and its base (Blender). A flat fan of six points round
    /// (its outer feathers' edges, the top and the underside), its end cut square and notched, or rounded.
    fn tail(&self) -> (Obj, V3) {
        let s = self.s;
        let (d, up) = (self.tail_dir, self.tail_up);
        let base = self.tail_base;
        let big_t = self.tail_len;
        let big_w = s.width;
        // Narrow and deep at the base (the coverts over and under it), spreading and thinning to its end: `tail`
        // [(along it, half its width, its thickness), in body widths]; `tailEnd` how much shorter its middle (a
        // notch) or its outer feathers (a rounded end) are, in tail lengths.
        let notch = s.tail_end;
        let mut rings: Vec<Vec<V3>> = Vec::new();
        let last = s.tail.len() - 1;
        for (i, &(f, w, th)) in s.tail.iter().enumerate() {
            let (mut fo, mut fm) = (f, f);
            if i == last {
                (fo, fm) = if notch > 0.0 {
                    (f, f - notch)
                } else {
                    (f + notch, f)
                };
            }
            let (co, cm) = (base + d * (big_t * fo), base + d * (big_t * fm));
            let x = v3(big_w * w, 0.0, 0.0);
            rings.push(vec![
                Bird::b(co + x),
                Bird::b(cm + x * 0.62 + up * big_w * th),
                Bird::b(cm - x * 0.62 + up * big_w * th),
                Bird::b(co - x),
                Bird::b(cm - x * 0.62 - up * big_w * th * 0.6),
                Bird::b(cm + x * 0.62 - up * big_w * th * 0.6),
            ]);
        }
        let tz = s.zones.tail.unwrap_or(|_, _| "tail");
        let along = |i: i64| s.tail[0.max(i.min(last as i64)) as usize].0;
        let zone = |i: i64, j: usize| {
            let k = (along(i) + along(i + 1)) / 2.0;
            self.mat(tz(
                k,
                if j == 1 {
                    "top"
                } else if j == 4 {
                    "under"
                } else {
                    "outer"
                },
            ))
        };
        (loft("tail", &rings, zone, None, None), Bird::b(base))
    }

    /// A wing spread for flight, level, from the shoulder out to `side` (1: the bird's left, Blender +x): an aerofoil
    /// lofted out through the `planform`, its trailing edge scalloped by the feathers' ends, its top in bands front
    /// to back (the coverts, the greater coverts' tips where a wing bar is, the flight feathers).
    fn wing(&self, side_: i32) -> Obj {
        let s = self.s;
        let sh = self.shoulder;
        let span = s.wingspan / 2.0 - (sh.x as f64).abs();
        let big_c = s.chord;
        // points under it: 1 or 2
        let under = self.rings.2 - 4;
        let rows = s.planform;
        let mut rings: Vec<Vec<V3>> = Vec::new();
        for &(u, lead, trail) in rows {
            let x = (sh.x as f64).abs() + span * u;
            let th = big_c * (0.11 * (1.0 - u) + 0.022);
            let (l, t_) = (lead * big_c, trail * big_c);
            let cut = |f: f64| l + (t_ - l) * f;
            let mut pts = vec![
                (l, 0.0),
                (cut(0.3), th * 0.72),
                (cut(0.44), th * 0.6),
                (t_, 0.0),
            ];
            if under == 1 {
                pts.push((cut(0.6), -th * 0.22));
            } else {
                pts.extend([(cut(0.66), -th * 0.18), (cut(0.3), -th * 0.3)]);
            }
            rings.push(
                pts.iter()
                    .map(|&(f, z)| Bird::b(v3(side_ as f64 * x, sh.y as f64 + f, sh.z as f64 + z)))
                    .collect(),
            );
        }
        let wz = s.zones.wing;
        let mut parts = vec!["lead", "bar", "flight"];
        parts.extend(std::iter::repeat_n("under", under + 1));
        let zone = |i: i64, j: usize| {
            let i = 0.max(i.min(rows.len() as i64 - 2)) as usize;
            self.mat(wz((rows[i].0 + rows[i + 1].0) / 2.0, parts[j]))
        };
        loft(&format!("wing{side_}"), &rings, zone, None, None)
    }

    pub fn build(mut self) -> Vec<Obj> {
        let s = self.s;
        let drop = radians(s.tail_down);
        self.tail_dir = v3(0.0, -cos(drop), -sin(drop));
        self.tail_up = v3(0.0, -sin(drop), cos(drop));
        self.tail_base =
            self.body_point(0.07, 180.0, -s.depth * 0.12, false) + self.n * s.depth * 0.12;
        self.tail_base.x = 0.0;
        self.shoulder = self.body_point(0.72, 62.0, 0.0, false);
        let mut neck = V3::ZERO;
        let head = new_parts(|| {
            neck = self.head().1;
        });
        // The tail reaches as far back as makes the bird its length, bill tip to tail tip.
        self.tail_len = reach(self.bill_tip, self.tail_base, self.tail_dir, s.length);
        let body = new_parts(|| {
            self.torso();
            self.folded_wing(1);
            self.folded_wing(-1);
            self.legs();
        });
        let body = join(&body, "body", Some(artkit::math::P3::ZERO));
        let head = join(&head, "head", Some(neck.into()));
        let mut base = V3::ZERO;
        let tail = new_parts(|| {
            base = self.tail().1;
        });
        let tail = join(&tail, "tail", Some(base.into()));
        let mut wings = Vec::new();
        for (side_, name) in [(1, "wing.L"), (-1, "wing.R")] {
            let w = self.wing(side_);
            let sh = self.shoulder;
            wings.push(join(
                &[w],
                name,
                Some(Bird::b(v3(side_ as f64 * (sh.x as f64).abs(), sh.y, sh.z)).into()),
            ));
        }
        scene::log(format!(
            "[gamekit] {}: tail {:.1} cm, body middle {:.1} cm up, head {:.1} cm up",
            s.name,
            self.tail_len * 100.0,
            self.c.z as f64 * 100.0,
            self.head_centre.z as f64 * 100.0
        ));
        let mut out = vec![body, head, tail];
        out.extend(wings);
        out
    }
}
