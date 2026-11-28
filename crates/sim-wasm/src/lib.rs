//! Browser bindings for `sim-core`. Keep this layer thin: no game logic here.

// SIMD comes from `.cargo/config.toml`'s rustflags, which a `RUSTFLAGS` in the environment replaces
// rather than adds to: such a build stops here instead of shipping a slower wasm without it.
#[cfg(all(target_arch = "wasm32", not(target_feature = "simd128")))]
compile_error!(
    "the wasm needs SIMD: a RUSTFLAGS in the environment replaces .cargo/config.toml's `-C target-feature=+simd128`, so add it there too"
);

use std::sync::atomic::{AtomicBool, Ordering};

use sim_core::save::{OLDEST_SAVE_VERSION, SAVE_VERSION};
use sim_core::{Command, Content, Error, World, mesh, snapshot, view};
use wasm_bindgen::prelude::*;

/// A panic happened: the instance is broken (the build aborts on panic, leaving whatever the
/// call was changing half done), so nothing it holds may be saved.
static PANICKED: AtomicBool = AtomicBool::new(false);

#[wasm_bindgen]
extern "C" {
    /// The worker's `__wasmPanic` (`sim.worker.ts`), which passes the message on to the game.
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __wasmPanic)]
    fn report_panic(message: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

/// Runs as the module starts. A panic aborts (`panic = "abort"`), and the trap only says
/// "unreachable": the hook first passes on what panicked and where.
#[wasm_bindgen(start)]
fn start() {
    std::panic::set_hook(Box::new(|info| {
        PANICKED.store(true, Ordering::Relaxed);
        let message = info.to_string();
        if report_panic(&message).is_err() {
            console_error(&message);
        }
    }));
}

/// Why a save can't be opened. The menu keeps one from a newer version, and one this version
/// failed to load (a later version may open it), and skips them; it removes one that is too old
/// or damaged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveProblem {
    TooNew,
    TooOld,
    Corrupt,
    Failed,
}

/// A failed `Game.fromSave`: `kind` says what's wrong with the save ("tooNew", "tooOld",
/// "corrupt" or "failed"); it's empty when the save isn't to blame (the game's own data didn't load).
#[wasm_bindgen]
pub struct LoadError {
    problem: Option<SaveProblem>,
    message: String,
}

#[wasm_bindgen]
impl LoadError {
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        match self.problem {
            Some(SaveProblem::TooNew) => "tooNew",
            Some(SaveProblem::TooOld) => "tooOld",
            Some(SaveProblem::Corrupt) => "corrupt",
            Some(SaveProblem::Failed) => "failed",
            None => "",
        }
        .to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

/// Loads a save; when that fails, works out whether (and how) the save is to blame.
fn load_save(content_json: &str, save_json: &str) -> Result<World, LoadError> {
    World::from_save_json(content_json, save_json).map_err(|e| LoadError {
        problem: save_problem(content_json, save_json, &e),
        message: e.message,
    })
}

/// What's wrong with a save the simulation refused (`error`): its version (newer than this
/// game, or older than it reads); damaged (its text isn't JSON at all, or the loader's own checks
/// found it so, which it says itself: `Error::is_damaged`); else this version failed to load
/// something in it, which is no reason to remove it. A value of the wrong type (a number saved as
/// `null`, a field this version doesn't know) is such a failure, not damage, though serde words
/// both as "invalid JSON". `None` when the game's own data is what failed.
fn save_problem(content_json: &str, save_json: &str, error: &Error) -> Option<SaveProblem> {
    if Content::from_json(content_json).is_err() {
        return None;
    }
    let json = serde_json::from_str::<serde_json::Value>(save_json).ok();
    let version = json
        .as_ref()
        .and_then(|v| v.get("version").and_then(serde_json::Value::as_u64));
    Some(match version {
        Some(v) if v > u64::from(SAVE_VERSION) => SaveProblem::TooNew,
        Some(v) if v < u64::from(OLDEST_SAVE_VERSION) => SaveProblem::TooOld,
        _ if json.is_none() || error.is_damaged() => SaveProblem::Corrupt,
        _ => SaveProblem::Failed,
    })
}

#[wasm_bindgen]
pub struct Game {
    world: World,
    snapshot: Box<[f32]>,
    /// Which cars aren't drawn, as of the last `snapshot::write_with`: kept so a frame's writes
    /// allocate nothing for it.
    hidden: Vec<bool>,
    /// What the UI was last sent (it gets changes only).
    ui: view::UiSync,
    /// What the renderer was last sent of the structure (`structure_update`: changes only).
    structure: view::StructureSync,
    /// The focus lot (`set_view`), and the area simulated in full detail if the view sent one
    /// (`set_watch`; else the focus lot's).
    view: Option<[i32; 4]>,
    watch: Option<[i32; 4]>,
    /// A fingerprint of the rooms as `rooms_changed` last gave them (None: not yet).
    rooms_sent: Option<u64>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(content_json: &str, lot_json: &str, seed: u32) -> Result<Game, JsError> {
        let world =
            World::from_json(content_json, lot_json, seed).map_err(|e| JsError::new(&e.message))?;
        Ok(Game::with_world(world))
    }

