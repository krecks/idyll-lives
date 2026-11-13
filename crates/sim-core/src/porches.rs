//! Build mode: porches over doors (`Command::BuildPorch`, `Command::RemovePorch`) and porch pieces
//! standing on their own (`Command::BuildPorchPiece`, `Command::RemovePorchPiece`).
//!
//! **A porch** stands outdoors in front of a door on the ground storey: a low landing with a step,
//! two posts and a small gabled canopy over it, and a lamp by the door (the renderer's
//! `HouseBuilder.porch`). It is the door's: it names the door's edge (`Porch::axis`, `x`, `z`) and
//! the side it stands on (`out`). Its landing is three tiles across (the door's and one either
//! side, partly) and two deep, its posts near the outer corners of the side tiles. It only looks:
//! the posts stand at the edges of those tiles and nobody walks into them, so walking round the
//! house stays as it was (the town's porches came before there was anything to build). The town
//! puts one over the front door of a house or a shop that the garden path meets (`LotFile::
//! porches`; a file or a save from before porches has them worked out again, `guess`), and the
//! player builds or takes one down on any outside door of their home for content `build.porch`,
//! sold back for `objectRules.resale` of it as a garage is. The door goes, so does its porch (a
//! wall or a window instead of it, the room round it moved: it goes with the room, `move_room`).
//!
//! **Porch pieces** are the same parts on their own, on whole tiles outdoors on the home lot, for
//! a porch of the player's own making: a post (one tile; it stands in the middle of it and nobody
//! walks through it, `cars::reblock` keeping its tile as it keeps a garage's), a gabled canopy
//! (`w` × `d` tiles, its ridge running the way it faces, a gable at either end: against a wall or
//! on posts), and a step platform (`w` × `d` tiles, a low step along its front: walked across).
//! Each faces one of four ways (`rot`, as a thing's: 0 towards +z, 1 +x, 2 -z, 3 -x). They take the
//! house's colours (its roof's for a canopy, its walls' for a canopy's gables). Pieces of a kind
//! don't overlap, but a canopy goes over posts and a platform. Taken down they sell back like a
//! garage. A room built round one, or a room moved onto one, isn't allowed through it: walls,
//! fences and diagonals don't go through a porch or a piece (`porch_across`).
//!
//! Both are kept for undo (`home.rs`), in blueprints (`blueprint.rs`) and in saves (`save.rs`).

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::command::EdgeAxis;
use crate::lot::{Edge, OUTDOORS};
use crate::world::World;

/// A porch over a door on the ground storey (see the module's doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Porch {
    pub plot: u32,
    /// The door's edge: `h` along x on grid line `z` (tile column `x`), `v` along z on grid line `x`
    /// (tile row `z`), on the ground storey's rows.
    pub axis: EdgeAxis,
    pub x: i32,
    pub z: i32,
    /// The side it stands on: 1 towards larger z (an `h` door) or x (a `v` door), -1 the other way.
    pub out: i8,
}

/// What a porch piece is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PieceKind {
    Post,
    Canopy,
    Step,
}

/// A porch piece on its own (see the module's doc): it covers tiles `x .. x + w` × `z .. z + d` (as
/// turned) and faces `rot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PorchPiece {
    pub plot: u32,
    pub kind: PieceKind,
    pub x: i32,
    pub z: i32,
    pub w: u8,
    pub d: u8,
    pub rot: u8,
}

/// Tiles across (along the way it faces' side) and deep a piece may be: `[across, deep]` at most.
pub const CANOPY_MOST: [u8; 2] = [3, 6];
pub const STEP_MOST: [u8; 2] = [4, 4];

