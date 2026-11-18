//! A species of tree as data, and the build that grows it. A tree grows in stages, in the order its random
//! numbers are drawn: trunk, roots, crown, hearts, extras; then what stands so far is joined, its perches are found
//! on it, and it is clad in its leaf cards with its fruit hung. A stage is either a spec, numbers the shared skeleton
//! (`skeleton`, `conifer`) grows it from, or any function of the tree in the making, so a species shares the others'
//! way where it is theirs and has its own where it isn't: nothing of one species' shape is another's but what both
//! name.

use crate::botany::conifer::{TierAims, tier_aims};
use crate::botany::skeleton::{
    _branches, _hearts, _roots, _trunk, Branches, Buttress, Clump, ClumpAims, Heart, Limb,
    LimbPoints, Stem, clump_aims, limb_points, palette, shades,
};
use crate::botany::{Clad, More, Tiles, clad, join_still};
use crate::materials::{Mat, Palette};
use crate::parts::{join_all, settle};
use crate::perch::{self, Find};
use crate::rand::Random;
use crate::scene::Obj;
use std::any::Any;

/// One stage of a tree's growth: none, a spec the shared skeleton grows it from, or the species' own way.
#[derive(Clone, Debug, Default)]
pub enum Stage<S> {
    #[default]
    None,
    Spec(S),
    Own(fn(&mut Tree)),
}

/// A species' random numbers: its build's, its perches' and its leaf cards' (none: it wears no cards).
#[derive(Clone, Copy, Debug, Default)]
pub struct Seeds {
    pub build: i128,
    pub perches: i128,
    pub cards: Option<i128>,
}

/// A trunk as `_trunk` sweeps it: through `pts`, radius `r` times `scale` (one per point), its bark wobbled by `seed`.
#[derive(Clone, Debug)]
pub struct Trunk {
    pub pts: Vec<(f64, f64, f64)>,
    pub r: f64,
    pub scale: Vec<f64>,
    pub seed: f64,
    pub o: Stem,
}

/// Root buttresses as `_roots` flares them: `n`, from a trunk `r` round at its foot, from `h` up it out to `reach`.
#[derive(Clone, Debug)]
pub struct Roots {
    pub n: usize,
    pub r: f64,
    pub h: f64,
    pub reach: f64,
    pub o: Buttress,
}

/// A crown as `_branches` grows it: `tips` leaf masses over the ellipsoid (`centre`, `radii`) on `limbs` limbs from
/// the trunk at heights `base_z`; `material` names the palette colour its limbs and boughs wear, if not the
/// species' bark.
#[derive(Clone, Debug)]
pub struct Crown {
    pub centre: (f64, f64, f64),
    pub radii: (f64, f64, f64),
    pub tips: usize,
    pub limbs: usize,
    pub base_z: (f64, f64),
    pub material: Option<&'static str>,
    pub o: Branches,
}

/// A crown's dark heart as `_hearts` closes it: `n` lumps of `size` round its middle.
#[derive(Clone, Debug)]
pub struct Hearts {
    pub n: usize,
    pub centre: (f64, f64, f64),
    pub radii: (f64, f64, f64),
    pub size: f64,
    pub o: Heart,
}

/// The lines a tree's perches are found along.
#[derive(Clone, Debug)]
pub enum Aims {
    /// In towards each clump of its crown (`clump_aims`), and towards `more` that it lists itself.
    Clumps { o: ClumpAims, more: Vec<Clump> },
    /// In towards each tier of a conifer (`tier_aims`).
    Tiers(TierAims),
}

/// A tree's perches as `perch::find` keeps them: up to `n` where `aims` meet its leaves, and with `limbs`, on its
/// bare limbs at those places along them (`limb_points`); `o`: `find`'s options.
#[derive(Clone, Debug)]
pub struct Perches {
    pub n: usize,
    pub aims: Aims,
    pub limbs: Option<LimbPoints>,
    pub o: Find,
}

/// A species: what it is called, its random numbers, its colours, and how each stage of it grows.
#[derive(Clone, Debug)]
pub struct Species {
    /// Its still part's name.
    pub name: &'static str,
    pub seeds: Seeds,
    /// Its leaf shades, darkest first (five, or four: the last stays the family's).
    pub leaves: &'static [&'static str],
    /// The other colours of the family's palette it changes: (name, #rrggbb).
    pub colours: &'static [(&'static str, &'static str)],
    /// Where its leaf cards' pictures are in the atlas.
    pub tiles: Tiles,
    /// The palette colour its trunk, roots and limbs wear.
    pub bark: &'static str,
    pub trunk: Stage<Trunk>,
    pub roots: Stage<Roots>,
    pub crown: Stage<Crown>,
    pub hearts: Stage<Hearts>,
    /// What else stands on it before its perches are found (green among blossom, laden boughs, blooms).
    pub extras: Option<fn(&mut Tree)>,
    pub perches: Stage<Perches>,
    /// What is done to the still part once its perches are found (fruit hung, a part that moves): the model's
    /// parts. None: it is clad in its leaf cards and settled.
    pub fruit: Option<fn(&mut Tree, Obj) -> Vec<Obj>>,
}

