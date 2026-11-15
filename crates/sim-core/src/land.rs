//! Build mode's Land tab: the land shaped by the player (`Command::ShapeLand`).
//!
//! The land itself, its heights, its water levels and what covers it, is the page's (`game/land.ts`):
//! the simulation is flat, and carries the land in the town's `meta` (`meta.land`, coded in tiles
//! of 64 × 64 samples) without reading it, as it carries the rest of `meta`. A stroke of the land
//! brushes sends the tiles it touched, merged into `meta.land` by index (and, the first time, the
//! land's header with them); the zones the page works out from the land's cover and water, all of
//! them, replacing `meta.zones`; and, when the stroke changed the water, the world's whole water
//! mask (1 m tiles, run-length, as the town file's `terrain.water`), which is the one thing here
//! the simulation reads: nobody walks on water (`Travel::base_blocked`).
//!
//! New water may not go on a lot, on a road, on anyone, or where something stands; such a stroke
//! is refused whole, with why. Water that dries leaves its tiles to walk on (but for a lit
//! junction's, kept off as ever). The page prices a stroke (content `build.land`) and the
//! simulation charges it as other build edits are charged: free in Creative, refused in Living
//! past the household's funds, refunded when negative (the stroke undone: the land keeps its own
//! undo, which sends the reverse stroke). It isn't a home edit, so it leaves the home edits kept
//! for undo as they are, but for what it changed under them (`World::keep_land_in_edits`).
//!
//! Both `meta.land` and the water mask are saved as they are (`save.rs`, still 39).

use serde_json::Value;

use crate::Error;
use crate::travel::Terrain;
use crate::world::World;

/// The most tiles `meta.land` may have: a world at the widest and deepest (`lot::MAX_SIDE`) is
/// 512 × 512 samples, 8 × 8 tiles; this leaves room for a finer cell or smaller tiles.
const MOST_LAND_TILES: usize = 4096;

/// What a stroke does to the water (`World::water_change`), checked and ready to set.
struct WaterChange {
    /// The new mask, as sent.
    rle: Vec<u32>,
    /// The tiles (`Lot::tile_index`) wet now and dry before, and dry now and wet before (none
    /// of them under the town file's other ground nobody walks on, `terrain.blocked`).
    wet: Vec<usize>,
    dried: Vec<usize>,
    /// The tiles changed, `[x0, z0, x1, z1)` (empty if none).
    rect: [i32; 4],
}

impl World {
    /// Shapes the land for `household` (see the module's doc): checks it all, then charges `cost`
    /// and changes `meta.land`, `meta.zones` and the water as `meta` and `water` say.
    pub fn shape_land(
        &mut self,
        household: u32,
        cost: i64,
        meta: &Value,
        water: Option<Vec<u32>>,
    ) -> Result<(), Error> {
        let h = household as usize;
        if h >= self.households.len() {
            return Err(Error::new(format!("unknown household {household}")));
        }
        let land = match meta.get("land") {
            Some(patch) if !patch.is_null() => Some(merged_land(self.meta.get("land"), patch)?),
            _ => None,
        };
        let zones = meta.get("zones").filter(|z| !z.is_null()).cloned();
        if (land.is_some() || zones.is_some()) && !(self.meta.is_object() || self.meta.is_null()) {
            return Err(Error::new("the town's meta isn't an object"));
        }
        let water = water.map(|rle| self.water_change(rle)).transpose()?;
        self.can_pay(h, cost, || {
            format!("not enough money: shaping the land costs ${cost}")
        })?;

        self.pay(h, cost);
        let paid = self.build_price(cost);
        if land.is_some() || zones.is_some() {
            if self.meta.is_null() {
                self.meta = Value::Object(Default::default());
            }
            let m = self.meta.as_object_mut().expect("checked above");
            if let Some(land) = land {
                m.insert("land".into(), land);
            }
            if let Some(zones) = zones {
                m.insert("zones".into(), zones);
            }
        }
        let (wet, dried) = match water {
            Some(change) => self.set_water(change),
            None => (Vec::new(), Vec::new()),
        };
        self.keep_land_in_edits(h, paid, &wet, &dried);
        Ok(())
    }

    /// The ground grid the town file's terrain masks cover: the lot's width by the ground
    /// storey's rows.
    pub(crate) fn ground_grid(&self) -> (usize, usize) {
        (self.lot.width, self.storey_depth().max(0) as usize)
    }

    /// The tiles (`Lot::tile_index`, sorted) `terrain` keeps everyone off: its water and its other
    /// ground nobody walks on (`Travel::base_blocked`).
    pub(crate) fn terrain_blocked(&self, terrain: &Terrain) -> Vec<usize> {
        let (w, d) = self.ground_grid();
        let mask = terrain.mask(w, d);
        let mut tiles: Vec<usize> = (0..mask.len())
            .filter(|&i| mask[i])
            .map(|i| self.lot.tile_index((i % w) as i32, (i / w) as i32))
            .collect();
        tiles.sort_unstable();
        tiles.dedup();
        tiles
    }

