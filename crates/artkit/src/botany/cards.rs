//! Leaf cards (the `leafcards` finish). A broadleaved tree's leaf masses are drawn as cut-out cards over a dark
//! core, not as lumpy balls: each card a quad carrying a cluster of its species' leaves from the one atlas the trees
//! share (`ATLAS`: a few tiles a species),
//! lying as a spray of leaves does, its normals the mass's hull's (not its own flat one), so the crown still shades
//! as a volume. The game draws the finish cut out by the atlas's alpha, from both sides; the file's materials stay
//! one-sided. A build makes its masses as ever (`leaves`: the lumps, kept out of the model, for what is found on
//! them by casting rays: perches, fruit, blooms) and `clad` draws them as cards at the end, in the lumps' box to
//! the millimetre, so the tree stands where it stood (`parts::settle`) with its perches. A far version keeps the
//! lumps.

use super::lump::{Lump, lump};
use crate::far::is_low;
use crate::materials::{Mat, Palette};
use crate::math::{self, P3, PI, TAU, V3, cos, pow, radians, sin, sqrt, v3};
use crate::mesh::Mesh;
use crate::normals::custom_from_verts;
use crate::parts::{join, measure};
use crate::py;
use crate::rand::Random;
use crate::scene::{self, Obj, meshes};
use crate::shapes::options;
use std::cell::RefCell;

/// An atlas in quarters, as the first was (three clusters of one leaf and a dense tile): each cluster's corner (u, v
/// from the bottom), 0.5 across. `Tiles::default`: what a crown wears that says nothing.
pub const CARD_TILES: [(f64, f64); 3] = [(0.0, 0.5), (0.5, 0.5), (0.0, 0.0)];
/// The dense, opaque wall of leaves, in the middle 0.8 of its tile: the cores'.
pub const CORE_TILE: (f64, f64) = (0.5, 0.0);
/// A card draws one of this many pictures: its species' clusters in turn, should it have fewer or more. (Each card
/// draws its tile from the cards' random numbers: drawn from another count, every card after it would lie
/// otherwise.)
pub const CARD_DRAWS: i64 = 3;

/// The trees' atlas (web/public/assets/textures/leafcards/albedo.png, which `cargo tex leafcards` draws from this
/// table): `ATLAS_PX` pixels a side, `ATLAS_GRID` tiles of `TILE_PX` a row and a column, `ATLAS_EDGE` pixels in
/// from the picture's top left.
pub const ATLAS_PX: usize = 1024;
pub const ATLAS_GRID: usize = 6;
pub const TILE_PX: usize = 170;
pub const ATLAS_EDGE: usize = 2;

/// What a tile of the atlas pictures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Picture {
    /// A cluster of leaves on their twigs, cut out.
    Cluster,
    /// A cluster of blossom with a few young leaves, cut out.
    Blossom,
    /// Leaves down hanging shoots, cut out, as tall as a hanging card (`Cards::hang`) and drawn as wide as it is
    /// tall: the card draws it narrow again.
    Strand,
    /// A dense wall of leaves, opaque, in the middle 0.8 of its tile: the cores'.
    Core,
}

