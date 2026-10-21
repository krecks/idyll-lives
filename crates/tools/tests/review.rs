//! `cargo review` on small fixtures (tests/fixtures/review/reference.json and a few objects made here): the
//! range logic, which kind an object is, and each flag of `uses`; and once over the shipped content.

use ll_tools::checkglb::{Height, Measure, Measures, Paths};
use ll_tools::review::{self, Flag, Object, Reference, Use};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn reference() -> Reference {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/review/reference.json");
    Reference::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn object(id: &str, category: &str, group: &str) -> Object {
    Object { id: id.into(), name: id.into(), page: category.into(), category: category.into(), group: group.into(), model: format!("model.{id}"), slots: 1, approach: "front", ..Object::default() }
}

fn using(id: &str, anim: &str, minutes: f32, tags: &[&str]) -> Use {
    Use { id: id.into(), anim: anim.into(), pose: "stand", minutes, tags: tags.iter().map(|t| t.to_string()).collect(), ..Use::default() }
}

fn height(what: &'static str, expected: f64, mesh: Option<f64>) -> Height {
    Height { what, expected, source: "test".into(), at: [0.0, 0.0], open: None, mesh, span: None, disagrees: None }
}

fn measured(size: [f64; 3], heights: Vec<Height>) -> Result<Measure, String> {
    Ok(Measure { model: String::new(), size, heights, ..Measure::default() })
}

#[test]
fn a_range_takes_the_tolerance_at_each_end_and_functional_heights_none() {
    assert!(review::within(0.40, [0.42, 0.48], 0.1), "0.378 is the low end less 10 %");
    assert!(!review::within(0.37, [0.42, 0.48], 0.1));
    assert!(review::within(0.528, [0.42, 0.48], 0.1) && !review::within(0.53, [0.42, 0.48], 0.1));
    assert!(review::within(0.48, [0.42, 0.48], 0.0) && review::within(0.4803, [0.42, 0.48], 0.0), "half a millimetre of f32 slack");
    assert!(!review::within(0.49, [0.42, 0.48], 0.0));

    let r = reference();
    let chair = object("chair", "kitchen", "chairs");
    let mut chair = chair;
    chair.model = "model.sharedChair".into();
    // H 1.05 is within 1.00 + 10 %; D 0.40 is under 0.45 - 10 %; the seat, strict, is 1 cm over.
    let row = review::size_row(&chair, Some(&measured([0.5, 1.05, 0.4], vec![height("seat", 0.49, Some(0.49))])), &r);
    assert_eq!(row.kind.as_deref(), Some("dining-chair"));
    assert_eq!(row.outside, ["D 0.400 < 0.405 (0.45–0.60 ±10 %)", "seat 0.490 > 0.480 (0.42–0.48)"]);
    assert!(row.status().starts_with("out: D 0.400"));
    // To the millimetre: 1.2104 is in 1.00 + 10 % (half a millimetre of slack), 1.211 isn't, and says so.
    assert_eq!(review::size_row(&chair, Some(&measured([0.5, 1.1004, 0.5], vec![height("seat", 0.46, None)])), &r).status(), "ok");
    assert_eq!(review::size_row(&chair, Some(&measured([0.5, 1.1011, 0.5], vec![height("seat", 0.46, None)])), &r).outside, ["H 1.101 > 1.100 (0.78–1.00 ±10 %)"]);
    // The model's height where it has one; the game's where it hasn't (nothing under the seat).
    let row = review::size_row(&chair, Some(&measured([0.5, 0.9, 0.5], vec![height("seat", 0.46, Some(0.40))])), &r);
    assert_eq!(row.outside, ["seat 0.400 < 0.420 (0.42–0.48)"]);
    let row = review::size_row(&chair, Some(&measured([0.5, 0.9, 0.5], vec![height("seat", 0.46, None)])), &r);
    assert_eq!(row.status(), "ok");
    // A kind's height the object has none of.
    let row = review::size_row(&chair, Some(&measured([0.5, 0.9, 0.5], vec![])), &r);
    assert_eq!(row.outside, ["no seat to measure (0.42–0.48)"]);
    // The lower bunk counts as the mattress; a kind without a measure checks nothing of it.
    let bed = object("sofaBed", "living", "seating");
    let mut bed = bed;
    bed.model = "model.sofaBed".into();
    let mut ids = r;
    ids.ids.insert("sofaBed".into(), "sofa-bed".into());
    let row = review::size_row(&bed, Some(&measured([2.0, 0.9, 0.9], vec![height("seat", 0.44, Some(0.44)), height("lower bunk", 0.44, Some(0.39))])), &ids);
    assert_eq!(row.outside, ["mattress 0.390 < 0.400 (0.40–0.50)"]);
}

#[test]
fn a_top_the_content_hasnt_a_table_without_its_seats_and_a_stairs_top_step() {
    let r = reference();
    let table = object("table", "kitchen", "dining");
    let with = |size: [f64; 3], heights: Vec<Height>, f: &dyn Fn(&mut Measure)| -> Result<Measure, String> {
        let mut m = measured(size, heights).unwrap();
        f(&mut m);
        Ok(m)
    };
    // Its kind has a top, its content none: the model's, near the range (0.72–0.76 less or more 20 %).
    let row = review::size_row(&table, Some(&with([1.2, 0.75, 0.8], vec![], &|m| m.top = Some(([0.0, 0.0], 0.74)))), &r);
    assert_eq!((row.top, row.status().as_str()), (Some(0.74), "ok"));
    let (_, files, _) = review::sizes(std::slice::from_ref(&table), &Measures { models: BTreeMap::new(), objects: BTreeMap::from([("table".to_string(), with([1.2, 0.75, 0.8], vec![], &|m| m.top = Some(([0.0, 0.0], 0.74))))]) }, &r);
    assert!(files["sizes-kitchen.md"].contains("| `table` table | dining-table | 1.20 × 0.75 × 0.80 | — | 0.74 (no content `surface`) | — | ok |"), "{}", files["sizes-kitchen.md"]);
    let row = review::size_row(&table, Some(&with([1.2, 0.75, 0.8], vec![], &|m| m.top = Some(([0.0, 0.0], 0.70)))), &r);
    assert_eq!(row.outside, ["top 0.700 < 0.720 (0.72–0.76)"]);
    // Farther than that it's something else (a shelf, a seat): no top, and the nearest face says what there is.
    let row = review::size_row(&table, Some(&with([1.2, 0.75, 0.8], vec![], &|m| m.top = Some(([0.0, 0.0], 0.45)))), &r);
    assert_eq!((row.top, row.outside.as_slice()), (None, ["no top to measure (0.72–0.76; the nearest face looking up 0.450)".to_string()].as_slice()));
    // The content's top wins over the model's found one.
    let row = review::size_row(&table, Some(&with([1.2, 0.75, 0.8], vec![height("surface", 0.75, Some(0.75))], &|m| m.top = Some(([0.0, 0.0], 0.70)))), &r);
    assert_eq!((row.top, row.status().as_str()), (None, "ok"));

    // A table's kind (a top, no seat) whose model carries its chairs: H is the table's.
    let row = review::size_row(&table, Some(&with([1.6, 1.03, 0.9], vec![height("seat", 0.46, Some(0.46)), height("surface", 0.75, Some(0.75))], &|m| m.table = Some(0.76))), &r);
    assert_eq!((row.h, row.status().as_str()), (Some(("table", 0.76)), "ok"));
    let row = review::size_row(&table, Some(&with([1.6, 1.03, 0.9], vec![height("surface", 0.75, Some(0.75))], &|m| m.table = Some(1.0))), &r);
    assert_eq!(row.outside, ["H (table) 1.000 > 0.858 (0.72–0.78 ±10 %)"]);
    // A set's kind (with a seat) has the whole set's ranges: its H stays the model's.
    let mut set = object("diningOak", "kitchen", "dining");
    set.model = "model.diningOak".into();
    let row = review::size_row(&set, Some(&with([1.6, 1.12, 0.9], vec![height("seat", 0.46, Some(0.46)), height("surface", 0.75, Some(0.75))], &|m| m.table = Some(0.76))), &r);
    assert_eq!((row.h, row.outside.as_slice()), (None, ["H 1.120 > 1.100 (0.78–1.00 ±10 %)".to_string()].as_slice()));
    // A desk's kind gives no top: its H is still the model's.
    let row = review::size_row(&object("desk", "office", "desks"), Some(&with([1.4, 1.2, 0.8], vec![], &|m| m.table = Some(0.75))), &r);
    assert_eq!(row.h, None);

    // A stair's H is its top step (the storey it climbs), not its rails over it.
    let stairs = object("stairs", "living", "");
    let m = with([0.86, 3.68, 4.0], vec![], &|m| m.steps = Some((2.8, 14)));
    let row = review::size_row(&stairs, Some(&m), &r);
    assert_eq!((row.h, row.steps.map(|(n, rise)| (n, (rise * 1000.0).round())), row.status().as_str()), (Some(("top step", 2.8)), Some((14, 200.0)), "ok"));
    let (_, files, _) = review::sizes(&[stairs], &Measures { models: BTreeMap::new(), objects: BTreeMap::from([("stairs".to_string(), m)]) }, &r);
    assert!(files["sizes-living.md"].contains("| 0.86 × 2.80 (top step; 14 steps of 0.200; 3.68 in all) × 4.00 |"), "{}", files["sizes-living.md"]);

    // What `measures` is asked to look for: each object's kind's top range.
    let tops = review::tops(&[table, set, object("chair", "kitchen", "chairs")], &r);
    assert_eq!(tops, BTreeMap::from([("diningOak".to_string(), [0.72, 0.76]), ("table".to_string(), [0.72, 0.76])]));
}

#[test]
fn the_kind_is_by_id_then_model_then_category_and_group_then_group() {
    let r = reference();
    assert_eq!(r.kind_of("stool", "model.stool", "kitchen", "dining"), Some("bar-stool"), "ids by id win over groups");
    assert_eq!(r.kind_of("chair2", "model.sharedChair", "kitchen", "dining"), Some("dining-chair"), "then ids by model key");
    assert_eq!(r.kind_of("table", "model.table", "kitchen", "dining"), Some("dining-table"), "then <category>/<group>");
    assert_eq!(r.kind_of("couch", "model.couch", "living", "seating"), Some("armchair"), "then the group alone");
    assert_eq!(r.kind_of("couch", "model.couch", "outdoor", "dining"), None, "kitchen/dining isn't outdoor's");
    assert_eq!(r.kind_of("thing", "model.thing", "kids", ""), None, "no group");
    assert_eq!(r.tolerance, 0.1);
    assert!(r.kind("vase").is_some() && r.kind("no-such-kind").is_none());

    // `sizes`: an object of no kind, or of a kind `kinds` hasn't, is an error (exit 1), listed at the end;
    // `--md` has every object, with its status.
    let objects = vec![object("stool", "kitchen", "dining"), object("thing", "kids", ""), object("ghost", "kids", "")];
    let mut found = BTreeMap::new();
    found.insert("stool".to_string(), measured([0.4, 0.75, 0.4], vec![height("seat", 0.72, Some(0.72))]));
    found.insert("thing".to_string(), measured([0.3, 0.3, 0.3], vec![]));
    found.insert("ghost".to_string(), Err("cannot read it".to_string()));
    let mut models = BTreeMap::new();
    models.insert("model.lampOnly".to_string(), Ok([0.3, 0.9, 0.3]));
    let (text, files, code) = review::sizes(&objects, &Measures { models, objects: found }, &r);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("## Error: 2 objects map to no kind"), "{text}");
    assert!(text.contains("- `thing` (kids, group ``, model.thing): no kind"), "{text}");
    assert!(text.contains("- `ghost` (kids, group ``, model.ghost): kind `no-such-kind` isn't in `kinds`"), "{text}");
    assert!(text.contains("## kitchen: 0 of 1 outside"), "{text}");
    // A model key in `ids` that no object draws is measured too.
    assert!(text.contains("## models: 1 of 1 outside") && text.contains("H 0.900 > 0.770 (0.30–0.70 ±10 %)"), "{text}");
    assert_eq!(files.keys().collect::<Vec<_>>(), ["sizes-kids.md", "sizes-kitchen.md", "sizes-models.md"]);
    assert!(files["sizes-kitchen.md"].contains("| `stool` stool | bar-stool | 0.40 × 0.75 × 0.40 | 0.72 | — | — | ok |"), "{}", files["sizes-kitchen.md"]);
    assert!(files["sizes-kids.md"].contains("| no kind |"));
}