    /// Restores a game from `save()` output; throws a `LoadError` if it can't.
    #[wasm_bindgen(js_name = fromSave)]
    pub fn from_save(content_json: &str, save_json: &str) -> Result<Game, LoadError> {
        load_save(content_json, save_json).map(Game::with_world)
    }

    /// The seed the game started from, for bug reports (0: unknown, a save from before v23).
    #[wasm_bindgen(getter)]
    pub fn seed(&self) -> u32 {
        self.world.seed
    }

    /// Serializes the whole game state (see `sim_core::save`). Refused after a panic: the state
    /// may be half updated, and the last good save must not be overwritten with it.
    pub fn save(&self) -> Result<String, JsError> {
        if PANICKED.load(Ordering::Relaxed) {
            return Err(JsError::new("the game stopped after an error"));
        }
        Ok(self.world.save_json())
    }

    /// Panics on purpose (`?debug`'s `__crash('panic')`): shows what the game does when the
    /// simulation fails.
    #[wasm_bindgen(js_name = debugPanic)]
    pub fn debug_panic(&mut self, why: &str) {
        panic!("{why}");
    }

    /// Runs one real-time step and refreshes the snapshot (at super speed, all of its most
    /// ticks while it stays quiet at home).
    pub fn advance(&mut self) {
        self.world.advance();
        snapshot::write_with(&self.world, &mut self.snapshot, &mut self.hidden);
    }

    /// Starts a real-time step (`World::start_step`): the speed's ticks, or at super speed
    /// none yet (`advance_more`). The ticks it ran. The snapshot waits for `end_step`.
    pub fn start_step(&mut self) -> u32 {
        self.world.start_step()
    }

    /// One more tick of a super-speed step (`World::advance_more`), while the worker has time
    /// for it: whether it ran (not at super speed, no longer quiet at home, or the step's most
    /// run: no).
    pub fn advance_more(&mut self) -> bool {
        self.world.advance_more()
    }

    /// Ends a step begun with `start_step`: refreshes the snapshot.
    pub fn end_step(&mut self) {
        snapshot::write_with(&self.world, &mut self.snapshot, &mut self.hidden);
    }

    /// Pointer into WASM memory; read it as a `Float32Array` of `snapshot_len()` floats.
    /// The buffer never moves for the lifetime of the `Game`, but a `Float32Array` made over it
    /// dies when WASM memory grows: make it again after any other call.
    pub fn snapshot_ptr(&self) -> *const f32 {
        self.snapshot.as_ptr()
    }

    pub fn snapshot_len(&self) -> usize {
        self.snapshot.len()
    }

    /// Applies a JSON-encoded `Command`.
    pub fn command(&mut self, json: &str) -> Result<(), JsError> {
        let cmd = Command::from_json(json).map_err(|e| JsError::new(&e.message))?;
        self.world
            .apply(cmd)
            .map_err(|e| JsError::new(&e.message))?;
        snapshot::write_with(&self.world, &mut self.snapshot, &mut self.hidden);
        Ok(())
    }

    /// The next UI update (JSON): what changed since the last one, or everything after
    /// `resync_ui` (see `view::UiSync`).
    pub fn ui_update(&mut self) -> String {
        self.ui.update_json(&self.world)
    }

    /// The next UI update sends everything.
    pub fn resync_ui(&mut self) {
        self.ui.resync();
    }

    /// The resident whose panel is open (their details and relationships are sent too).
    pub fn inspect(&mut self, sim: Option<u32>) {
        self.ui.inspect(sim);
    }

