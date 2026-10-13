//! Browser bindings for `sim-core`. Keep this layer thin: no game logic here.

use sim_core::save::SAVE_VERSION;
use sim_core::{Command, Content, World, mesh, snapshot, view};
use wasm_bindgen::prelude::*;

/// Oldest save version `sim_core::save` still reads (it migrates everything from 1 on).
const OLDEST_SAVE_VERSION: u64 = 1;

/// Why a save can't be opened. The menu keeps one from a newer version (and skips it) and
/// removes one that is too old or damaged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveProblem {
    TooNew,
    TooOld,
    Corrupt,
}

/// A failed `Game.fromSave`: `kind` says what's wrong with the save ("tooNew", "tooOld" or
/// "corrupt"); it's empty when the save isn't to blame (the game's own data didn't load).
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
        problem: save_problem(content_json, save_json),
        message: e.0,
    })
}

/// What's wrong with a save the simulation refused: its version (newer than this game, or
/// older than it reads), else it's damaged. `None` when the game's own data is what failed.
fn save_problem(content_json: &str, save_json: &str) -> Option<SaveProblem> {
    if Content::from_json(content_json).is_err() {
        return None;
    }
    let version = serde_json::from_str::<serde_json::Value>(save_json)
        .ok()
        .and_then(|v| v.get("version").and_then(serde_json::Value::as_u64));
    Some(match version {
        Some(v) if v > u64::from(SAVE_VERSION) => SaveProblem::TooNew,
        Some(v) if v < OLDEST_SAVE_VERSION => SaveProblem::TooOld,
        _ => SaveProblem::Corrupt,
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

    /// Serializes the whole game state (see `sim_core::save`).
    pub fn save(&self) -> String {
        self.world.save_json()
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

    /// Applies a JSON-encoded `Command`.
    pub fn command(&mut self, json: &str) -> Result<(), JsError> {
        let cmd = Command::from_json(json).map_err(|e| JsError::new(&e.0))?;
        self.world.apply(cmd).map_err(|e| JsError::new(&e.0))?;
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
        // The game's own data failing isn't the save's fault (it mustn't be removed for that).
        assert_eq!(problem("{", &good), None);
        assert_eq!(problem("{", &with_version("999")), None);
        let err = load_save(&content, &with_version("999")).err().unwrap();
        assert_eq!(err.kind(), "tooNew");
        assert!(err.message().contains("999"), "{}", err.message());
    }
}