/// The atlas's tiles, a row at a time from the top left: whose it is, its leaf's shape (a class of
/// docs/review/reference.json's species) and what it pictures.
pub const ATLAS: [(&str, &str, Picture); ATLAS_GRID * ATLAS_GRID] = {
    use Picture::{Blossom, Cluster, Core, Strand};
    [
        ("oak", "lobed", Cluster),
        ("oak", "lobed", Cluster),
        ("oak", "lobed", Cluster),
        ("oak", "lobed", Core),
        ("maple", "palmate", Cluster),
        ("maple", "palmate", Cluster),
        ("maple", "palmate", Cluster),
        ("maple", "palmate", Core),
        ("birch", "serrate-small", Cluster),
        ("birch", "serrate-small", Cluster),
        ("birch", "serrate-small", Cluster),
        ("birch", "serrate-small", Core),
        ("cherry", "oval", Blossom),
        ("cherry", "oval", Blossom),
        ("cherry", "oval", Blossom),
        ("cherry", "oval", Core),
        ("apple", "oval", Cluster),
        ("apple", "oval", Cluster),
        ("apple", "oval", Cluster),
        ("apple", "oval", Core),
        ("japaneseMaple", "palmate", Cluster),
        ("japaneseMaple", "palmate", Cluster),
        ("japaneseMaple", "palmate", Cluster),
        ("japaneseMaple", "palmate", Core),
        ("magnolia", "obovate-large", Cluster),
        ("magnolia", "obovate-large", Cluster),
        ("magnolia", "obovate-large", Cluster),
        ("magnolia", "obovate-large", Core),
        ("willow", "lance", Cluster),
        ("willow", "lance", Cluster),
        ("willow", "lance", Strand),
        ("willow", "lance", Strand),
        ("willow", "lance", Core),
        ("lemon", "oval", Cluster),
        ("lemon", "oval", Cluster),
        ("lemon", "oval", Cluster),
    ]
};

/// Tile `i` of the atlas (`ATLAS`'s order): its left, top, width and height in pixels.
pub fn tile_px(i: usize) -> [usize; 4] {
    [
        ATLAS_EDGE + i % ATLAS_GRID * TILE_PX,
        ATLAS_EDGE + i / ATLAS_GRID * TILE_PX,
        TILE_PX,
        TILE_PX,
    ]
}
/// A card's UVs stop this far inside its tile: no mipmap of its edge reaches the next.
pub const CARD_INSET: f64 = 0.03;
/// How much of its own facing a card's normals keep: sprays differ a little.
pub const CARD_OWN: f64 = 0.25;
/// How far a card's plane turns from straight out of its mass to straight up.
pub const CARD_UP: f64 = 0.55;
/// A card stays within this of its mass's ellipsoid (the lump's noise reaches as far).
pub const CARD_REACH: f64 = 1.1;
/// A card on top of its mass lies this much farther out: the crown keeps its height.
pub const CARD_TOP: f64 = 0.25;
/// Cards a leaf mass.
pub const CARDS: usize = 15;
/// A core's radius, of its masses' mean.
pub const CORE: f64 = 0.8;
/// Each leaf colour's cards (the palette's names): a card takes its mass's shade.
pub const CARD_OF: [(&str, &str); 7] = [
    ("leaf", "card"),
    ("leaf2", "card2"),
    ("leaf3", "card3"),
    ("leaf4", "card4"),
    ("leaf5", "card5"),
    ("green", "cardg"),
    ("green2", "cardg2"),
];

/// The cards' colour for the leaf colour `leaf` (`CARD_OF`).
pub fn card_of(leaf: &str) -> &'static str {
    match CARD_OF.iter().find(|(name, _)| *name == leaf) {
        Some((_, card)) => card,
        None => panic!("no cards for the leaf colour {leaf:?}"),
    }
}

/// Where a species' pictures are in the atlas: each cluster's corner (u, v from the bottom, `across` wide and
/// tall), the dense tile's, and, where it has them, the clusters its green among blossom wears (the `green` leaf
/// colours' cards) and the ones its hanging cards do (`Cards::hang`); none: its clusters. A quartered atlas's by
/// default; a species takes its own from the trees' atlas (`Tiles::of`).
#[derive(Clone, Debug, PartialEq)]
pub struct Tiles {
    pub cards: Vec<(f64, f64)>,
    pub green: Vec<(f64, f64)>,
    pub strands: Vec<(f64, f64)>,
    pub core: (f64, f64),
    pub across: f64,
}

impl Default for Tiles {
    fn default() -> Tiles {
        Tiles {
            cards: CARD_TILES.to_vec(),
            green: Vec::new(),
            strands: Vec::new(),
            core: CORE_TILE,
            across: 0.5,
        }
    }
}