    /// Social interactions `actor` can start with `target`, with success chances (JSON).
    pub fn social_options(&self, actor: u32, target: u32) -> String {
        view::social_options_json(&self.world, actor as usize, target as usize)
    }

    /// The water mask as it is now (JSON: `{"water": rle, "width", "depth"}`; see
    /// `World::water_mask`): what Build mode's Land tab lays its strokes' water over.
    pub fn water_mask(&self) -> String {
        self.world.water_mask_json()
    }

    /// The household's home as a blueprint (JSON; see `sim_core::blueprint`).
    pub fn blueprint(&self, household: u32) -> Result<String, JsError> {
        self.world
            .blueprint_json(household)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// The whole story log, oldest first (JSON).
    pub fn events(&self) -> String {
        view::events_json(&self.world)
    }

    /// Every career level plus object and build rules (JSON, static per game).
    pub fn catalog(&self) -> String {
        view::catalog_json(&self.world.content)
    }

    /// The next structure update (JSON): what changed since the last one (the first time:
    /// everything); with `lot`, the lot's walls, openings, diagonals, fences and floors too
    /// (never the rooms: `rooms`). See `sim_core::view::StructureSync`.
    pub fn structure_update(&mut self, lot: bool) -> String {
        // (The taxi bays found here once an edit's made, not on every update while paused.)
        sim_core::taxi::warm_kerbs(&mut self.world);
        self.structure.update_json(&self.world, lot)
    }

    /// The focus lot, a tile rectangle: the lot whose walls and floors are drawn in full. Until
    /// the view sends `set_watch`, it is also what's simulated in full detail (lots it touches;
    /// lots nobody watches in less, see `sim_core::lod`).
    pub fn set_view(&mut self, x0: i32, z0: i32, x1: i32, z1: i32) {
        self.view = Some([x0, z0, x1, z1]);
        self.world.set_view(self.watch.or(self.view));
    }

    /// The whole town is looked at: everything in full detail.
    pub fn clear_view(&mut self) {
        self.view = None;
        self.watch = None;
        self.world.set_view(None);
        self.world.set_watch(None);
    }

    /// The camera's stream rectangle (tiles): simulated in full detail, lots it touches too, from
    /// now on instead of the focus lot (`set_view`). It's what the renderer draws: cars out of it
    /// drive through each other (`World::set_watch`).
    pub fn set_watch(&mut self, x0: i32, z0: i32, x1: i32, z1: i32) {
        self.watch = Some([x0, z0, x1, z1]);
        self.world.set_view(self.watch);
        self.world.set_watch(self.watch);
    }

    /// Back to the focus lot as the area simulated in full detail (and every car in sight).
    pub fn clear_watch(&mut self) {
        self.watch = None;
        self.world.set_view(self.view);
        self.world.set_watch(None);
    }

    /// The room id of every tile, row by row (`width × depth` of the structure; 0 outdoors), as a
    /// `Uint16Array`, if they changed since this last gave them (or never did): a big world's are
    /// megabytes, and most edits of the lot (a fence, a floor painted, a door) leave them as they were.
    pub fn rooms_changed(&mut self) -> Option<Vec<u16>> {
        let print = self.world.lot.rooms_print();
        if self.rooms_sent == Some(print) {
            return None;
        }
        self.rooms_sent = Some(print);
        Some(self.world.lot.rooms_row_major())
    }

    /// Food and dishes on tables, counters and floors (JSON; see `sim_core::view::items_json`).
    pub fn items(&self) -> String {
        view::items_json(&self.world)
    }

    /// Bumped whenever food or dishes change (also in the snapshot header, `itemsVersion`).
    pub fn items_version(&self) -> u32 {
        self.world.items.version
    }

    /// The lamps switched off (JSON array of object ids; see `sim_core::lights`).
    pub fn lamps_off(&self) -> String {
        serde_json::to_string(&sim_core::lights::lamps_off(&self.world))
            .unwrap_or_else(|_| "[]".into())
    }

    /// Bumped whenever a lamp is switched on or off (see `lampsOff`).
    pub fn lights_version(&self) -> u32 {
        self.world.lights_version()
    }

    /// The curtains drawn shut (JSON array of object ids; see `sim_core::curtains`).
    pub fn curtains_shut(&self) -> String {
        serde_json::to_string(&sim_core::curtains::shut(&self.world))
            .unwrap_or_else(|_| "[]".into())
    }

    /// Bumped whenever curtains are drawn or opened (see `curtainsShut`).
    pub fn curtains_version(&self) -> u32 {
        self.world.curtains_version()
    }

    /// Changes whenever the lot (walls, floors, fences...) does; see `World::lot_version`.
    pub fn lot_version(&self) -> u32 {
        self.world.lot_version()
    }

    /// Snapshot layout (JSON): float offsets plus the content's animation tags (`actions`).
    pub fn snapshot_layout(&self) -> String {
        snapshot::layout_json(&self.world.content)
    }

    pub fn structure_version(&self) -> u32 {
        self.world.structure_version()
    }

    /// Builds lot geometry inside the tile rectangle `[x0, z0, x1, z1)`.
    /// `kind`: 0 = full walls, 1 = low walls, 2 = floors (anything else: nothing).
    pub fn build_mesh(&self, kind: u8, x0: i32, z0: i32, x1: i32, z1: i32) -> MeshBuffers {
        let lot = &self.world.lot;
        let region = [x0, z0, x1, z1];
        let m = match kind {
            0 => mesh::build_walls_in(lot, mesh::WALL_HEIGHT, region),
            1 => mesh::build_walls_in(lot, mesh::WALL_HEIGHT_LOW, region),
            2 => mesh::build_floors_in(lot, region),
            _ => mesh::MeshData::default(),
        };
        MeshBuffers {
            positions: m.positions,
            normals: m.normals,
            uvs: m.uvs,
            indices: m.indices,
        }
    }
}

impl Game {
    fn with_world(world: World) -> Game {
        let mut game = Game {
            world,
            snapshot: vec![0.0; snapshot::CAPACITY].into_boxed_slice(),
            ui: view::UiSync::new(),
            structure: view::StructureSync::default(),
            view: None,
            watch: None,
            rooms_sent: None,
            hidden: Vec::new(),
        };
        snapshot::write_with(&game.world, &mut game.snapshot, &mut game.hidden);
        game
    }
}

/// Geometry handed to JS once per structure change. Each `take_*` moves its array out (once).
#[wasm_bindgen]
pub struct MeshBuffers {
    positions: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl MeshBuffers {
    pub fn take_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.positions)
    }