/// The tile `k` across (-1, 0, 1: the door's and the side tiles) and `row` out (0, 1) of a porch.
fn porch_tile(p: &Porch, k: i32, row: i32) -> (i32, i32) {
    let out = i32::from(p.out.signum());
    match p.axis {
        EdgeAxis::V => {
            let col = if out > 0 { p.x + row } else { p.x - 1 - row };
            (col, p.z + k)
        }
        _ => {
            let r = if out > 0 { p.z + row } else { p.z - 1 - row };
            (p.x + k, r)
        }
    }
}

/// The grid edge between two tiles side by side (`h` or `v`, as an edge edit names it).
fn between(a: (i32, i32), b: (i32, i32)) -> (EdgeAxis, i32, i32) {
    if a.1 == b.1 {
        (EdgeAxis::V, a.0.max(b.0), a.1)
    } else {
        (EdgeAxis::H, a.0, a.1.max(b.1))
    }
}

/// The edges inside a block of tiles (between each tile and the next across and down).
fn inner_edges(tiles: &[(i32, i32)]) -> Vec<(EdgeAxis, i32, i32)> {
    let mut out = Vec::new();
    for &t in tiles {
        for n in [(t.0 + 1, t.1), (t.0, t.1 + 1)] {
            if tiles.contains(&n) {
                out.push(between(t, n));
            }
        }
    }
    out
}

impl Porch {
    /// The tiles its landing, step and canopy are over: three across, two out.
    pub fn tiles(&self) -> Vec<(i32, i32)> {
        let mut out = Vec::with_capacity(6);
        for row in 0..2 {
            for k in -1..=1 {
                out.push(porch_tile(self, k, row));
            }
        }
        out
    }

    /// The tiles its posts stand at the edges of (either side, the second row).
    pub fn post_tiles(&self) -> [(i32, i32); 2] {
        [porch_tile(self, -1, 1), porch_tile(self, 1, 1)]
    }

    /// The tile in front of the door, outdoors.
    pub fn front(&self) -> (i32, i32) {
        porch_tile(self, 0, 0)
    }

    /// The tile behind the door, indoors.
    pub fn back(&self) -> (i32, i32) {
        porch_tile(self, 0, -1)
    }

    /// The edges inside it: no wall, fence or gate may stand on them.
    pub fn inner_edges(&self) -> Vec<(EdgeAxis, i32, i32)> {
        inner_edges(&self.tiles())
    }
}

impl PorchPiece {
    /// The tiles it covers.
    pub fn tiles(&self) -> Vec<(i32, i32)> {
        let (x, z) = (self.x, self.z);
        (z..z + i32::from(self.d))
            .flat_map(move |tz| (x..x + i32::from(self.w)).map(move |tx| (tx, tz)))
            .collect()
    }

    /// Whether it covers tile `(x, z)`.
    pub fn covers(&self, x: i32, z: i32) -> bool {
        x >= self.x
            && z >= self.z
            && x < self.x + i32::from(self.w)
            && z < self.z + i32::from(self.d)
    }

    /// Tiles across (along its front) and deep.
    pub fn extent(&self) -> [u8; 2] {
        if self.rot.is_multiple_of(2) {
            [self.w, self.d]
        } else {
            [self.d, self.w]
        }
    }
}

impl PieceKind {
    fn name(self) -> &'static str {
        match self {
            PieceKind::Post => "porch post",
            PieceKind::Canopy => "porch canopy",
            PieceKind::Step => "step platform",
        }
    }
}

impl World {
    /// What a piece costs: a post, or a canopy or a platform by the tile.
    pub fn piece_price(&self, kind: PieceKind, tiles: i64) -> i64 {
        let b = &self.content.build;
        match kind {
            PieceKind::Post => b.porch_post,
            PieceKind::Canopy => b.porch_canopy * tiles,
            PieceKind::Step => b.porch_step * tiles,
        }
    }