    /// Checks the new water mask `rle` against the one there is: it covers the ground grid, and
    /// none of its new water is on a lot, on a road, on anyone or where something stands.
    fn water_change(&self, rle: Vec<u32>) -> Result<WaterChange, Error> {
        let (w, d) = self.ground_grid();
        let size = w * d;
        let covered: u64 = rle.chunks(2).map(|p| u64::from(p[0])).sum();
        if covered != size as u64 {
            return Err(Error::new(format!(
                "the water mask covers {covered} tiles; the ground has {size} ({w} × {d})"
            )));
        }
        let terrain = self.travel.file.terrain.as_ref();
        let new = rle_mask(Some(&rle), size);
        let old = rle_mask(terrain.and_then(|t| t.water.as_deref()), size);
        let rock = rle_mask(terrain.and_then(|t| t.blocked.as_deref()), size);
        let (mut fresh, mut gone) = (Vec::new(), Vec::new());
        let mut rect = [i32::MAX, i32::MAX, i32::MIN, i32::MIN];
        for k in (0..size).filter(|&k| new[k] != old[k] && !rock[k]) {
            let (x, z) = ((k % w) as i32, (k / w) as i32);
            rect = [
                rect[0].min(x),
                rect[1].min(z),
                rect[2].max(x + 1),
                rect[3].max(z + 1),
            ];
            if new[k] { &mut fresh } else { &mut gone }.push((x, z));
        }
        if fresh.is_empty() && gone.is_empty() {
            rect = [0, 0, 0, 0];
        }
        if !fresh.is_empty() {
            let wet: Vec<bool> = (0..size).map(|k| new[k] && !old[k] && !rock[k]).collect();
            self.water_fits(&fresh, &wet, w)?;
        }
        let slots = |tiles: &[(i32, i32)]| -> Vec<usize> {
            tiles
                .iter()
                .map(|&(x, z)| self.lot.tile_index(x, z))
                .filter(|&i| i != crate::lot::VOID)
                .collect()
        };
        Ok(WaterChange {
            wet: slots(&fresh),
            dried: slots(&gone),
            rle,
            rect,
        })
    }

