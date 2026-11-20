use super::*;

/// What changed in which storage chunk (`lot::CHUNK` square; stacked rows, as `structure_rect`
/// takes them): a fingerprint per chunk of its walls, doors, windows, fences, diagonals, floor
/// coverings and the objects standing there (as views send them), row by row over
/// `Lot::chunk_grid`. Compared from one structure version to the next, it tells a view which
/// chunks to fetch again.
pub fn chunk_fingerprints(world: &World) -> (usize, usize, Vec<u64>) {
    use std::hash::{Hash, Hasher};
    let lot = &world.lot;
    let (cols, rows) = lot.chunk_grid();
    let mut hashers: Vec<Option<std::collections::hash_map::DefaultHasher>> =
        (0..cols * rows).map(|_| None).collect();
    let chunk =
        |x: i32, z: i32| lot.chunk_at(x.clamp(0, lot.width as i32), z.clamp(0, lot.depth as i32));
    macro_rules! at {
        ($x:expr, $z:expr) => {
            hashers[chunk($x, $z)].get_or_insert_with(Default::default)
        };
    }
    for (e, edge) in lot.walls() {
        let (x, z) = match e {
            crate::lot::EdgeRef::H(x, z)
            | crate::lot::EdgeRef::V(x, z)
            | crate::lot::EdgeRef::Diag(x, z) => (x as i32, z as i32),
        };
        let h = at!(x, z);
        e.hash(h);
        (edge as u8).hash(h);
        let l = lot.look(e);
        (l.sides, l.form, l.style, l.sill, l.head).hash(h);
        if let crate::lot::EdgeRef::Diag(..) = e {
            lot.half_rooms()[lot.tile_index(x, z)].hash(h);
        }
    }
    for (x, z, c) in lot.floors() {
        let h = at!(x as i32, z as i32);
        (x, z, c).hash(h);
    }
    let lifts = surface_lifts(world);
    for o in &world.objects {
        let v = object_view(world, &lifts, o);
        let h = at!(o.x, o.z);
        serde_json::to_string(&v).unwrap_or_default().hash(h);
    }
    let out = hashers
        .into_iter()
        .map(|h| h.map_or(0, |h| h.finish()))
        .collect();
    (cols, rows, out)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RectView<'a> {
    /// The tiles `[x0, z0, x1, z1)` (stacked rows) this covers.
    pub rect: [i32; 4],
    pub version: u32,
    pub lot_version: u32,
    pub(crate) walls: Vec<EdgeView>,
    pub(crate) openings: Vec<OpeningView>,
    pub(crate) fences: Vec<FenceView>,
    pub(crate) diagonals: Vec<DiagonalView>,
    pub floors: Vec<[u16; 3]>,
    /// The objects whose tile (`x`, `z`) is inside.
    pub(crate) objects: Vec<ObjectView<'a>>,
}

/// The walls, doors, windows, fences, diagonals, floor coverings and objects inside tiles
/// `[x0, z0, x1, z1)` (stacked rows: storey `k` at rows `k * storeyDepth ..`), as
/// `structure_json` gives them for the whole lot (JSON). An edge counts where the tile holding it
/// is (its `+z` / `+x` side's tile), an object where its tile is.
pub fn structure_rect_json(world: &World, rect: [i32; 4]) -> String {
    let lot = &world.lot;
    let [x0, z0, x1, z1] = rect;
    let inside = |x: i32, z: i32| x >= x0 && z >= z0 && x < x1 && z < z1;
    let list = lot.walls_in(rect);
    let (walls, openings, fences) = wall_edges(
        lot,
        list.iter()
            .copied()
            .filter(|(e, _)| !matches!(e, crate::lot::EdgeRef::Diag(..))),
    );
    let diagonals = diagonals(
        lot,
        list.iter().filter_map(|&(e, _)| match e {
            crate::lot::EdgeRef::Diag(x, z) => lot
                .diag(x.into(), z.into())
                .map(|d| (usize::from(x), usize::from(z), d)),
            _ => None,
        }),
    );
    let floors = lot
        .floors()
        .filter(|&(x, z, _)| inside(x.into(), z.into()))
        .map(|(x, z, c)| [x, z, u16::from(c)])
        .collect();
    let lifts = surface_lifts(world);
    let objects = world
        .objects
        .iter()
        .filter(|o| inside(o.x, o.z))
        .map(|o| object_view(world, &lifts, o))
        .collect();
    serde_json::to_string(&RectView {
        rect,
        version: world.structure_version(),
        lot_version: world.lot_version(),
        walls,
        openings,
        fences,
        diagonals,
        floors,
        objects,
    })
    .unwrap_or_else(|_| "{}".into())
}

