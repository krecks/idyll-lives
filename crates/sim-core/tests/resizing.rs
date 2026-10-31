//! Resizing: the player can give anything (but stairs) its own size, 10 cm to `MAX_SIZE` tiles
//! each way (it takes whole tiles, rounded up), and make it taller or shorter, to the centimetre.
//! A bigger table takes more floor and has a bigger top; a taller one lifts what stands on it.
//! Things on a wall keep their depth, things on a top their footprint. Sizes are saved, undone
//! and carried by blueprints.

use sim_core::surface::{Top, host_of};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "objects":[
      {"id":"table","name":"Table","price":200,"footprint":[2,1],"surface":{"height":0.75,"kind":"table"}},
      {"id":"vase","name":"Vase","price":30,"layer":"surface","radius":0.1},
      {"id":"picture","name":"Picture","price":50,"layer":"wall","interactions":[]},
      {"id":"stairs","name":"Stairs","price":400,"footprint":[1,4],"stairs":true,"interactions":[]}],
    "economy":{"startingFunds":5000}}"#;

/// One room (x 1..9, z 1..6) with a door at the bottom, on a 12×10 plot.
const TOWN: &str = r#"{"width":12,"depth":12,
    "walls":[[1,1,10,1],[1,7,10,7],[1,1,1,7],[10,1,10,7]],
    "doors":[{"x":5,"z":7,"axis":"x"}],
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true}]}"#;

fn world() -> World {
    World::from_json(CONTENT, TOWN, 1).unwrap()
}

fn buy(
    w: &mut World,
    def: &str,
    x: i32,
    z: i32,
    rot: i32,
    size: Option<[u16; 2]>,
    tall: Option<i16>,
) -> Result<u32, sim_core::Error> {
    w.apply(Command::Buy {
        household: 0,
        object: def.into(),
        at: Some([x, z, rot]),
        colors: Default::default(),
        turn: None,
        offset: None,
        size,
        tall,
    })?;
    Ok(w.objects.last().unwrap().stable)
}

fn resize(
    w: &mut World,
    object: u32,
    x: i32,
    z: i32,
    size: [u16; 2],
    tall: i16,
) -> Result<(), sim_core::Error> {
    let rot = w.objects[object as usize].rot;
    w.apply(Command::MoveObject {
        household: 0,
        object,
        x,
        z,
        rot,
        turn: None,
        offset: None,
        size: Some(size),
        tall: Some(tall),
    })
}

fn find(w: &World, def: &str) -> u32 {
    w.objects
        .iter()
        .find(|o| w.content.objects[o.def].id == def)
        .unwrap()
        .id
}

#[test]
fn a_bigger_table_takes_more_floor_and_has_a_bigger_higher_top() {
    let mut w = world();
    let table = buy(&mut w, "table", 2, 3, 0, None, None).unwrap();
    assert_eq!(
        w.objects[table as usize].footprint,
        [2, 1],
        "its kind's size to begin with"
    );
    let vase = buy(&mut w, "vase", 2, 3, 0, None, None).unwrap();

    resize(&mut w, table, 2, 3, [300, 200], 30).unwrap();
    let table = find(&w, "table");
    let t = &w.objects[table as usize];
    assert_eq!(
        (t.footprint, t.extent, t.tall, t.size(&w.content)),
        ([3, 2], [300, 200], 30, (3, 2))
    );
    let top = Top::of(&w.content, t).unwrap();
    assert_eq!(top.half, [1.5, 1.0], "the top grows with the footprint");
    assert!((top.surface.height - 1.05).abs() < 1e-5, "and rises 30 cm");
    // The vase went with it, onto the new top.
    let vase = w
        .objects
        .iter()
        .position(|o| w.content.objects[o.def].id == "vase")
        .unwrap_or(vase as usize);
    assert_eq!(
        host_of(&w.content, &w.objects, &w.objects[vase]),
        Some(table)
    );
    // The new tiles are taken: nothing else goes there.
    assert!(buy(&mut w, "table", 4, 4, 0, None, None).is_err());

    // Undo puts the old size back.
    w.apply(Command::Undo { household: 0 }).unwrap();
    let t = &w.objects[find(&w, "table") as usize];
    assert_eq!((t.footprint, t.tall), ([2, 1], 0));

    // Any size in between takes its tiles rounded up, and its top is its own size.
    let table = find(&w, "table");
    resize(&mut w, table, 2, 3, [150, 80], 7).unwrap();
    let t = &w.objects[find(&w, "table") as usize];
    assert_eq!((t.footprint, t.extent, t.tall), ([2, 1], [150, 80], 7));
    let top = Top::of(&w.content, t).unwrap();
    assert!(
        (top.half[0] - 0.75).abs() < 1e-5 && (top.half[1] - 0.4).abs() < 1e-5,
        "{:?}",
        top.half
    );
    assert!((top.surface.height - 0.82).abs() < 1e-5);
}