impl Tiles {
    /// `species`' tiles of the trees' atlas (`ATLAS`).
    pub fn of(species: &str) -> Tiles {
        let corner = |i: usize| {
            let [x, y, _, tall] = tile_px(i);
            (
                x as f64 / ATLAS_PX as f64,
                1.0 - (y + tall) as f64 / ATLAS_PX as f64,
            )
        };
        let all = |picture: &[Picture]| -> Vec<(f64, f64)> {
            (0..ATLAS.len())
                .filter(|&i| ATLAS[i].0 == species && picture.contains(&ATLAS[i].2))
                .map(corner)
                .collect()
        };
        let cards = all(&[Picture::Cluster, Picture::Blossom]);
        assert!(!cards.is_empty(), "the atlas has no {species:?}");
        Tiles {
            cards,
            green: Vec::new(),
            strands: all(&[Picture::Strand]),
            // (a species with no dense tile of its own says whose it wears: `core_of`)
            core: all(&[Picture::Core]).first().copied().unwrap_or(CORE_TILE),
            across: TILE_PX as f64 / ATLAS_PX as f64,
        }
    }

    /// With `species`' dense tile under the cards.
    pub fn core_of(self, species: &str) -> Tiles {
        Tiles {
            core: Tiles::of(species).core,
            ..self
        }
    }

    /// With `species`' clusters on its green among blossom.
    pub fn green_of(self, species: &str) -> Tiles {
        Tiles {
            green: Tiles::of(species).cards,
            ..self
        }
    }
}

/// What masses share a core: a name and a number (a tree's `("clump", 3)`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group(pub String, pub usize);

impl<S: AsRef<str>> crate::Arg<Option<Group>> for (S, usize) {
    fn arg(self) -> Option<Group> {
        Some(Group(self.0.as_ref().to_string(), self.1))
    }
}

/// A leaf mass, until `clad` draws it.
#[derive(Clone, Debug)]
struct Mass {
    obj: Obj,
    centre: V3,
    half: V3,
    top: V3,
    material: &'static str,
    group: Option<Group>,
    cards: usize,
    core: bool,
    seed: f64,
}

thread_local! {
    /// The leaf masses of the build in hand, until `clad` draws them.
    static MASSES: RefCell<Vec<Mass>> = const { RefCell::new(Vec::new()) };
    /// What else stands in for cards till then (the willow's ribbons).
    static STAND_INS: RefCell<Vec<Obj>> = const { RefCell::new(Vec::new()) };
}

/// Forgets the leaf masses and stand-ins of a build that was left unclad (before the scene is emptied for the
/// next).
pub fn clear() {
    MASSES.with(|m| m.borrow_mut().clear());
    STAND_INS.with(|s| s.borrow_mut().clear());
}

options! {
    /// `leaves`' options: `lump`'s, then the mass's group, its count of cards and whether it has a core.
    Leaves { scale: P3 = P3::ONE, seed: f64 = 0.0, noise: f64 = 0.25, segments: usize = 10, rings: usize = 6, group: Option<Group> = None, cards: usize = CARDS, core: bool = true }
}

/// A leaf mass: a lump (`lump`, its arguments), which `clad` draws as `cards` leaf cards over a core (one for all
/// the masses of a `group`; none with `core` off: a spray at a bough's tip). Until then the lump stands in the
/// scene, out of the model (`join_still`), for the rays that find perches, fruit and blooms on the crown. A far
/// version keeps it.
pub fn leaves(name: impl AsRef<str>, r: f64, at: impl Into<V3>, material: Mat, o: Leaves) -> Obj {
    let at: V3 = at.into();
    let scale = o.scale;
    let obj = lump(
        name,
        r,
        at,
        material,
        Lump {
            scale,
            seed: o.seed,
            noise: o.noise,
            segments: o.segments,
            rings: o.rings,
        },
    );
    if !is_low() {
        let half = v3(r * scale.x, r * scale.y, r * scale.z);
        let top = obj.read(|part| {
            let mut top = part.mesh.verts[0];
            for &p in &part.mesh.verts {
                if p.z > top.z {
                    top = p;
                }
            }
            top
        });
        MASSES.with(|m| {
            m.borrow_mut().push(Mass {
                obj,
                centre: at + v3(0, 0, half.z),
                half,
                top,
                material: material.name,
                group: o.group,
                cards: o.cards,
                core: o.core,
                seed: o.seed,
            })
        });
    }
    obj
}