pub fn structure_json(world: &World) -> String {
    structure_json_with(world, true)
}

/// Food and dishes for the renderer (`ItemPlacement` in protocol.ts).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ItemView<'a> {
    pub id: u32,
    pub def: &'a str,
    /// Where it stands, in tiles (`z` on the lot's rows, like objects; see `storeys.rs`).
    pub x: f32,
    pub z: f32,
    /// Height above its storey's floor (m): the top it stands on.
    pub y: f32,
    /// Turned like what it stands on (radians).
    pub yaw: f32,
    pub servings: u8,
    /// `fresh`, `spoiled`, `dirty` (a dirty plate or pot), or `stored` (in a fridge: not drawn).
    pub state: &'static str,
    pub household: u32,
}

/// Every item: food and dishes on surfaces, on the floor and in fridges (JSON array, by id), and
/// the parcels of things bought waiting to be set up (`setup::parcels`, on the floor).
pub fn items_json(world: &World) -> String {
    let content = &world.content;
    let parcels = crate::setup::parcels(world)
        .into_iter()
        .map(|(id, def, [x, z], household)| ItemView {
            id,
            def: &content.items[def].id,
            x: round_to(x, 100.0),
            z: round_to(z, 100.0),
            y: 0.0,
            yaw: 0.0,
            servings: 0,
            state: "fresh",
            household,
        });
    let items: Vec<ItemView> = world
        .items
        .list
        .iter()
        .map(|it| {
            let [x, z] = it.at.centre();
            let holder = crate::items::holder(content, &world.objects, &it.at)
                .map(|o| &world.objects[o as usize]);
            let y = match it.at {
                crate::items::ItemAt::Surface { .. } => holder
                    .and_then(|o| content.objects[o.def].surface)
                    .map_or(content.cooking.default_surface_height, |s| s.height),
                _ => 0.0,
            };
            let state = match it.at {
                crate::items::ItemAt::Stored { .. } => "stored",
                _ if it.spoiled => "spoiled",
                _ if content.items[it.def].dirty => "dirty",
                _ => "fresh",
            };
            ItemView {
                id: it.id,
                def: &content.items[it.def].id,
                x: round_to(x, 100.0),
                z: round_to(z, 100.0),
                y: round_to(y, 100.0),
                yaw: holder.map_or(0.0, |o| {
                    round_to(o.yaw() + f32::from(o.turn).to_radians(), 1000.0)
                }),
                servings: it.servings,
                state,
                household: it.household,
            }
        })
        .chain(parcels)
        .collect();
    serde_json::to_string(&items).expect("items serialize")
}

/// The world structure; without `lot`, leaves out the lot itself (rooms, walls, openings,
/// diagonals, fences, floors), as `StructureSync` does unless asked for it (the tests compare
/// the two).
pub fn structure_json_with(world: &World, lot: bool) -> String {
    let lifts = surface_lifts(world);
    let objects = world
        .objects
        .iter()
        .map(|o| object_view(world, &lifts, o))
        .collect();
    let (walls, openings, fences) = if lot {
        let (w, o, f) = wall_edges(&world.lot, world.lot.walls());
        (Some(w), Some(o), Some(f))
    } else {
        (None, None, None)
    };
    let diagonals = lot.then(|| diagonals(&world.lot, world.lot.diagonal_walls()));
    serde_json::to_string(&StructureView {
        version: world.structure_version(),
        lot_version: world.lot_version(),
        mode: world.mode,
        width: world.lot.width,
        depth: world.lot.depth,
        storeys: world.storeys,
        storey_depth: world.storey_depth(),
        basement: basement_block(world),
        objects,
        sims: sim_infos(world),
        former: &world.former,
        households: household_views(world),
        plots: plot_views(world),
        exits: &world.exits,
        rooms: lot.then(|| world.lot.rooms_row_major()),
        walls,
        openings,
        fences,
        diagonals,
        floors: lot.then(|| floor_views(world)),
        meta: &world.meta,
        staff: staff_views(world),
        cars: car_views(world),
        garages: &world.garages,
        porches: &world.porches,
        porch_pieces: &world.porch_pieces,
        pull_over_streets: pull_over_street_views(world),
        light_poles: light_pole_views(world),
        junctions: junction_views(world),
        parking: parking_views(world),
        car_parks: &world.travel.file.car_parks,
        flats: flat_views(world),
    })
    .expect("structure serializes")
}