/// A toy catalogue: a sofa bed whose model is missing `motions.open`, a fridge with a stray key, decor.
fn catalogue() -> (Vec<Object>, BTreeMap<String, Value>, BTreeSet<String>) {
    let mut sofa = object("sofaBed", "living", "seating");
    sofa.uses = vec![using("relax", "relax", 30.0, &["rest"]), using("doze", "nap", 30.0, &["rest", "nap"]), using("pullOut", "fetch", 2.0, &[])];
    sofa.uses[1].leaves = Some("open".into());
    let mut fridge = object("fridge", "kitchen", "appliances");
    fridge.uses = vec![using("snack", "fetch", 5.0, &["food"]), using("feast", "eat", 90.0, &["food"])];
    fridge.station_anims = ["stir".to_string()].into();
    let mut sink = object("sink", "kitchen", "sinks");
    sink.uses = vec![using("wash", "wash", 2.0, &["food"]), using("polish", "buff", 6.0, &["food"])];
    let mut table = object("table", "dining", "tables");
    table.uses = vec![using("dine", "eat", 15.0, &["food"]), using("lunch", "eat", 20.0, &["food"])];
    let vase = object("vase", "decor", "ornaments");
    let crate_ = object("crate", "decor", "boxes");
    let mut crop = object("tomatoes", "garden", "edibles");
    crop.uses = vec![using("harvest", "harvest", 6.0, &["food"])];
    crop.uses[0].when = Some("ripe".into());
    let motions = BTreeMap::from([
        ("model.sofaBed".to_string(), json!({ "uses": { "doze": "bed_out", "pullOut": "bed_out" } })),
        ("model.fridge".to_string(), json!({ "uses": { "fetch": "door", "stir": "door", "drink": "door" }, "loops": { "feast": "hum", "dance": "spin" } })),
    ]);
    let actions: BTreeSet<String> = ["relax", "nap", "fetch", "eat", "wash", "harvest"].iter().map(|s| s.to_string()).collect();
    (vec![sofa, fridge, sink, table, vase, crate_, crop], motions, actions)
}