/// Keeps `obj` out of the model as the leaf masses are, for the shape `clad`'s cards are to have (`more`): the
/// rays meet it and the cards stay in its box. A far version keeps it.
pub fn stand_in(obj: Obj) -> Obj {
    if !is_low() {
        STAND_INS.with(|s| s.borrow_mut().push(obj));
    }
    obj
}

/// The leaf masses' lumps, then the stand-ins.
fn stood_in() -> Vec<Obj> {
    let mut out: Vec<Obj> = MASSES.with(|m| m.borrow().iter().map(|m| m.obj).collect());
    out.extend(STAND_INS.with(|s| s.borrow().clone()));
    out
}

/// `parts::join_all`, but for what `clad` is to draw as cards (`leaves`, `stand_in`): the still part so far.
pub fn join_still(name: &str) -> Obj {
    let out = stood_in();
    let still: Vec<Obj> = meshes().into_iter().filter(|o| !out.contains(o)).collect();
    join(&still, name, Some(P3::ZERO))
}

/// A card in the making: its corners, their normals and UVs, and its shading.
#[derive(Clone, Debug)]
pub struct Quad {
    pub corners: [V3; 4],
    pub normals: [V3; 4],
    pub uvs: [(f64, f64); 4],
    pub shade: f64,
}

options! {
    /// `Cards::over`'s options.
    Over { tilt: (f64, f64) = (0.0, 20.0), inside: f64 = 0.35, push: (f64, f64) = (0.6, 0.86), blend: f64 = 0.4, under: f64 = -0.45 }
}

/// Leaf cards in the making: `over` each mass, then `build` one mesh a material.
pub struct Cards {
    pub name: String,
    /// The cards' own random numbers (what a tree draws its own cards with draws from it too).
    pub rng: Random,
    pub lo: [f64; 3],
    pub hi: [f64; 3],
    /// The cards so far, by material, in the order each was first used.
    pub by: Vec<(Mat, Vec<Quad>)>,
    /// Where the cards' pictures are in the atlas (a quartered one's, or a species' own).
    pub tiles: Tiles,
}

impl Cards {
    /// `lo`, `hi`: the box no card leaves, and `fill` has them reach (the crown's, as its lumps drew it: the model
    /// keeps its size).
    pub fn new(name: &str, seed: i128, lo: [f64; 3], hi: [f64; 3]) -> Cards {
        Cards {
            name: name.to_string(),
            rng: Random::new(seed),
            lo,
            hi,
            by: Vec::new(),
            tiles: Tiles::default(),
        }
    }