impl Default for Species {
    fn default() -> Species {
        Species {
            name: "",
            seeds: Seeds::default(),
            leaves: &[],
            colours: &[],
            tiles: Tiles::default(),
            bark: "bark",
            trunk: Stage::None,
            roots: Stage::None,
            crown: Stage::None,
            hearts: Stage::None,
            extras: None,
            perches: Stage::None,
            fruit: None,
        }
    }
}

/// A tree in the making: what its stages are given and what they leave for the ones after.
pub struct Tree<'a> {
    pub species: &'a Species,
    /// The family's palette in the species' colours.
    pub palette: Palette,
    /// Its leaf shades, darkest first.
    pub cols: Vec<Mat>,
    /// The build's random numbers.
    pub rng: Random,
    /// The clumps of its crown so far (`_mass`), for its perches and what sits on its leaves.
    pub clumps: Vec<Clump>,
    /// Its limbs so far, for the perches on them.
    pub limbs: Vec<Limb>,
    /// A conifer's tiers' feet, for its perches.
    pub feet: Vec<(f64, f64, f64)>,
    /// What it draws as leaf cards of its own once its masses have theirs (`clad`'s `more`).
    pub more: Option<More<'static>>,
    /// What one of the species' own stages keeps for a later one.
    pub kept: Option<Box<dyn Any>>,
}

impl Tree<'_> {
    /// The species' bark.
    pub fn bark(&self) -> Mat {
        self.palette.get(self.species.bark)
    }

    /// What an earlier stage of the species' own kept (`kept`).
    pub fn kept<T: 'static>(&self) -> &T {
        self.kept
            .as_ref()
            .and_then(|k| k.downcast_ref())
            .unwrap_or_else(|| panic!("{}: nothing of that kind kept", self.species.name))
    }

    /// The still part clad in the tree's leaf cards (the species' seed and tiles, the tree's own cards).
    pub fn clad(&mut self, still: Obj) -> Obj {
        match self.species.seeds.cards {
            Some(seed) => clad(
                &self.palette,
                still,
                seed,
                Clad {
                    more: self.more.take(),
                    tiles: self.species.tiles.clone(),
                },
            ),
            None => still,
        }
    }
}

/// Builds a tree of `species`: the model's parts, settled.
pub fn grow(species: &Species) -> Vec<Obj> {
    let p = palette(species.leaves, species.colours);
    let cols = shades(&p, species.leaves.len());
    let mut tree = Tree {
        species,
        palette: p,
        cols,
        rng: Random::new(species.seeds.build),
        clumps: Vec::new(),
        limbs: Vec::new(),
        feet: Vec::new(),
        more: None,
        kept: None,
    };
    match &species.trunk {
        Stage::None => {}
        Stage::Spec(t) => {
            let o = Stem {
                material: t.o.material.or(Some(tree.bark())),
                ..t.o.clone()
            };
            _trunk(&tree.palette, &t.pts, t.r, &t.scale, t.seed, o);
        }
        Stage::Own(trunk) => trunk(&mut tree),
    }
    match &species.roots {
        Stage::None => {}
        Stage::Spec(r) => {
            let o = Buttress {
                material: r.o.material.or(Some(tree.bark())),
                ..r.o.clone()
            };
            _roots(&tree.palette, &mut tree.rng, r.n, r.r, r.h, r.reach, o);
        }
        Stage::Own(roots) => roots(&mut tree),
    }
    match &species.crown {
        Stage::None => {}
        Stage::Spec(c) => {
            let material = match c.material {
                Some(name) => tree.palette.get(name),
                None => tree.bark(),
            };
            let o = Branches {
                material: c.o.material.or(Some(material)),
                ..c.o.clone()
            };
            let (clumps, limbs) = _branches(
                &tree.palette,
                &mut tree.rng,
                &tree.cols,
                c.centre,
                c.radii,
                c.tips,
                c.limbs,
                c.base_z,
                o,
            );
            tree.clumps.extend(clumps);
            tree.limbs.extend(limbs);
        }
        Stage::Own(crown) => crown(&mut tree),
    }
    match &species.hearts {
        Stage::None => {}
        Stage::Spec(h) => _hearts(
            &mut tree.rng,
            &tree.cols,
            h.n,
            h.centre,
            h.radii,
            h.size,
            h.o.clone(),
        ),
        Stage::Own(hearts) => hearts(&mut tree),
    }
    if let Some(extras) = species.extras {
        extras(&mut tree);
    }
    // what stands so far, but for the leaf masses its cards are to be drawn over
    let still = match species.seeds.cards {
        Some(_) => join_still(species.name),
        None => join_all(species.name),
    };
    match &species.perches {
        Stage::None => {}
        Stage::Spec(perches) => {
            let mut rng = Random::new(species.seeds.perches);
            let aims = match &perches.aims {
                Aims::Clumps { o, more } => {
                    let mut clumps = tree.clumps.clone();
                    clumps.extend_from_slice(more);
                    clump_aims(&clumps, &mut rng, o.clone())
                }
                Aims::Tiers(o) => tier_aims(&mut rng, &tree.feet, o.clone()),
            };
            let mut o = perches.o.clone();
            if let Some(along) = &perches.limbs {
                o.limbs = limb_points(&tree.limbs, along.clone());
            }
            perch::find(perches.n, &aims, o);
        }
        Stage::Own(perches) => perches(&mut tree),
    }
    match species.fruit {
        Some(fruit) => fruit(&mut tree, still),
        None => {
            let still = tree.clad(still);
            settle(&[still])
        }
    }
}