#[test]
fn each_uses_flag() {
    let (objects, motions, actions) = catalogue();
    let medians = review::tag_medians(&objects);
    // By animation: the eats 15, 20, 90 (the middle one); the fetches 5 and 2.
    assert_eq!(medians["eat"], 20.0);
    assert_eq!(medians["fetch"], 3.5);
    let flags = |id: &str, decor: bool| {
        let o = objects.iter().find(|o| o.id == id).unwrap();
        review::use_flags(o, motions.get(&o.model), &actions, &medians, decor)
    };
    // Opens with `leaves` but its model has no `motions.open`.
    assert_eq!(flags("sofaBed", false), [Flag::NoOpen { using: "doze".into() }]);
    // A key that is a use's id (`feast`), an anim (`fetch`) or a station's anim (`stir`) is fine; `drink` and
    // `dance` aren't. 90 minutes is over 3 × the median of `eat`, 20.
    assert_eq!(flags("fridge", false), [
        Flag::MotionKey { key: "drink".into() },
        Flag::MotionKey { key: "dance".into() },
        Flag::Duration { using: "feast".into(), minutes: 90.0, like: "eat".into(), median: 20.0 },
    ]);
    // `buff` isn't in ACTIONS; `wash` and `buff` are the only uses like them.
    assert_eq!(flags("sink", false), [Flag::NoAction { using: "polish".into(), anim: "buff".into() }]);
    // No use: flagged unless its kind is decor.
    assert_eq!(flags("crate", false), [Flag::NoUse]);
    assert_eq!(flags("vase", true), []);
    // A lamp is switched: no flag.
    let mut lamp = object("lamp", "lighting", "lamps");
    lamp.lamp = true;
    assert_eq!(review::use_flags(&lamp, None, &actions, &medians, false), []);
    // `when: ripe` is a crop's, not an open state.
    assert_eq!(flags("tomatoes", false), []);
    assert_eq!(Flag::NoUse.column(), review::FLAG_COLUMNS[0]);

    // The whole table: the vase is decor by the reference (its group `ornaments`), the crate isn't.
    let sounds = BTreeMap::from([("wash".to_string(), "tap".to_string())]);
    let r = reference();
    let (text, files) = review::uses(&objects, &motions, &actions, &sounds, Some(&r));
    assert!(text.contains("| kitchen | 2 | 2 | 0 | 1 | 2 | 0 | 1 |"), "{text}");
    assert!(text.contains("| decor | 2 | 1 | 1 | 0 | 0 | 0 | 0 |"), "{text}");
    assert!(text.contains("| **all** | 7 | 4 | 1 | 1 | 2 | 1 | 1 |"), "{text}");
    let living = &files["uses-living.md"];
    assert!(living.contains("| `sofaBed` sofaBed | armchair | relax:relax<br>doze:nap<br>pullOut:fetch | stand<br>stand<br>stand | 30<br>30<br>2 |"), "{living}");
    assert!(living.contains("| uses doze→bed_out, pullOut→bed_out |"), "{living}");
    let (text, _) = review::uses(&objects, &motions, &actions, &sounds, None);
    assert!(text.contains("No docs/review/reference.json yet") && text.contains("| decor | 2 | 2 | 2 |"), "{text}");
}