    /// A card `size` across at `at`, its plane facing `facing`, turned at random in it, moved to where
    /// `fit(at, corners' offsets)` has it; `hull(point)` gives the normal at each corner, `shade` (0..1) its baked
    /// shading.
    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        at: impl Into<V3>,
        facing: impl Into<V3>,
        size: f64,
        material: Mat,
        hull: impl Fn(V3) -> V3,
        shade: f64,
        fit: impl Fn(V3, &[V3]) -> V3,
    ) {
        let m = facing.into().normalized();
        let u = m.orthogonal().normalized();
        let v = m.cross(u);
        let a = self.rng.uniform(0.0, TAU);
        let (u, v) = (u * cos(a) + v * sin(a), v * cos(a) - u * sin(a));
        let h = size / 2.0;
        let offsets = [-u * h - v * h, u * h - v * h, u * h + v * h, -u * h + v * h];
        let at = fit(at.into(), &offsets);
        self._quad(offsets.map(|o| at + o), m, material, &hull, shade, false);
    }

    /// An upright card `width` across and `height` tall at `at`, its plane facing `facing` (a hanging strand's:
    /// the cluster drawn long, as leaves that hang).
    #[allow(clippy::too_many_arguments)]
    pub fn hang(
        &mut self,
        at: impl Into<V3>,
        facing: impl Into<V3>,
        width: f64,
        height: f64,
        material: Mat,
        hull: impl Fn(V3) -> V3,
        shade: f64,
    ) {
        let m = facing.into().normalized();
        let v = (v3(0, 0, 1) - m * m.z).normalized();
        let u = v.cross(m);
        let (w, h) = (width / 2.0, height / 2.0);
        let at: V3 = at.into();
        self._quad(
            [-u * w - v * h, u * w - v * h, u * w + v * h, -u * w + v * h].map(|o| at + o),
            m,
            material,
            &hull,
            shade,
            true,
        );
    }

    /// A card of `corners` (round `m`, its plane's facing), moved into the box, its normals `hull`'s at each, a
    /// tile of the atlas on it: one of its species' clusters (of the ones for its green among blossom on a
    /// `green` colour's card, of the ones for `hanging` cards on one that hangs, where the species has them).
    pub fn _quad(
        &mut self,
        mut corners: [V3; 4],
        mut m: V3,
        material: Mat,
        hull: &dyn Fn(V3) -> V3,
        shade: f64,
        hanging: bool,
    ) {
        for i in 0..3 {
            let reach = corners.map(|c| c[i]);
            let (least, most) = (
                reach.iter().copied().fold(f32::INFINITY, f32::min) as f64,
                reach.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64,
            );
            let by = 0.0_f64.max(self.lo[i] - least) - 0.0_f64.max(most - self.hi[i]);
            for c in &mut corners {
                c[i] = (c[i] as f64 + by) as f32;
            }
        }
        let mut normals = corners.map(|c| hull(c).normalized());
        if math::sum(normals).dot(m) < 0.0 {
            // wound to face the way its normals do
            corners.reverse();
            normals.reverse();
            m = -m;
        }
        let normals = normals.map(|n| n.lerp(m, CARD_OWN).normalized());
        let green = material.name.starts_with("cardg");
        let tiles = if hanging && !self.tiles.strands.is_empty() {
            &self.tiles.strands
        } else if green && !self.tiles.green.is_empty() {
            &self.tiles.green
        } else {
            &self.tiles.cards
        };
        let (u0, v0) = tiles[self.rng.randrange(CARD_DRAWS) as usize % tiles.len()];
        let flip = self.rng.random() < 0.5;
        let (k, across) = (CARD_INSET, self.tiles.across);
        let uvs = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(x, y): (f64, f64)| {
            (
                u0 + across * (k + (1.0 - 2.0 * k) * if flip { 1.0 - x } else { x }),
                v0 + across * (k + (1.0 - 2.0 * k) * y),
            )
        });
        let quad = Quad {
            corners,
            normals,
            uvs,
            shade,
        };
        match self
            .by
            .iter_mut()
            .find(|(mat, _)| mat.name == material.name)
        {
            Some((_, quads)) => quads.push(quad),
            None => self.by.push((material, vec![quad])),
        }
    }

    /// `n` cards over the ellipsoid at `centre` (`half` its half sizes): the first level on `top` (the lump's highest
    /// point: the mass keeps its height, and a perch on it stays inside the tree), the rest spread evenly round it,
    /// but not where it faces the crown's middle (`crown`; over `inside` towards it) nor underneath (below `under`),
    /// each `push` of the way out to its surface (`CARD_TOP` more on top), `size` (a range) across. A card lies as a
    /// spray of leaves does: its plane faces `CARD_UP` of the way from straight out of the mass to straight up,
    /// then `tilt` degrees off that at random, so it is never edge-on to a camera that looks down on the tree (the
    /// game's), on the crown's near side or at its outline. Normals: the ellipsoid's, `blend` of the way to the
    /// crown's. `shade(the cards' random numbers, direction, point)` gives each card's baked shading.
    #[allow(clippy::too_many_arguments)]
    pub fn over(
        &mut self,
        centre: impl Into<V3>,
        half: impl Into<P3>,
        top: impl Into<V3>,
        n: usize,
        size: (f64, f64),
        material: Mat,
        crown: impl Into<V3>,
        shade: impl Fn(&mut Random, V3, V3) -> f64,
        o: Over,
    ) {
        let (c, crown): (V3, V3) = (centre.into(), crown.into());
        let (half, top): (P3, V3) = (half.into(), top.into());
        let ga = PI * (3.0 - sqrt(5.0));
        let off = self.rng.random() * TAU;
        let inward = crown - c;
        let inward = if inward.length() > 1e-4 {
            inward.normalized()
        } else {
            v3(0, 0, -1)
        };
        // the card drawn in towards the mass's middle until its corners are within `CARD_REACH` of its ellipsoid
        let fit = |mut at: V3, offsets: &[V3]| {
            for _ in 0..4 {
                // (each pass by its farthest corner)
                let (mut out, mut corner) = (f64::NEG_INFINITY, at);
                for &o in offsets {
                    let far = sqrt(py::sum((0..3).map(|i| {
                        pow((at[i] as f64 + o[i] as f64 - c[i] as f64) / half[i], 2.0)
                    })));
                    if far > out {
                        (out, corner) = (far, at + o);
                    }
                }
                if out <= CARD_REACH {
                    break;
                }
                at -= (corner - c) * (1.0 - CARD_REACH / out);
            }
            at
        };
        let hull = |p: V3| {
            let own = v3(
                (p.x as f64 - c.x as f64) / pow(half[0], 2.0),
                (p.y as f64 - c.y as f64) / pow(half[1], 2.0),
                (p.z as f64 - c.z as f64) / pow(half[2], 2.0),
            )
            .normalized();
            own.lerp((p - crown).normalized(), o.blend) + v3(0, 0, 0.12)
        };
        let across = self.rng.uniform(size.0, size.1);
        let grey = shade(&mut self.rng, v3(0, 0, 1), top);
        self.add(top, (0, 0, 1), across, material, hull, grey, |at, _| at);
        let (mut made, mut i) = (1, 0);
        while made < n && i < n * 4 {
            let t = 1.0 - 2.0 * py::fmod((i as f64 + 0.5) / (n as f64 * 1.6), 1.0);
            let ring = sqrt(0.0_f64.max(1.0 - t * t));
            let a = off + ga * i as f64 + self.rng.uniform(-0.3, 0.3);
            let d = v3(ring * cos(a), ring * sin(a), t);
            i += 1;
            if (d.dot(inward) > o.inside && (d.z as f64) < 0.5) || (d.z as f64) < o.under {
                continue;
            }
            let at = c + v3(
                d.x as f64 * half[0],
                d.y as f64 * half[1],
                d.z as f64 * half[2],
            ) * self.rng.uniform(o.push.0, o.push.1)
                * (1.0 + CARD_TOP * 0.0_f64.max(d.z as f64));
            let flat = d.lerp(v3(0, 0, 1), CARD_UP).normalized();
            let side = flat.orthogonal().normalized();
            let turn = self.rng.uniform(0.0, TAU);
            let side = side * cos(turn) + flat.cross(side) * sin(turn);
            let lean = radians(self.rng.uniform(o.tilt.0, o.tilt.1));
            let across = self.rng.uniform(size.0, size.1);
            let grey = shade(&mut self.rng, d, at);
            self.add(
                at,
                flat * cos(lean) + side * sin(lean),
                across,
                material,
                hull,
                grey,
                fit,
            );
            made += 1;
        }
    }

    /// Moves the card nearest each side of the box out to it (not down to its floor): the cards are the lumps' size
    /// across, deep and up to the millimetre, so the model is centred (`parts::settle`) where the lumps centred it.
    pub fn fill(&mut self) {
        let sides = [
            (0, self.lo[0], false),
            (0, self.hi[0], true),
            (1, self.lo[1], false),
            (1, self.hi[1], true),
            (2, self.hi[2], true),
        ];
        for (i, bound, most) in sides {
            // the first of the cards that reaches farthest that way
            let pick = |corners: &[V3; 4]| {
                let reach = corners.iter().map(|c| c[i]);
                if most {
                    reach.fold(f32::NEG_INFINITY, f32::max)
                } else {
                    reach.fold(f32::INFINITY, f32::min)
                }
            };
            let mut best: Option<(usize, usize, f32)> = None;
            for (g, (_, quads)) in self.by.iter().enumerate() {
                for (q, quad) in quads.iter().enumerate() {
                    let reach = pick(&quad.corners);
                    if best.is_none_or(|(_, _, b)| if most { reach > b } else { reach < b }) {
                        best = Some((g, q, reach));
                    }
                }
            }
            let Some((g, q, reach)) = best else {
                return;
            };
            let by = bound - reach as f64;
            for c in &mut self.by[g].1[q].corners {
                c[i] = (c[i] as f64 + by) as f32;
            }
        }
    }

    /// One mesh a material: four corners a card, their UVs and shading as given, their normals as Blender keeps the
    /// ones given (`normals::custom_from_verts`).
    pub fn build(&self) -> Vec<Obj> {
        let mut out = Vec::new();
        for (material, quads) in &self.by {
            let faces: Vec<Vec<u32>> = (0..quads.len() as u32)
                .map(|i| (4 * i..4 * i + 4).collect())
                .collect();
            let mut mesh =
                Mesh::from_pydata(quads.iter().flat_map(|q| q.corners).collect(), &faces);
            // (a quad's corners run in its points' order)
            mesh.uvs = Some(
                quads
                    .iter()
                    .flat_map(|q| q.uvs.map(|(u, v)| [u as f32, v as f32]))
                    .collect(),
            );
            mesh.card_shade = Some(
                quads
                    .iter()
                    .flat_map(|q| [0.05_f64.max(1.0_f64.min(q.shade)) as f32; 4])
                    .collect(),
            );
            for face in &mut mesh.faces {
                face.smooth = true;
            }
            let given: Vec<V3> = quads.iter().flat_map(|q| q.normals).collect();
            mesh.normals = Some(custom_from_verts(&mesh, &given));
            mesh.uv_metres = true; // its UVs are the atlas's: `uv::ensure` leaves them
            mesh.set_material(Some(*material));
            out.push(scene::add(
                &format!("{}_{}", self.name, material.name),
                mesh,
            ));
        }
        out
    }
}

