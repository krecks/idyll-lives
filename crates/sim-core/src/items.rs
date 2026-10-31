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
        Some(self.list.remove(i))
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