#[test]
fn sizes_have_limits() {
    let mut w = world();
    let table = buy(&mut w, "table", 2, 3, 0, None, None).unwrap();
    let err =
        |w: &mut World, size, tall| resize(w, table, 2, 3, size, tall).unwrap_err().to_string();
    assert!(err(&mut w, [601, 100], 0).contains("10 cm to 6 m"));
    assert!(err(&mut w, [9, 100], 0).contains("10 cm to 6 m"));
    assert!(err(&mut w, [200, 100], 201).contains("at most"));
    assert!(err(&mut w, [200, 100], -101).contains("at most"));
    // Too big for the room: nothing changes.
    assert!(resize(&mut w, table, 2, 3, [600, 600], 0).is_err());
    assert_eq!(w.objects[table as usize].footprint, [2, 1]);

    // Stairs keep their size; a vase only gets taller or shorter; a picture keeps its depth.
    assert!(
        buy(&mut w, "stairs", 8, 2, 0, Some([200, 400]), None)
            .unwrap_err()
            .to_string()
            .contains("keep their size")
    );
    buy(&mut w, "vase", 2, 3, 0, None, Some(20)).unwrap();
    assert!(
        buy(&mut w, "vase", 3, 3, 0, Some([200, 200]), None)
            .unwrap_err()
            .to_string()
            .contains("taller or shorter")
    );
    // A picture on the back wall (it faces +z, its back on the wall at z = 1).
    let picture = buy(&mut w, "picture", 6, 1, 0, Some([250, 100]), Some(40)).unwrap();
    assert_eq!(
        w.objects[picture as usize].footprint,
        [3, 1],
        "wider along its wall"
    );
    assert!(
        buy(&mut w, "picture", 3, 1, 0, Some([100, 200]), None)
            .unwrap_err()
            .to_string()
            .contains("flat on its wall")
    );
}

#[test]
fn things_on_a_wall_slide_only_along_it() {
    let mut w = world();
    let buy_off = |w: &mut World, x, offset| {
        w.apply(Command::Buy {
            household: 0,
            object: "picture".into(),
            at: Some([x, 1, 0]),
            colors: Default::default(),
            turn: None,
            offset: Some(offset),
            size: None,
            tall: None,
        })
    };
    buy_off(&mut w, 3, [30, 0]).unwrap();
    assert!(
        buy_off(&mut w, 5, [0, 20])
            .unwrap_err()
            .to_string()
            .contains("flat on its wall")
    );
}

#[test]
fn sizes_are_saved_and_carried_by_blueprints() {
    let mut w = world();
    let table = buy(&mut w, "table", 2, 3, 1, Some([300, 100]), Some(-20)).unwrap();
    assert_eq!(
        w.objects[table as usize].size(&w.content),
        (1, 3),
        "turned, it runs the other way"
    );
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    let t = &loaded.objects[table as usize];
    assert_eq!((t.footprint, t.tall), ([3, 1], -20));

    let bp = w.blueprint(0).unwrap();
    assert_eq!(
        (bp.objects[0].size, bp.objects[0].tall),
        (Some([300, 100]), -20)
    );
}

#[test]
fn windows_and_doors_get_their_own_height_for_free() {
    use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
    use sim_core::lot::EdgeRef;
    let mut w = world();
    let window = |sill, head| EdgeEdit {
        sill,
        head,
        ..EdgeEdit::new(EdgeAxis::H, 3, 1, EdgeKind::Window)
    };
    let build = |w: &mut World, e: EdgeEdit| {
        w.apply(Command::Build {
            household: 0,
            edits: vec![e],
        })
    };
    build(&mut w, window(None, None)).unwrap();
    assert_eq!(
        (
            w.lot.look(EdgeRef::H(3, 1)).sill,
            w.lot.look(EdgeRef::H(3, 1)).head
        ),
        (None, None),
        "as its style comes"
    );

    let funds = w.households[0].funds;
    build(&mut w, window(Some(40), Some(253))).unwrap();
    assert_eq!(w.households[0].funds, funds, "resizing is free");
    let look = w.lot.look(EdgeRef::H(3, 1));
    assert_eq!(
        (look.sill, look.head),
        (Some(40), Some(253)),
        "to the centimetre"
    );
    // Restyled, it keeps its size.
    build(
        &mut w,
        EdgeEdit {
            style: Some(0),
            ..EdgeEdit::new(EdgeAxis::H, 3, 1, EdgeKind::Window)
        },
    )
    .unwrap();
    assert_eq!(w.lot.look(EdgeRef::H(3, 1)).head, Some(253));

    assert!(
        build(&mut w, window(Some(200), Some(220)))
            .unwrap_err()
            .to_string()
            .contains("at least")
    );
    assert!(
        build(&mut w, window(None, Some(271)))
            .unwrap_err()
            .to_string()
            .contains("at most")
    );
    let door = |sill, head| EdgeEdit {
        sill,
        head,
        ..EdgeEdit::new(EdgeAxis::H, 7, 1, EdgeKind::Door)
    };
    assert!(
        build(&mut w, door(Some(2), None))
            .unwrap_err()
            .to_string()
            .contains("floor")
    );
    assert!(
        build(&mut w, door(None, Some(150)))
            .unwrap_err()
            .to_string()
            .contains("tall")
    );
    build(&mut w, door(None, Some(240))).unwrap();

    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.lot.look(EdgeRef::H(3, 1)).sill, Some(40));
    assert_eq!(loaded.lot.look(EdgeRef::H(7, 1)).head, Some(240));
}