/// Puts a core's UVs in the atlas's dense tile (`tile`, its corner, `size` wide and tall): a box projection about
/// its own middle, the tile `across` metres, turning back at the tile's edges (mirrored), so the leaves keep their
/// size on a big core and nothing wraps into the cut-out tiles.
pub fn core_uvs(obj: Obj, across: f64, tile: (f64, f64), size: f64) -> Obj {
    obj.edit(|part| {
        let mesh = &mut part.mesh;
        let co = &mesh.verts;
        let mid: Vec<f64> = (0..3)
            .map(|i| {
                let least = co.iter().map(|c| c[i]).fold(f32::INFINITY, f32::min);
                let most = co.iter().map(|c| c[i]).fold(f32::NEG_INFINITY, f32::max);
                (least as f64 + most as f64) / 2.0
            })
            .collect();
        let fold = |x: f64| {
            let t = py::fmod(x / across + 0.5, 2.0);
            0.13 + 0.74 * if t > 1.0 { 2.0 - t } else { t }
        };
        let mut uvs = mesh
            .uvs
            .take()
            .unwrap_or_else(|| vec![[0.0, 0.0]; mesh.loops.len()]);
        for f in 0..mesh.faces.len() {
            let normal = mesh.face_normal(f);
            let mut d = 0;
            for i in 1..3 {
                if normal[i].abs() > normal[d].abs() {
                    d = i;
                }
            }
            let (ua, va) = [(1, 2), (0, 2), (0, 1)][d];
            let face = mesh.faces[f];
            for l in face.start as usize..(face.start + face.len) as usize {
                let c = mesh.verts[mesh.loops[l] as usize];
                uvs[l] = [
                    (tile.0 + size * fold(c[ua] as f64 - mid[ua])) as f32,
                    (tile.1 + size * fold(c[va] as f64 - mid[va])) as f32,
                ];
            }
        }
        mesh.uvs = Some(uvs);
        mesh.uv_metres = true;
    });
    obj
}

