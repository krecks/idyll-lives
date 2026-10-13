//! Browser bindings for `sim-core`. Keep this layer thin: no game logic here.

// SIMD comes from `.cargo/config.toml`'s rustflags, which a `RUSTFLAGS` in the environment replaces
// rather than adds to: such a build stops here instead of shipping a slower wasm without it.
#[cfg(all(target_arch = "wasm32", not(target_feature = "simd128")))]
compile_error!("the wasm needs SIMD: a RUSTFLAGS in the environment replaces .cargo/config.toml's `-C target-feature=+simd128`, so add it there too");

use std::sync::atomic::{AtomicBool, Ordering};

use sim_core::save::SAVE_VERSION;
use sim_core::{Command, Content, World, mesh, snapshot, view};
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
    #[wasm_bindgen(js_namespace = console, js_name = debug)]
    fn console_debug(message: &str);
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

/// Oldest save version `sim_core::save` still reads (it migrates everything from 1 on).
const OLDEST_SAVE_VERSION: u64 = 1;

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
        problem: save_problem(content_json, save_json, &e.0),
        message: e.0,
    })
}

/// What's wrong with a save the simulation refused (`error`): its version (newer than this
/// game, or older than it reads); damaged (it doesn't read as a save file, "invalid JSON …", or
/// the loader's own checks found it so: "corrupt …", "saved lot …"); else this version failed to
/// load something in it, which is no reason to remove it. `None` when the game's own data is what
/// failed. (Errors are told apart by their words: parsing the save again here would put a second
/// copy of its whole reader into the wasm.)
fn save_problem(content_json: &str, save_json: &str, error: &str) -> Option<SaveProblem> {
    if Content::from_json(content_json).is_err() {
        return None;
    }
    let version = serde_json::from_str::<serde_json::Value>(save_json)
        .ok()
        .and_then(|v| v.get("version").and_then(serde_json::Value::as_u64));
    let damaged = ["invalid JSON", "corrupt", "saved "].iter().any(|start| error.starts_with(start));
    Some(match version {
        Some(v) if v > u64::from(SAVE_VERSION) => SaveProblem::TooNew,
        Some(v) if v < OLDEST_SAVE_VERSION => SaveProblem::TooOld,
        _ if damaged => SaveProblem::Corrupt,
        _ => SaveProblem::Failed,
    })
}

