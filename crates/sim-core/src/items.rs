//! Things residents put down and pick up: dishes of food on tables and counters, leftovers in
//! the fridge, dirty plates waiting to be washed (content `items`; see `cooking.rs`).
//!
//! Items aren't objects: they take no tile, nobody buys them, and their ids never change (object
//! ids shift when things are sold or moved). Where an item is, is a place (a tile and an offset,
//! like an object's), never an object id: the surface or fridge it's on or in is whatever object
//! stands there (`surface::host_at`, `storing_object`), so items follow their host when it moves
//! (`World::carry_items`) and go with it when it's sold.

use crate::content::Content;
use crate::world::ObjectInstance;

/// Where an item is. `x, z` is a tile (on any storey: the lot's rows hold every storey), the
/// offset centimetres off its middle (as `ObjectInstance::offset`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemAt {
    /// Standing on the surface (`content::Surface`) of the object over this tile.
    Surface { x: i32, z: i32, offset: [i8; 2] },
    /// Put away in the object over this tile (a fridge; see `ObjectDef::stores`).
    Stored { x: i32, z: i32 },
    /// On the floor (nowhere else to put it).
    Floor { x: i32, z: i32, offset: [i8; 2] },
}

impl ItemAt {
    pub fn tile(&self) -> (i32, i32) {
        match *self {
            ItemAt::Surface { x, z, .. } | ItemAt::Stored { x, z } | ItemAt::Floor { x, z, .. } => {
                (x, z)
            }
        }
    }

    /// Where it is, in tiles.
    pub fn centre(&self) -> [f32; 2] {
        let (x, z) = self.tile();
        let offset = match *self {
            ItemAt::Surface { offset, .. } | ItemAt::Floor { offset, .. } => offset,
            ItemAt::Stored { .. } => [0, 0],
        };
        [
            x as f32 + 0.5 + offset[0] as f32 / 100.0,
            z as f32 + 0.5 + offset[1] as f32 / 100.0,
        ]
    }

    /// The same kind of place moved to point `p` (tiles).
    pub fn moved_to(&self, p: [f32; 2]) -> ItemAt {
        let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
        let cm = |v: f32| ((v - v.floor() - 0.5) * 100.0).round().clamp(-50.0, 50.0) as i8;
        let offset = [cm(p[0]), cm(p[1])];
        match self {
            ItemAt::Surface { .. } => ItemAt::Surface { x, z, offset },
            ItemAt::Stored { .. } => ItemAt::Stored { x, z },
            ItemAt::Floor { .. } => ItemAt::Floor { x, z, offset },
        }
    }

    /// A place on a surface (`surface::SurfaceSpot`).
    pub fn on(spot: &crate::surface::SurfaceSpot) -> ItemAt {
        ItemAt::Surface {
            x: spot.x,
            z: spot.z,
            offset: spot.offset,
        }
    }
}

/// An item somewhere in town.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemInstance {
    /// Stable for the item's life (never reused).
    pub id: u32,
    /// Index into `Content::items`.
    pub def: usize,
    pub at: ItemAt,
    /// Servings left (0: not food, or eaten up).
    pub servings: u8,
    /// How good the food is (eating gives the recipe's effects times this).
    pub quality: f32,
    /// Tick it was made.
    pub made_at: u64,
    /// The household it belongs to.
    pub household: u32,
    /// Gone off: nobody eats it, and it makes the room dirty until it's cleared away.
    pub spoiled: bool,
    /// The recipe that made it (food).
    pub recipe: Option<usize>,
}

impl ItemInstance {
    /// Something someone could eat.
    pub fn edible(&self) -> bool {
        self.servings > 0 && self.recipe.is_some() && !self.spoiled
    }

    /// Something to wash up or throw out.
    pub fn to_clear(&self, content: &Content) -> bool {
        self.spoiled || content.items[self.def].dirty
    }
}

/// What a resident has in their hands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Held {
    pub def: usize,
    /// Servings of food, or how many dirty things they carry.
    pub servings: u8,
    pub quality: f32,
    pub made_at: u64,
    pub recipe: Option<usize>,
}