    /// Whether a wall, a fence, a door or a window on edge (`axis`, `x`, `z`) would go through a
    /// porch or a porch piece (a canopy's or a platform's inside edges, a diagonal across any of
    /// their tiles or a post's): Build mode keeps walls out of them, as out of a garage.
    pub(crate) fn porch_across(&self, axis: EdgeAxis, x: i32, z: i32) -> Option<&'static str> {
        match axis {
            EdgeAxis::H | EdgeAxis::V => {
                let on = |edges: Vec<(EdgeAxis, i32, i32)>| edges.contains(&(axis, x, z));
                if self.porches.iter().any(|p| on(p.inner_edges())) {
                    return Some("porch");
                }
                self.porch_pieces
                    .iter()
                    .filter(|p| p.kind != PieceKind::Post)
                    .find(|p| on(inner_edges(&p.tiles())))
                    .map(|p| p.kind.name())
            }
            EdgeAxis::Dp | EdgeAxis::Dn => {
                if self.porches.iter().any(|p| p.tiles().contains(&(x, z))) {
                    return Some("porch");
                }
                self.porch_pieces
                    .iter()
                    .find(|p| p.covers(x, z))
                    .map(|p| p.kind.name())
            }
        }
    }

    /// Whether tile `(x, z)` is outdoors on the ground of plot `plot`, with no diagonal across it.
    fn open_ground(&self, plot: u32, x: i32, z: i32) -> bool {
        self.plots[plot as usize].contains(x, z)
            && self.storey_of(z) == 0
            && self.lot.in_bounds(x, z)
            && self.lot.room_at(x, z) == OUTDOORS
            && self.lot.diag(x, z).is_none()
    }

    /// Whether a wall, a fence or a gate stands on an edge.
    fn edge_built(&self, (axis, x, z): (EdgeAxis, i32, i32)) -> bool {
        if x < 0 || z < 0 {
            return false;
        }
        let e = match axis {
            EdgeAxis::H if z <= self.lot.depth as i32 && x < self.lot.width as i32 => {
                self.lot.h_edge(x as usize, z as usize)
            }
            EdgeAxis::V if x <= self.lot.width as i32 && z < self.lot.depth as i32 => {
                self.lot.v_edge(x as usize, z as usize)
            }
            _ => Edge::Open,
        };
        e != Edge::Open
    }

    /// The name (lower case) of the thing standing on tile `(x, z)`, if one does.
    fn thing_on(&self, x: i32, z: i32) -> Option<String> {
        self.objects
            .iter()
            .find(|o| o.tiles(&self.content).any(|t| t == (x, z)))
            .map(|o| self.content.objects[o.def].name.to_lowercase())
    }

    /// Whether the household's car parks on a tile (on its driveway, or where it parks by a garage).
    fn car_parks_on(&self, plot: u32, tiles: &[(i32, i32)]) -> bool {
        let on = |t: &(i32, i32)| tiles.contains(t);
        self.cars.iter().filter(|c| c.plot == plot).any(|c| {
            let mut spots = vec![(c.spot, c.yaw)];
            spots.extend(c.drive_spot.map(|[x, z, yaw]| ([x, z], yaw)));
            spots
                .into_iter()
                .any(|(spot, yaw)| crate::cars::car_tiles_at(c, spot, yaw).iter().any(on))
        }) || self
            .drives
            .iter()
            .filter(|d| d.plot == plot)
            .any(|d| crate::cars::drive_tiles(d).iter().any(on))
    }

    /// The porch over door (`axis`, `x`, `z`) of plot `plot`, if it can stand there (on an outside
    /// door on the ground storey, its tiles open ground, nothing in its way), or why not.
    fn porch_at(&self, plot: u32, axis: EdgeAxis, x: i32, z: i32) -> Result<Porch, Error> {
        if !matches!(axis, EdgeAxis::H | EdgeAxis::V) {
            return Err(Error::new("a porch goes on a door in a straight wall"));
        }
        let door = x >= 0
            && z >= 0
            && match axis {
                EdgeAxis::H => {
                    x < self.lot.width as i32
                        && z <= self.lot.depth as i32
                        && self.lot.h_edge(x as usize, z as usize) == Edge::Door
                }
                _ => {
                    x <= self.lot.width as i32
                        && z < self.lot.depth as i32
                        && self.lot.v_edge(x as usize, z as usize) == Edge::Door
                }
            };
        if !door {
            return Err(Error::new("a porch goes over a door"));
        }
        let row = if axis == EdgeAxis::H { z.max(1) - 1 } else { z };
        if self.storey_of(row) != 0 || (axis == EdgeAxis::H && self.storey_of(z) != 0) {
            return Err(Error::new("a porch goes on the ground floor"));
        }
        let outdoors = |out: i8| {
            let p = Porch {
                plot,
                axis,
                x,
                z,
                out,
            };
            let (tx, tz) = p.front();
            self.lot.in_bounds(tx, tz) && self.lot.room_at(tx, tz) == OUTDOORS
        };
        let out = match (outdoors(1), outdoors(-1)) {
            (true, false) => 1,
            (false, true) => -1,
            (true, true) => return Err(Error::new("a porch goes on a door into the house")),
            (false, false) => return Err(Error::new("that door is indoors")),
        };
        let porch = Porch {
            plot,
            axis,
            x,
            z,
            out,
        };
        if porch
            .tiles()
            .iter()
            .any(|&(tx, tz)| !self.open_ground(plot, tx, tz))
        {
            return Err(Error::new(
                "a porch needs open ground on your lot in front of the door",
            ));
        }
        if porch.inner_edges().into_iter().any(|e| self.edge_built(e)) {
            return Err(Error::new("there's a wall or a fence in the way"));
        }
        let tiles = porch.tiles();
        if self
            .garages
            .iter()
            .any(|g| g.tiles().iter().any(|t| tiles.contains(t)))
        {
            return Err(Error::new("the garage is in the way"));
        }
        for (tx, tz) in porch.post_tiles() {
            if let Some(name) = self.thing_on(tx, tz) {
                return Err(Error::new(format!("the {name} is in the way")));
            }
            if self.blocked[self.lot.tile_index(tx, tz)] {
                return Err(Error::new("there's no room for a porch there"));
            }
        }
        Ok(porch)
    }

    /// Builds a porch over a door of `household`'s home (`axis`, `x`, `z`: the door's edge), on its
    /// outdoor side, for content `build.porch`. Refused (saying why) on a door upstairs, between
    /// two rooms or out into the garden, where it would stand off the lot or indoors, over walls,
    /// fences, the garage or things, over a door that has one, or without the money.
    pub fn build_porch(
        &mut self,
        household: u32,
        axis: EdgeAxis,
        x: i32,
        z: i32,
    ) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        if !self.edge_on_plot(plot, axis, x, z) {
            return Err(Error::new("a porch goes on your own home"));
        }
        if self
            .porches
            .iter()
            .any(|p| (p.axis, p.x, p.z) == (axis, x, z))
        {
            return Err(Error::new("that door has a porch already"));
        }
        let porch = self.porch_at(plot, axis, x, z)?;
        let price = self.content.build.porch;
        self.can_pay(h, price, || "not enough money for a porch".into())?;
        self.porches.push(porch);
        self.pay(h, price);
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// Takes down the porch over a door of `household`'s home, for `objectRules.resale` of a
    /// porch's price (nothing in Creative).
    pub fn remove_porch(
        &mut self,
        household: u32,
        axis: EdgeAxis,
        x: i32,
        z: i32,
    ) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        let Some(i) = self
            .porches
            .iter()
            .position(|p| p.plot == plot && (p.axis, p.x, p.z) == (axis, x, z))
        else {
            return Err(Error::new("there's no porch over that door"));
        };
        self.porches.remove(i);
        self.refund(h, self.content.build.porch);
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// `objectRules.resale` of `price` back to household `h` (nothing in Creative).
    fn refund(&mut self, h: usize, price: i64) {
        let back = (price as f32 * self.content.object_rules.resale).round() as i64;
        self.households[h].funds += self.build_price(back);
    }

    /// Builds a porch piece on `household`'s home lot: tiles `x .. x + w` × `z .. z + d`, facing
    /// `rot`, for content `build.porchPost` (a post) or `build.porchCanopy` / `build.porchStep` a
    /// tile. Refused (saying why) off the lot or its ground, indoors, over walls or fences, on the
    /// garage, over a piece of its kind, a post or a platform where the car parks, a post on a
    /// thing or someone or where it would shut anyone or anything off, or without the money.
    #[allow(clippy::too_many_arguments)]
    pub fn build_porch_piece(
        &mut self,
        household: u32,
        kind: PieceKind,
        x: i32,
        z: i32,
        w: u8,
        d: u8,
        rot: u8,
    ) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        if rot > 3 {
            return Err(Error::new(format!(
                "a piece faces one of four ways, not {rot}"
            )));
        }
        let piece = PorchPiece {
            plot,
            kind,
            x,
            z,
            w,
            d,
            rot,
        };
        let [across, deep] = piece.extent();
        let most = match kind {
            PieceKind::Post => [1, 1],
            PieceKind::Canopy => CANOPY_MOST,
            PieceKind::Step => STEP_MOST,
        };
        if across == 0 || deep == 0 || across > most[0] || deep > most[1] {
            return Err(Error::new(if kind == PieceKind::Post {
                "a post takes one tile".to_string()
            } else {
                format!(
                    "a {} is 1 to {} tiles across and 1 to {} deep",
                    kind.name(),
                    most[0],
                    most[1]
                )
            }));
        }
        let tiles = piece.tiles();
        if tiles
            .iter()
            .any(|&(tx, tz)| !self.plots[plot as usize].contains(tx, tz) || self.storey_of(tz) != 0)
        {
            return Err(Error::new(format!(
                "a {} goes on your own lot",
                kind.name()
            )));
        }
        if tiles
            .iter()
            .any(|&(tx, tz)| !self.open_ground(plot, tx, tz))
        {
            return Err(Error::new(format!(
                "a {} goes outdoors, not in a room",
                kind.name()
            )));
        }
        if inner_edges(&tiles).into_iter().any(|e| self.edge_built(e)) {
            return Err(Error::new("there's a wall or a fence in the way"));
        }
        if self
            .garages
            .iter()
            .any(|g| g.tiles().iter().any(|t| tiles.contains(t)))
        {
            return Err(Error::new("the garage is in the way"));
        }
        if self
            .porch_pieces
            .iter()
            .any(|p| p.kind == kind && p.tiles().iter().any(|t| tiles.contains(t)))
        {
            return Err(Error::new(format!(
                "there's a {} there already",
                kind.name()
            )));
        }
        if kind != PieceKind::Canopy && self.car_parks_on(plot, &tiles) {
            return Err(Error::new("that's where the car parks"));
        }
        if kind == PieceKind::Post {
            let (tx, tz) = (x, z);
            if let Some(name) = self.thing_on(tx, tz) {
                return Err(Error::new(format!("the {name} is in the way")));
            }
            if self
                .sims
                .iter()
                .any(|s| s.away_until.is_none() && s.tile() == (tx, tz))
            {
                return Err(Error::new("someone is standing there"));
            }
            if self.blocked[self.lot.tile_index(tx, tz)] {
                return Err(Error::new("there's no room for a post there"));
            }
        }
        let price = self.piece_price(kind, tiles.len() as i64);
        self.can_pay(h, price, || {
            format!("not enough money for a {}", kind.name())
        })?;
        if kind == PieceKind::Post {
            let before = self.reach_signature(plot);
            self.porch_pieces.push(piece);
            crate::cars::posts_changed(self);
            if !self.keeps_reach(plot, &before, None) {
                self.porch_pieces.pop();
                crate::cars::posts_changed(self);
                return Err(Error::new("that would block the way"));
            }
        } else {
            self.porch_pieces.push(piece);
        }
        self.pay(h, price);
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// Takes down the porch piece of `kind` on tile `(x, z)` of `household`'s home lot, for
    /// `objectRules.resale` of its price (nothing in Creative).
    pub fn remove_porch_piece(
        &mut self,
        household: u32,
        kind: PieceKind,
        x: i32,
        z: i32,
    ) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        let Some(i) = self
            .porch_pieces
            .iter()
            .position(|p| p.plot == plot && p.kind == kind && p.covers(x, z))
        else {
            return Err(Error::new(format!("there's no {} there", kind.name())));
        };
        let piece = self.porch_pieces.remove(i);
        if kind == PieceKind::Post {
            crate::cars::posts_changed(self);
        }
        let price = self.piece_price(kind, piece.tiles().len() as i64);
        self.refund(h, price);
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// After walls changed on `plot` (built, taken down, a room moved): a porch whose door is gone
    /// (a wall or a window now, or no wall) or no longer leads outdoors, or (moved with its room)
    /// whose ground isn't open now, goes, and so does a piece a room was built round; each sells
    /// back as if taken down.
    pub(crate) fn porches_follow(&mut self, plot: u32) {
        let Some(h) = self.households.iter().position(|h| h.plot == Some(plot)) else {
            return;
        };
        let mut gone = 0;
        let mut i = 0;
        while i < self.porches.len() {
            let p = self.porches[i];
            let stands = p.plot != plot
                || self
                    .porch_at(plot, p.axis, p.x, p.z)
                    .is_ok_and(|q| q.out == p.out)
                || self.porch_kept(&p);
            if stands {
                i += 1;
            } else {
                self.porches.remove(i);
                gone += self.content.build.porch;
            }
        }
        let mut posts = false;
        let mut i = 0;
        while i < self.porch_pieces.len() {
            let p = self.porch_pieces[i];
            let stands = p.plot != plot
                || p.tiles()
                    .iter()
                    .all(|&(tx, tz)| self.open_ground(plot, tx, tz));
            if stands {
                i += 1;
            } else {
                self.porch_pieces.remove(i);
                posts |= p.kind == PieceKind::Post;
                gone += self.piece_price(p.kind, p.tiles().len() as i64);
            }
        }
        if posts {
            crate::cars::posts_changed(self);
        }
        if gone > 0 || posts {
            self.refund(h, gone);
            self.structure_version = self.structure_version.wrapping_add(1);
        }
    }

    /// Porches `moved` (indices) went with their room (`move_room`): one without open ground in
    /// front of its door where it went goes, sold back as if taken down.
    pub(crate) fn porches_moved(&mut self, plot: u32, moved: &[usize]) {
        let Some(h) = self.households.iter().position(|h| h.plot == Some(plot)) else {
            return;
        };
        let gone: Vec<usize> = moved
            .iter()
            .copied()
            .filter(|&i| {
                let p = self.porches[i];
                self.porch_at(plot, p.axis, p.x, p.z)
                    .map_or(true, |q| q.out != p.out)
            })
            .collect();
        for &i in gone.iter().rev() {
            self.porches.remove(i);
        }
        if !gone.is_empty() {
            self.refund(h, self.content.build.porch * gone.len() as i64);
        }
        if !moved.is_empty() {
            self.structure_version = self.structure_version.wrapping_add(1);
        }
    }

    /// A porch that stands as the town built it, though it couldn't be built there now (a thing at
    /// its post, its landing up to a fence): its door is still a door out of the house.
    fn porch_kept(&self, p: &Porch) -> bool {
        let (fx, fz) = p.front();
        let door = match p.axis {
            EdgeAxis::H => {
                p.z >= 0
                    && p.x >= 0
                    && p.x < self.lot.width as i32
                    && p.z <= self.lot.depth as i32
                    && self.lot.h_edge(p.x as usize, p.z as usize) == Edge::Door
            }
            _ => {
                p.z >= 0
                    && p.x >= 0
                    && p.x <= self.lot.width as i32
                    && p.z < self.lot.depth as i32
                    && self.lot.v_edge(p.x as usize, p.z as usize) == Edge::Door
            }
        };
        door && self.lot.in_bounds(fx, fz) && self.lot.room_at(fx, fz) == OUTDOORS
    }

    /// The porches a town file or a save from before porches had drawn over its doors (the
    /// renderer's old `porchDoors`): over each door on the ground storey in the outline of a
    /// house's walls (`view::house_bounds`) that the plot's garden path (its first path narrower
    /// than a driveway, `meta.paths`) comes up to, its middle within 0.6 m of the path's, on the
    /// path's side.
    pub fn guess_porches(&self) -> Vec<Porch> {
        /// Paths narrower than this are garden paths (wider ones driveways: the renderer's `DRIVEWAY_MIN_WIDTH`).
        const GARDEN_PATH: f32 = 1.8;
        let mut out = Vec::new();
        for (i, p) in self.plots.iter().enumerate() {
            let Some([bx0, bz0, bx1, bz1]) =
                crate::view::house_bounds(self, p.x, p.z, p.x + p.w, p.z + p.d)
            else {
                continue;
            };
            let inside = |r: &[f32; 4]| {
                r[0] >= p.x as f32 - 0.01
                    && r[1] >= p.z as f32 - 0.01
                    && r[2] <= (p.x + p.w) as f32 + 0.01
                    && r[3] <= (p.z + p.d) as f32 + 0.01
            };
            let Some(&r) = self
                .paths
                .iter()
                .find(|r| inside(r) && (r[2] - r[0]).min(r[3] - r[1]) < GARDEN_PATH)
            else {
                continue;
            };
            let door = |axis: EdgeAxis, x: i32, z: i32| {
                x >= 0
                    && z >= 0
                    && match axis {
                        EdgeAxis::H => {
                            x < self.lot.width as i32
                                && z <= self.lot.depth as i32
                                && self.lot.h_edge(x as usize, z as usize) == Edge::Door
                        }
                        _ => {
                            x <= self.lot.width as i32
                                && z < self.lot.depth as i32
                                && self.lot.v_edge(x as usize, z as usize) == Edge::Door
                        }
                    }
            };
            // Along z, to a door in a wall along x (and the other way round).
            for (axis, across, mid, lo, hi, a0, a1, b0, b1) in [
                (
                    EdgeAxis::H,
                    r[2] - r[0],
                    (r[0] + r[2]) / 2.0,
                    r[1],
                    r[3],
                    bx0,
                    bx1,
                    bz0,
                    bz1,
                ),
                (
                    EdgeAxis::V,
                    r[3] - r[1],
                    (r[1] + r[3]) / 2.0,
                    r[0],
                    r[2],
                    bz0,
                    bz1,
                    bx0,
                    bx1,
                ),
            ] {
                if across >= GARDEN_PATH {
                    continue;
                }
                for (line, side) in [(b0, -1i8), (b1, 1)] {
                    let before = if side < 0 {
                        hi <= line as f32 + 0.01
                    } else {
                        lo >= line as f32 - 0.01
                    };
                    if !before {
                        continue;
                    }
                    for a in a0..a1 {
                        if (a as f32 + 0.5 - mid).abs() >= 0.6 {
                            continue;
                        }
                        let (x, z) = if axis == EdgeAxis::H {
                            (a, line)
                        } else {
                            (line, a)
                        };
                        if door(axis, x, z) {
                            out.push(Porch {
                                plot: i as u32,
                                axis,
                                x,
                                z,
                                out: side,
                            });
                        }
                    }
                }
            }
        }
        out
    }
}