    /// Refuses new water (`fresh`, tiles of the ground grid `w` wide, and as a mask, `wet`) on a lot,
    /// on a road (within its width with the sidewalks), on anyone, or where something stands.
    fn water_fits(&self, fresh: &[(i32, i32)], wet: &[bool], w: usize) -> Result<(), Error> {
        let (mut x0, mut z0, mut x1, mut z1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for &(x, z) in fresh {
            (x0, z0, x1, z1) = (x0.min(x), z0.min(z), x1.max(x + 1), z1.max(z + 1));
        }
        let is_fresh = |x: i32, z: i32| {
            x >= 0
                && z >= 0
                && (x as usize) < w
                && wet
                    .get(z as usize * w + x as usize)
                    .copied()
                    .unwrap_or(false)
        };
        // On a lot.
        for p in &self.plots {
            let (px0, pz0) = (p.x.max(x0), p.z.max(z0));
            let (px1, pz1) = ((p.x + p.w).min(x1), (p.z + p.d).min(z1));
            if (pz0..pz1).any(|z| (px0..px1).any(|x| is_fresh(x, z))) {
                return Err(Error::new(format!("water can't go on a lot ({})", p.name)));
            }
        }
        // On a road: a tile whose middle is within the road's half width (and half a tile).
        for road in &self.travel.graph.roads {
            let reach = road.width / 2.0 + 0.5;
            for q in road.pts.windows(2) {
                let (a, b) = (q[0], q[1]);
                let sx0 = ((a[0].min(b[0]) - reach).floor() as i32).max(x0);
                let sz0 = ((a[1].min(b[1]) - reach).floor() as i32).max(z0);
                let sx1 = ((a[0].max(b[0]) + reach).ceil() as i32).min(x1);
                let sz1 = ((a[1].max(b[1]) + reach).ceil() as i32).min(z1);
                for z in sz0..sz1 {
                    for x in sx0..sx1 {
                        let middle = [x as f32 + 0.5, z as f32 + 0.5];
                        if is_fresh(x, z) && crate::cars::to_segment(middle, a, b) <= reach {
                            return Err(Error::new("water can't go across a road"));
                        }
                    }
                }
            }
        }
        // On anyone.
        if let Some(s) = self.sims.iter().find(|s| {
            s.here()
                && s.pos.iter().all(|v| v.is_finite())
                && is_fresh(s.pos[0].floor() as i32, s.pos[1].floor() as i32)
        }) {
            return Err(Error::new(format!(
                "water can't go where {} is standing",
                s.name
            )));
        }
        // Where something stands (a thing, a parked car, a lit junction's tiles): blocked, and not
        // by the land itself.
        let blocked = self.blocked();
        for &(x, z) in fresh {
            let i = self.lot.tile_index(x, z);
            if i != crate::lot::VOID
                && blocked.get(i).copied().unwrap_or(false)
                && self.travel.base_blocked.binary_search(&i).is_err()
            {
                return Err(Error::new(format!(
                    "something stands in the way of the water at {x}, {z}"
                )));
            }
        }
        Ok(())
    }

    /// Sets the water `change` checked: the town file's water mask, who may walk where, and the
    /// ways through the tiles it changed found again (as a wall built there would have them).
    /// Returns the tiles it made wet and dried.
    fn set_water(&mut self, change: WaterChange) -> (Vec<usize>, Vec<usize>) {
        let WaterChange {
            rle,
            wet,
            dried,
            rect,
        } = change;
        self.travel
            .file
            .terrain
            .get_or_insert_with(Default::default)
            .water = Some(rle);
        let terrain = self.travel.file.terrain.as_ref().expect("set above");
        self.travel.base_blocked = self.terrain_blocked(terrain);
        for &i in &wet {
            if let Some(b) = self.blocked.get_mut(i) {
                *b = true;
            }
        }
        for &i in &dried {
            let kept = self.travel.junction_tiles.binary_search(&i).is_ok()
                || self.car_tiles.binary_search(&i).is_ok();
            if let Some(b) = self.blocked.get_mut(i) {
                *b = kept;
            }
        }
        if rect[2] > rect[0] {
            self.objects_changed_in([rect[0] - 1, rect[1] - 1, rect[2] + 1, rect[3] + 1]);
        }
        (wet, dried)
    }
}

/// A run-length mask (`[run, value, ...]`, `Terrain`) over `size` tiles; none: all clear.
fn rle_mask(rle: Option<&[u32]>, size: usize) -> Vec<bool> {
    let mut out = vec![false; size];
    let mut at = 0usize;
    for pair in rle.unwrap_or_default().chunks(2) {
        let end = at.saturating_add(pair[0] as usize).min(size);
        if pair.get(1).is_some_and(|&v| v != 0) {
            out[at..end].fill(true);
        }
        at = end;
    }
    out
}

/// `meta.land` (`old`, if there is one) with a stroke's `patch` merged in: its `h` and `wl`, tiles
/// by index (a JSON object's keys), over the old ones; any other field it has over the old one
/// (`cover` whole). With no land yet, the patch must say the land's size (`cols`, `rows`): the
/// tiles it doesn't send are left empty (flat at 0 and dry, as `decodeLand` reads them).
fn merged_land(old: Option<&Value>, patch: &Value) -> Result<Value, Error> {
    let Some(patch) = patch.as_object() else {
        return Err(Error::new("meta.land must be an object"));
    };
    let mut land = match old {
        Some(Value::Object(o)) => o.clone(),
        _ => {
            if !(patch.contains_key("cols") && patch.contains_key("rows")) {
                return Err(Error::new(
                    "the land's first shaping must say its size (cols and rows)",
                ));
            }
            serde_json::Map::new()
        }
    };
    for (k, v) in patch.iter().filter(|(k, _)| *k != "h" && *k != "wl") {
        land.insert(k.clone(), v.clone());
    }
    let count = |key: &str, or: Option<u64>| {
        land.get(key)
            .and_then(Value::as_u64)
            .or(or)
            .filter(|&n| n > 0)
            .ok_or_else(|| Error::new(format!("meta.land.{key} must be a whole number > 0")))
    };
    let (cols, rows, tile) = (
        count("cols", None)?,
        count("rows", None)?,
        count("tile", Some(64))?,
    );
    let tiles = (cols.div_ceil(tile) * rows.div_ceil(tile)) as usize;
    if tiles > MOST_LAND_TILES {
        return Err(Error::new(format!(
            "meta.land has {tiles} tiles, more than {MOST_LAND_TILES}"
        )));
    }
    for key in ["h", "wl"] {
        // (Holes in an array the page coded with some tiles only come as nulls: empty tiles.)
        let mut codes: Vec<Value> = match land.remove(key) {
            Some(Value::Array(a)) => a
                .into_iter()
                .map(|v| if v.is_string() { v } else { Value::from("") })
                .collect(),
            _ => Vec::new(),
        };
        codes.resize(tiles, Value::from(""));
        match patch.get(key) {
            None | Some(Value::Null) => {}
            Some(Value::Object(by_tile)) => {
                for (t, code) in by_tile {
                    let t: usize = t
                        .parse()
                        .ok()
                        .filter(|&t| t < tiles)
                        .ok_or_else(|| Error::new(format!("meta.land.{key}: no tile {t}")))?;
                    if !code.is_string() {
                        return Err(Error::new(format!("meta.land.{key}[{t}] must be a string")));
                    }
                    codes[t] = code.clone();
                }
            }
            Some(_) => {
                return Err(Error::new(format!(
                    "meta.land.{key} must be an object of tiles by index"
                )));
            }
        }
        land.insert(key.into(), Value::Array(codes));
    }
    Ok(Value::Object(land))
}