#[test]
fn the_renderer_tables_are_read_from_its_source() {
    let ts = "const HANDS = { grip: 0.1 };\nconst ACTIONS: Record<string, ActionAnim> = {\n  eat: { stand: ['eat', 'idle'], upper: 'eat' },\n  // A comment.\n  exercise: {\n    stand: (def) =>\n      def.includes('Bag') ? ['punchJab'] : ['jog'],\n    sit: 'sitHands',\n  },\n  'odd-key': { hands: 1 },\n};\nconst AFTER = { not: 1 };\n";
    assert_eq!(review::action_keys(ts).unwrap(), ["eat", "exercise", "odd-key"].iter().map(|s| s.to_string()).collect());
    assert!(review::action_keys("const OTHER = {\n};\n").is_err());
    let world = "const ACTION_SOUNDS: Record<string, LoopKind> = {\n  cook: 'sizzle',\n  // The cooking steps.\n  chop: 'chop',\n  wash: 'tap',\n};\n";
    let sounds = review::action_sounds(world).unwrap();
    assert_eq!(sounds.len(), 3);
    // `soundOf`'s washing: up, or at a basin.
    let thing = Object::default();
    let mut u = using("washUp", "wash", 5.0, &[]);
    assert_eq!(review::sound_of(&sounds, &thing, &u).as_deref(), Some("washUp"));
    u.id = "wash".into();
    assert_eq!(review::sound_of(&sounds, &thing, &u), None);
    u.tags = vec!["handwash".into()];
    assert_eq!(review::sound_of(&sounds, &thing, &u).as_deref(), Some("tap"));
    assert_eq!(review::sound_of(&sounds, &thing, &using("cook", "chop", 5.0, &[])).as_deref(), Some("chop"));
    // By the thing: the use's own sound, else the object's for the animation, `none` for silence.
    let mut laundry = using("laundry", "clean", 60.0, &[]);
    assert_eq!(review::sound_of(&sounds, &thing, &laundry), None);
    laundry.sound = Some("washer".into());
    assert_eq!(review::sound_of(&sounds, &thing, &laundry).as_deref(), Some("washer"));
    let oven = Object { sounds: [("cook".to_string(), "none".to_string())].into(), ..Object::default() };
    let mut bake = using("bake", "cook", 30.0, &[]);
    assert_eq!(review::sound_of(&sounds, &thing, &bake).as_deref(), Some("sizzle"));
    assert_eq!(review::sound_of(&sounds, &oven, &bake), None);
    // A recipe sounds by its steps at the object's stations.
    bake.steps_here = Some(vec![("chop".into(), None), ("cook".into(), None), ("stir".into(), Some("none".into()))]);
    assert_eq!(review::sound_of(&sounds, &oven, &bake).as_deref(), Some("chop"));
}