/// What a tree draws as cards of its own once its masses have theirs (the willow's strands).
pub type More<'a> = Box<dyn FnOnce(&mut Cards) + 'a>;

/// `clad`'s options: `more(cards)` draws the stand-ins as a tree's own cards; `tiles` is where its pictures are
/// in the atlas.
#[derive(Default)]
pub struct Clad<'a> {
    pub more: Option<More<'a>>,
    pub tiles: Tiles,
}

/// Draws the build's leaf masses (`leaves`) as leaf cards, joined into `still` (returned): over each mass its cards
/// in its colour's (`CARD_OF`, palette `p`), 1 to 1.4 of its radius across, shaded open on top and on the crown's
/// outside, darker underneath and towards its middle; under each group of masses, and each mass of no group, a
/// dark core (the palette's `core`, `CORE` of their mean radius); the lumps go, and the stand-ins (`stand_in`),
/// which `more(cards)` draws as a tree's own cards (the willow's strands). The cards fill the box of what they
/// replace and no more (`Cards::fill`), so the tree keeps its size and its place. Its own random numbers (`seed`).
/// Nothing on a far version (its lumps stay).
pub fn clad(p: &Palette, still: Obj, seed: i128, o: Clad) -> Obj {
    let masses = MASSES.with(|m| m.borrow().clone());
    if masses.is_empty() {
        return still;
    }
    let stood_in = stood_in();
    let (lo, hi) = measure(&stood_in);
    let crown = (V3::from(lo) + V3::from(hi)) / 2;
    let mut cards = Cards::new("cards", seed, lo, hi);
    let (core, size) = (o.tiles.core, o.tiles.across);
    cards.tiles = o.tiles;
    let shade = |rng: &mut Random, d: V3, at: V3| {
        0.7 + 0.18 * d.dot((at - crown).normalized()) + 0.16 * d.z as f64 + rng.uniform(-0.1, 0.1)
    };
    // the masses under each core, by group, in the order each group came (a mass of no group: its own)
    let mut groups: Vec<(Group, Vec<&Mass>)> = Vec::new();
    for (i, m) in masses.iter().enumerate() {
        let r = py::sum(m.half.array().map(f64::from)) / 3.0;
        cards.over(
            m.centre,
            m.half,
            m.top,
            m.cards,
            (r, 1.4 * r),
            p.get(card_of(m.material)),
            crown,
            shade,
            Over::default(),
        );
        if m.core {
            let key = m.group.clone().unwrap_or(Group("own".to_string(), i));
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, of)) => of.push(m),
                None => groups.push((key, vec![m])),
            }
        }
    }
    if let Some(more) = o.more {
        more(&mut cards);
    }
    cards.fill();
    let mut made = cards.build();
    for (k, (_, of)) in groups.iter().enumerate() {
        let mid = math::sum(of.iter().map(|m| m.centre)) / of.len();
        let half = math::sum(of.iter().map(|m| m.half)) / of.len() * CORE;
        let (hx, hy, hz) = (half.x as f64, half.y as f64, half.z as f64);
        let r = (hx + hy) / 2.0;
        made.push(core_uvs(
            lump(
                format!("core{k}"),
                r,
                (mid.x as f64, mid.y as f64, mid.z as f64 - hz),
                p.get("core"),
                Lump {
                    scale: P3 {
                        x: hx / r,
                        y: hy / r,
                        z: hz / r,
                    },
                    noise: 0.3,
                    seed: of[0].seed,
                    segments: 6,
                    rings: 5,
                },
            ),
            1.5 * r,
            core,
            size,
        ));
    }
    for obj in stood_in {
        obj.remove();
    }
    clear();
    let mut all = vec![still];
    all.extend(made);
    join(&all, &still.name(), Some(P3::ZERO))
}