    pub fn take_normals(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.normals)
    }

    pub fn take_uvs(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.uvs)
    }

    pub fn take_indices(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped content, merged like the web loader does (base.json plus its includes).
    fn content() -> String {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content");
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).unwrap();
        let base = read("base.json");
        let v: serde_json::Value = serde_json::from_str(&base).unwrap();
        let parts: Vec<String> = v["include"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| read(n.as_str().unwrap()))
            .collect();
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
        sim_core::merge_content(&base, &parts).unwrap()
    }

    /// Saves written by released builds: 0.1.0 (version 17) and the live site's d7c506e (19). The
    /// game no longer opens them.
    fn old_save(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/saves")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn problem(content: &str, save: &str) -> Option<SaveProblem> {
        load_save(content, save)
            .err()
            .expect("the save is refused")
            .problem
    }

    /// A save as this version writes it: `sim_core`'s town of day 5 (its `tests/saves`), loaded and
    /// saved again.
    fn current_save(content: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../sim-core/tests/saves/live_day5.json");
        World::from_save_json(content, &std::fs::read_to_string(path).unwrap())
            .unwrap_or_else(|e| panic!("live_day5.json: {}", e.message))
            .save_json()
    }

    #[test]
    fn old_releases_saves_are_too_old() {
        let content = content();
        for name in ["v0.1.0-day5.json", "d7c506e-day5.json"] {
            let err = load_save(&content, &old_save(name)).err().expect("refused");
            assert_eq!(err.kind(), "tooOld", "{name}: {}", err.message);
        }
        let world = load_save(&content, &current_save(&content))
            .unwrap_or_else(|e| panic!("{}", e.message));
        assert!(world.sims.len() > 10, "{} residents", world.sims.len());
    }

    #[test]
    fn unloadable_saves_say_why() {
        let content = content();
        let good = current_save(&content);
        let now = format!("\"version\":{SAVE_VERSION}");
        assert!(good.starts_with(&format!("{{{now},")));
        let with_version = |v: &str| good.replacen(&now, &format!("\"version\":{v}"), 1);
        assert_eq!(
            problem(&content, &with_version("999")),
            Some(SaveProblem::TooNew)
        );
        let next = (SAVE_VERSION + 1).to_string();
        assert_eq!(
            problem(&content, &with_version(&next)),
            Some(SaveProblem::TooNew)
        );
        assert_eq!(
            problem(&content, &with_version("0")),
            Some(SaveProblem::TooOld)
        );
        assert_eq!(
            problem(&content, "garbage, not a save"),
            Some(SaveProblem::Corrupt)
        );
        assert_eq!(problem(&content, ""), Some(SaveProblem::Corrupt));
        assert_eq!(problem(&content, &good[..5000]), Some(SaveProblem::Corrupt));
        // Text that reads as JSON but not as this version's save is kept: a type this version
        // can't read is no proof the save is damaged.
        assert_eq!(
            problem(&content, &format!("{{{now}}}")),
            Some(SaveProblem::Failed)
        );
        assert_eq!(
            problem(&content, &with_version("-3")),
            Some(SaveProblem::Failed)
        );
        assert_eq!(
            problem(&content, &with_version("\"17\"")),
            Some(SaveProblem::Failed)
        );
        let mut odd: serde_json::Value = serde_json::from_str(&good).unwrap();
        odd["sims"][0]["traits"] = serde_json::json!(null);
        assert_eq!(
            problem(&content, &odd.to_string()),
            Some(SaveProblem::Failed)
        );
        // A save that is whole but has a value of the wrong type isn't damaged: kept (a later
        // version may read it). (A trait this version doesn't know is dropped, not an error.)
        let mut odd: serde_json::Value = serde_json::from_str(&good).unwrap();
        odd["tick"] = serde_json::json!("left");
        assert_eq!(
            problem(&content, &odd.to_string()),
            Some(SaveProblem::Failed)
        );
        assert_eq!(
            load_save(&content, &odd.to_string()).err().unwrap().kind(),
            "failed"
        );
        // The loader's own checks say damaged: removed.
        let mut odd: serde_json::Value = serde_json::from_str(&good).unwrap();
        odd["households"][0]["plot"] = serde_json::json!(999);
        assert_eq!(
            problem(&content, &odd.to_string()),
            Some(SaveProblem::Corrupt)
        );
        // As do its "saved …" checks.
        let mut odd: serde_json::Value = serde_json::from_str(&good).unwrap();
        odd["lot"]["width"] = serde_json::json!(0);
        let err = load_save(&content, &odd.to_string()).err().unwrap();
        assert!(err.message.starts_with("saved "), "{}", err.message);
        assert_eq!(err.problem, Some(SaveProblem::Corrupt));
        // The game's own data failing isn't the save's fault (it mustn't be removed for that).
        assert_eq!(problem("{", &good), None);
        assert_eq!(problem("{", &with_version("999")), None);
        let err = load_save(&content, &with_version("999")).err().unwrap();
        assert_eq!(err.kind(), "tooNew");
        assert!(err.message().contains("999"), "{}", err.message());
    }

    /// The rooms go out the first time and after a wall changes them, not when nothing did.
    #[test]
    fn rooms_go_out_only_when_they_change() {
        let lot = r#"{"width":12,"depth":12,"walls":[[2,2,8,2],[8,2,8,8],[2,8,8,8],[2,2,2,8]]}"#;
        let mut g = Game::with_world(World::from_json(&content(), lot, 1).unwrap());
        let first = g.rooms_changed().expect("the first time");
        assert_eq!(first.len(), g.world.lot.width * g.world.lot.depth);
        assert!(
            first.iter().any(|&r| r != sim_core::lot::OUTDOORS),
            "a room"
        );
        assert_eq!(g.rooms_changed(), None, "nothing changed");
        g.world.lot.set_h(4, 2, sim_core::lot::Edge::Open);
        g.world.lot.compute_rooms();
        let open = g.rooms_changed().expect("a wall taken down");
        assert!(open.iter().all(|&r| r == sim_core::lot::OUTDOORS));
        assert_eq!(g.rooms_changed(), None);
        g.world.lot.set_h(4, 2, sim_core::lot::Edge::Wall);
        g.world.lot.compute_rooms();
        assert_eq!(g.rooms_changed(), Some(first), "built again");
    }
}