#[test]
fn the_shipped_content_reads() {
    let paths = Paths::repo();
    let objects = review::objects(Path::new(&paths.content)).unwrap();
    assert_eq!(objects.len(), 541);
    let by = |id: &str| objects.iter().find(|o| o.id == id).unwrap();
    let sofa = by("sofaBed");
    assert_eq!((sofa.page.as_str(), sofa.group.as_str(), sofa.model.as_str(), sofa.slots), ("living", "seating", "model.sofaBed", 2));
    let doze = sofa.uses.iter().find(|u| u.id == "doze").unwrap();
    assert_eq!((doze.anim.as_str(), doze.pose, doze.minutes, doze.leaves.as_deref()), ("nap", "lie", 30.0, Some("open")));
    assert_eq!(doze.tags, ["rest", "nap"]);
    assert!(doze.effects.iter().any(|(n, g)| n == "energy" && (*g - 0.1).abs() < 1e-6), "{:?}", doze.effects);
    assert_eq!(by("gloomy.journalDesk").page, "packs", "a pack's objects are on packs.md");
    assert!(by("fridge").station_anims.contains("fetch"), "{:?}", by("fridge").station_anims);
    assert_eq!(by("toilet").uses[0].end_sound.as_deref(), Some("flush"));
    assert_eq!(by("fountainTown").ambient.as_deref(), Some("fountain"));
    assert_eq!(by("wallOven").sounds.get("stir").map(String::as_str), Some("none"));
    let motions = review::motions(Path::new(&paths.assets)).unwrap();
    assert_eq!(motions["model.sofaBed"]["open"], "bed_out");
    let root = ll_tools::checkglb::repo_root();
    let actions = review::action_keys(&std::fs::read_to_string(format!("{root}/web/src/render/babylon/characters.ts")).unwrap()).unwrap();
    assert!(actions.len() > 40 && actions.contains("serve") && !actions.contains("stand"), "{actions:?}");
    let sounds = review::action_sounds(&std::fs::read_to_string(format!("{root}/web/src/audio/world.ts")).unwrap()).unwrap();
    assert_eq!(sounds.get("shower").map(String::as_str), Some("shower"));
    let (text, files) = review::uses(&objects, &motions, &actions, &sounds, None);
    assert!(text.contains("| **all** | 541 |"), "{text}");
    assert_eq!(files.len(), 15);
}