#[wasm_bindgen]
pub struct Game {
    world: World,
    snapshot: Box<[f32]>,
    /// What the UI was last sent (it gets changes only).
    ui: view::UiSync,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(content_json: &str, lot_json: &str, seed: u32) -> Result<Game, JsError> {
        let world =
            World::from_json(content_json, lot_json, seed).map_err(|e| JsError::new(&e.0))?;
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

    /// Runs one real-time step and refreshes the snapshot.
    pub fn advance(&mut self) {
        self.world.advance();
        snapshot::write(&self.world, &mut self.snapshot);
    }

    /// Pointer into WASM memory; read it as a `Float32Array` of `snapshot_len()` floats.
    /// The buffer never moves for the lifetime of the `Game`.
    pub fn snapshot_ptr(&self) -> *const f32 {
        self.snapshot.as_ptr()
    }

    pub fn snapshot_len(&self) -> usize {
        self.snapshot.len()
    }

    /// Applies a JSON-encoded `Command`. One that names an object by id with what the view
    /// showed there (`seen`) is skipped when that id names another object by now.
    pub fn command(&mut self, json: &str) -> Result<(), JsError> {
        let (cmd, seen) = Command::from_json_seen(json).map_err(|e| JsError::new(&e.0))?;
        if !self.world.apply_seen(cmd, seen.as_ref()).map_err(|e| JsError::new(&e.0))? {
            console_debug(&format!("skipped (the object's id changed): {json}"));
        }
        snapshot::write(&self.world, &mut self.snapshot);
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

    /// The household's home as a blueprint (JSON; see `sim_core::blueprint`).
    pub fn blueprint(&self, household: u32) -> Result<String, JsError> {
        self.world.blueprint_json(household).map_err(|e| JsError::new(&e.to_string()))
    }

    /// The whole story log, oldest first (JSON).
    pub fn events(&self) -> String {
        view::events_json(&self.world)
    }

    /// Every career level plus object and build rules (JSON, static per game).
    pub fn catalog(&self) -> String {
        view::catalog_json(&self.world.content)
    }

    /// The world structure; with `lot` false, without the lot parts (see `lotVersion`).
    pub fn structure(&self, lot: bool) -> String {
        view::structure_json_with(&self.world, lot)
    }

    /// The part of the town being looked at, a tile rectangle: lots it touches are simulated
    /// in full detail, lots nobody watches in less (see `sim_core::lod`).
    pub fn set_view(&mut self, x0: i32, z0: i32, x1: i32, z1: i32) {
        self.world.set_view(Some([x0, z0, x1, z1]));
    }

    /// The whole town is looked at: everything in full detail.
    pub fn clear_view(&mut self) {
        self.world.set_view(None);
    }

    /// Food and dishes on tables, counters and floors (JSON; see `sim_core::view::items_json`).
    pub fn items(&self) -> String {
        view::items_json(&self.world)
    }

    /// Bumped whenever food or dishes change (also in the snapshot header, `itemsVersion`).
    pub fn items_version(&self) -> u32 {
        self.world.items.version
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
    /// `kind`: 0 = full walls, 1 = low walls, 2 = floors.
    pub fn build_mesh(&self, kind: u8, x0: i32, z0: i32, x1: i32, z1: i32) -> MeshBuffers {
        let lot = &self.world.lot;
        let region = [x0, z0, x1, z1];
        let m = match kind {
            0 => mesh::build_walls_in(lot, mesh::WALL_HEIGHT, region),
            1 => mesh::build_walls_in(lot, mesh::WALL_HEIGHT_LOW, region),
            _ => mesh::build_floors_in(lot, region),
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
        };
        snapshot::write(&game.world, &mut game.snapshot);
        game
    }
}

/// Geometry handed to JS once per structure change. Getters copy into typed arrays.
#[wasm_bindgen]
pub struct MeshBuffers {
    positions: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl MeshBuffers {
    #[wasm_bindgen(getter)]
    pub fn positions(&self) -> Vec<f32> {
        self.positions.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn normals(&self) -> Vec<f32> {
        self.normals.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn uvs(&self) -> Vec<f32> {
        self.uvs.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> Vec<u32> {
        self.indices.clone()
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
        let parts: Vec<String> =
            v["include"].as_array().unwrap().iter().map(|n| read(n.as_str().unwrap())).collect();
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
        sim_core::merge_content(&base, &parts).unwrap()
    }

    /// Saves written by released builds: 0.1.0 (version 17) and the live site's d7c506e (19).
    fn old_save(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves").join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn problem(content: &str, save: &str) -> Option<SaveProblem> {
        load_save(content, save).err().expect("the save is refused").problem
    }

    #[test]
    fn old_releases_saves_load() {
        let content = content();
        for name in ["v0.1.0-day5.json", "d7c506e-day5.json"] {
            let world = load_save(&content, &old_save(name))
                .unwrap_or_else(|e| panic!("{name}: {}", e.message));
            assert!(world.sims.len() > 10, "{name}: {} residents", world.sims.len());
        }
    }

    #[test]
    fn unloadable_saves_say_why() {
        let content = content();
        let good = old_save("v0.1.0-day5.json");
        assert!(good.starts_with("{\"version\":17,"));
        let with_version =
            |v: &str| good.replacen("\"version\":17", &format!("\"version\":{v}"), 1);
        assert_eq!(problem(&content, &with_version("999")), Some(SaveProblem::TooNew));
        let next = (SAVE_VERSION + 1).to_string();
        assert_eq!(problem(&content, &with_version(&next)), Some(SaveProblem::TooNew));
        assert_eq!(problem(&content, &with_version("0")), Some(SaveProblem::TooOld));
        assert_eq!(problem(&content, "garbage, not a save"), Some(SaveProblem::Corrupt));
        assert_eq!(problem(&content, ""), Some(SaveProblem::Corrupt));
        assert_eq!(problem(&content, &good[..5000]), Some(SaveProblem::Corrupt));
        assert_eq!(problem(&content, "{\"version\":17}"), Some(SaveProblem::Corrupt));
        assert_eq!(problem(&content, &with_version("-3")), Some(SaveProblem::Corrupt));
        assert_eq!(problem(&content, &with_version("\"17\"")), Some(SaveProblem::Corrupt));
        // A whole save this version fails to load (here a trait it doesn't know) isn't damaged: kept.
        let mut odd: serde_json::Value = serde_json::from_str(&good).unwrap();
        odd["sims"][0]["traits"] = serde_json::json!(["noSuchTrait"]);
        assert_eq!(problem(&content, &odd.to_string()), Some(SaveProblem::Failed));
        assert_eq!(load_save(&content, &odd.to_string()).err().unwrap().kind(), "failed");
        // The loader's own checks say damaged: removed.
        let mut odd: serde_json::Value = serde_json::from_str(&good).unwrap();
        odd["households"][0]["plot"] = serde_json::json!(999);
        assert_eq!(problem(&content, &odd.to_string()), Some(SaveProblem::Corrupt));
        // The game's own data failing isn't the save's fault (it mustn't be removed for that).
        assert_eq!(problem("{", &good), None);
        assert_eq!(problem("{", &with_version("999")), None);
        let err = load_save(&content, &with_version("999")).err().unwrap();
        assert_eq!(err.kind(), "tooNew");
        assert!(err.message().contains("999"), "{}", err.message());
    }
}
