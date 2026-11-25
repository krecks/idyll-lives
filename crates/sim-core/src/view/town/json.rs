use super::*;

pub fn structure_json(world: &World) -> String {
    structure_json_with(world, true)
}

/// Food and dishes for the renderer (`ItemPlacement` in protocol.ts).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ItemView<'a> {
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
    // (What each stands on or in, in one pass over the objects: one pass for each item was most
    // of a super-speed step in a big town, `items::holders`.)
    let ats: Vec<crate::items::ItemAt> = world.items.list.iter().map(|it| it.at).collect();
    let holders = crate::items::holders(content, &world.objects, &ats);
    let items: Vec<ItemView> = world
        .items
        .list
        .iter()
        .zip(holders)
        .map(|(it, holder)| {
            let [x, z] = it.at.centre();
            let holder = holder.map(|o| &world.objects[o as usize]);
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