impl Held {
    pub fn of(def: usize) -> Held {
        Held {
            def,
            servings: 0,
            quality: 1.0,
            made_at: 0,
            recipe: None,
        }
    }
}

/// Every item in town, in id order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Items {
    pub list: Vec<ItemInstance>,
    /// The next item's id.
    pub next_id: u32,
    /// Bumped when anything about them changes (views send them again).
    pub version: u32,
    /// Servings promised to a Sim who is on their way to eat them (`(item, sim)`). Not saved:
    /// a loaded world reserves again as its eaters start.
    reserved: Vec<(u32, u32)>,
}

impl Items {
    /// Adds an item (its `id` is set here) and returns its id.
    pub fn add(&mut self, mut item: ItemInstance) -> u32 {
        item.id = self.next_id;
        self.next_id += 1;
        self.list.push(item);
        self.version = self.version.wrapping_add(1);
        self.next_id - 1
    }

    pub fn get(&self, id: u32) -> Option<&ItemInstance> {
        self.position(id).map(|i| &self.list[i])
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut ItemInstance> {
        let i = self.position(id)?;
        self.version = self.version.wrapping_add(1);
        Some(&mut self.list[i])
    }

    pub fn remove(&mut self, id: u32) -> Option<ItemInstance> {
        let i = self.position(id)?;
        self.version = self.version.wrapping_add(1);
        self.reserved.retain(|&(item, _)| item != id);
        Some(self.list.remove(i))
    }

    /// Whether `sim` may have a serving of `id`: they have promised themselves one, or there are
    /// more servings left than others have promised themselves (a pot feeds the whole table).
    pub fn free_for(&self, id: u32, sim: u32) -> bool {
        let mut others = 0u32;
        for &(item, by) in &self.reserved {
            if item == id {
                if by == sim {
                    return true;
                }
                others += 1;
            }
        }
        self.get(id)
            .is_some_and(|it| u32::from(it.servings) > others)
    }

    /// `household`'s servings `sim` may have (edible, and not all promised to others).
    pub fn free_servings(&self, household: u32, sim: u32) -> impl Iterator<Item = &ItemInstance> {
        self.list
            .iter()
            .filter(move |it| it.household == household && it.edible() && self.free_for(it.id, sim))
    }

    /// Promises a serving of `id` to `sim`. False if the others' promises take every serving.
    pub fn reserve(&mut self, id: u32, sim: u32) -> bool {
        if self.reserved.contains(&(id, sim)) {
            return true;
        }
        if !self.free_for(id, sim) {
            return false;
        }
        self.reserved.push((id, sim));
        true
    }

    /// Gives up `sim`'s promise on `id` (the serving is in their hands, or gone).
    pub fn release(&mut self, id: u32, sim: u32) {
        self.reserved
            .retain(|&(item, owner)| item != id || owner != sim);
    }

    /// Drops promises nobody is still on their way to eat. Called after a tick's steps, so a
    /// task that ended or was replaced cannot keep a serving out of reach.
    pub fn release_stale(&mut self, mut still_eating: impl FnMut(u32, u32) -> bool) {
        self.reserved.retain(|&(item, sim)| still_eating(item, sim));
    }

    fn position(&self, id: u32) -> Option<usize> {
        self.list.binary_search_by_key(&id, |it| it.id).ok()
    }

    /// What stands on surfaces (middle and radius), for `surface::free_surface_spot_near`.
    pub fn occupancy(&self, content: &Content) -> Vec<([f32; 2], f32)> {
        self.list
            .iter()
            .filter(|it| matches!(it.at, ItemAt::Surface { .. }))
            .map(|it| (it.at.centre(), content.items[it.def].radius))
            .collect()
    }
}

/// The object that keeps things at tile `(x, z)` (a fridge; see `ObjectDef::stores`).
pub fn storing_object(
    content: &Content,
    objects: &[ObjectInstance],
    x: i32,
    z: i32,
) -> Option<u32> {
    objects.iter().find_map(|o| {
        (content.objects[o.def].stores && o.tiles(content).any(|t| t == (x, z))).then_some(o.id)
    })
}

/// The object an item is on or in (none on the floor, or when it's gone).
pub fn holder(content: &Content, objects: &[ObjectInstance], at: &ItemAt) -> Option<u32> {
    match *at {
        ItemAt::Surface { x, z, .. } => crate::surface::host_at(content, objects, x, z),
        ItemAt::Stored { x, z } => storing_object(content, objects, x, z),
        ItemAt::Floor { .. } => None,
    }
}

/// `holder` of each of `ats`, in one pass over the objects rather than one for each: a big town
/// has thousands of objects, and the sweep over its items (`cooking::update`) asks for every one.
/// The same answers, the first object (by id) over each tile (the sim tests check it against
/// `holder`: `common::check_lookups`).
pub fn holders(content: &Content, objects: &[ObjectInstance], ats: &[ItemAt]) -> Vec<Option<u32>> {
    use std::collections::HashMap;
    // The tiles asked about, on a top and put away, and the first object found for each.
    let mut tops: HashMap<(i32, i32), Option<u32>> = HashMap::new();
    let mut stores: HashMap<(i32, i32), Option<u32>> = HashMap::new();
    for at in ats {
        match *at {
            ItemAt::Surface { x, z, .. } => tops.insert((x, z), None),
            ItemAt::Stored { x, z } => stores.insert((x, z), None),
            ItemAt::Floor { .. } => None,
        };
    }
    for o in objects {
        let def = &content.objects[o.def];
        if def.surface.is_some() && !tops.is_empty() {
            let (w, d) = o.size(content);
            for z in o.z..o.z + d {
                for x in o.x..o.x + w {
                    if let Some(host @ None) = tops.get_mut(&(x, z)) {
                        *host = Some(o.id);
                    }
                }
            }
        }
        if def.stores && !stores.is_empty() {
            for t in o.tiles(content) {
                if let Some(host @ None) = stores.get_mut(&t) {
                    *host = Some(o.id);
                }
            }
        }
    }
    ats.iter()
        .map(|at| match *at {
            ItemAt::Surface { x, z, .. } => tops[&(x, z)],
            ItemAt::Stored { x, z } => stores[&(x, z)],
            ItemAt::Floor { .. } => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serving() -> ItemInstance {
        ItemInstance {
            id: 0,
            def: 0,
            at: ItemAt::Floor {
                x: 0,
                z: 0,
                offset: [0, 0],
            },
            servings: 1,
            quality: 1.0,
            made_at: 0,
            household: 0,
            spoiled: false,
            recipe: Some(0),
        }
    }

    #[test]
    fn a_serving_is_promised_to_one_sim_at_a_time() {
        let mut items = Items::default();
        let first = items.add(serving());
        let second = items.add(serving());
        assert!(items.reserve(first, 1));
        assert!(items.free_for(first, 1) && !items.free_for(first, 2));
        assert!(items.reserve(first, 1), "the same sim may keep it");
        assert!(!items.reserve(first, 2), "another sim may not take it");
        assert!(!items.free_for(first, 2));
        assert!(items.reserve(second, 2));
        items.release(first, 1);
        assert!(items.reserve(first, 2));
        items.remove(second);
        assert!(items.reserved.iter().all(|&(item, _)| item != second));
        items.release_stale(|_, _| false);
        assert!(items.free_for(first, 1));
    }

    #[test]
    fn a_pot_is_promised_a_serving_at_a_time() {
        let mut items = Items::default();
        let pot = items.add(ItemInstance {
            servings: 2,
            ..serving()
        });
        assert!(items.reserve(pot, 1));
        assert!(items.free_for(pot, 2), "a second serving is left");
        assert!(items.reserve(pot, 2));
        assert!(!items.free_for(pot, 3), "both servings are promised");
        assert!(!items.reserve(pot, 3));
    }
}
