//! A family: the models one module builds. Each model is a build function and the facts the game needs;
//! `build_model` builds, checks and writes one in memory (its .glb, its far version's, its pack entry), and the
//! runner (`cargo art`) puts them where they go.

use crate::json::Json;
use crate::materials::Palette;
use crate::parts::{Tolerance, check_size};
use crate::scene::{self, Obj, meshes};
use crate::{far, glb, motion, perch, shade, uv};

/// Builds a model in the empty scene (Blender axes, metres), standing on the floor and centred on its footprint;
/// it returns its parts (none: every mesh in the scene).
pub type Build = fn() -> Vec<Obj>;

/// A model's far version, models/<family>/<id>.far.glb, drawn where it's only seen from afar (the town overview,
/// the far woods).
#[derive(Clone, Copy, Debug)]
pub enum Far {
    /// The model's own build again at low detail (`far::low`, `far::is_low`).
    Low,
    /// A build of its own.
    Own(Build),
    /// The model at rest, joined and decimated to this share of its triangles (`far=0.25`). The kernel has no
    /// decimator yet: a model that asks for one fails to build, and says so.
    Ratio(f64),
}

/// One model: models/<family>/<id>.glb.
#[derive(Clone, Debug)]
pub struct Model {
    pub id: &'static str,
    pub build: Build,
    /// (width, height, depth) in game metres, checked within `tolerance`.
    pub size: [f64; 3],
    /// The manifest key it draws for (default model.<family>.<id>).
    pub key: Option<&'static str>,
    pub tolerance: Tolerance,
    /// The pack entry's `motions` for its clips (docs/design/objects.md, "Moving parts").
    pub motions: Option<Json>,
    /// The parts the player may colour (the pack's `paint`): (material, label), the main part first.
    pub paint: Vec<(&'static str, &'static str)>,
    /// Primitive parts drawn with the file (a mirror a file can't reflect yet).
    pub parts: Option<Json>,
    /// (game x, depth) the model's bounds are centred on at rest, for one that stands off the middle of its
    /// footprint; `settle` centres it there. Written into centres.json, for the gate only.
    pub centre: [f64; 2],
    /// False for a thing on a wall: built at its real height, not stood on the floor.
    pub floor: bool,
    pub far: Option<Far>,
    /// How far the model may reach below the floor, for the few the game sinks.
    pub sink: f64,
    /// A model the game draws by code, not for a content object or item: its key needn't be one the content names.
    pub code: bool,
    /// The entry's `bird` for a bird the game flies about by code.
    pub bird: Option<Json>,
    /// What its perches are on, the entry's `habitat`: 'tree', 'conifer', 'bush' or 'hedge'.
    pub habitat: Option<&'static str>,
    /// The most draw calls the model may take, where it isn't four.
    pub draws: Option<i64>,
    /// Whether the floor it stands on shades the foot of it; default: on the floor and not a bird.
    pub ground: Option<bool>,
    /// The gate's strict checks this model is right to fail: (check, why).
    pub accept: Vec<(&'static str, &'static str)>,
    /// Its own palette, for a family whose models' colours differ in finish (each garden thing's `body`: a stone
    /// pond, a wooden planter, an iron post): its pack entry's `materials` come from it, not from the family's.
    pub palette: Option<Palette>,
}

impl Model {
    pub fn new(id: &'static str, build: Build, size: (f64, f64, f64)) -> Model {
        Model {
            id,
            build,
            size: [size.0, size.1, size.2],
            key: None,
            tolerance: Tolerance([0.02; 3]),
            motions: None,
            paint: Vec::new(),
            parts: None,
            centre: [0.0, 0.0],
            floor: true,
            far: None,
            sink: 0.0,
            code: false,
            bird: None,
            habitat: None,
            draws: None,
            ground: None,
            accept: Vec::new(),
            palette: None,
        }
    }

    pub fn key(mut self, key: &'static str) -> Model {
        self.key = Some(key);
        self
    }
    pub fn tolerance(mut self, t: impl Into<Tolerance>) -> Model {
        self.tolerance = t.into();
        self
    }
    pub fn motions(mut self, motions: Json) -> Model {
        self.motions = Some(motions);
        self
    }
    pub fn paint(mut self, paint: &[(&'static str, &'static str)]) -> Model {
        self.paint = paint.to_vec();
        self
    }
    pub fn parts(mut self, parts: Json) -> Model {
        self.parts = Some(parts);
        self
    }
    pub fn centre(mut self, x: f64, depth: f64) -> Model {
        self.centre = [x, depth];
        self
    }
    pub fn on_wall(mut self) -> Model {
        self.floor = false;
        self
    }
    /// With a far version: the same build at low detail.
    pub fn far_low(mut self) -> Model {
        self.far = Some(Far::Low);
        self
    }
    /// With a far version built by `build`.
    pub fn far(mut self, build: Build) -> Model {
        self.far = Some(Far::Own(build));
        self
    }
    /// With a far version decimated to `ratio` of the model's triangles (`far=0.25`).
    pub fn far_ratio(mut self, ratio: f64) -> Model {
        self.far = Some(Far::Ratio(ratio));
        self
    }
    pub fn sink(mut self, sink: f64) -> Model {
        self.sink = sink;
        self
    }
    pub fn code(mut self) -> Model {
        self.code = true;
        self
    }
    pub fn bird(mut self, bird: Json) -> Model {
        self.bird = Some(bird);
        self
    }
    pub fn habitat(mut self, habitat: &'static str) -> Model {
        self.habitat = Some(habitat);
        self
    }
    pub fn draws(mut self, draws: i64) -> Model {
        self.draws = Some(draws);
        self
    }
    pub fn ground(mut self, ground: bool) -> Model {
        self.ground = Some(ground);
        self
    }
    pub fn accept(mut self, check: &'static str, why: &'static str) -> Model {
        assert!(
            !why.trim().is_empty(),
            "{}: each `accept` gives its reason",
            self.id
        );
        self.accept.push((check, why));
        self
    }

    /// With a palette of its own (the scripts' `_own(id)` / `_palette(body=...)`): the pack's `materials` for
    /// this model are read from it.
    pub fn palette(mut self, palette: Palette) -> Model {
        self.palette = Some(palette);
        self
    }

    /// Whether the floor shades the foot of it.
    pub fn on_ground(&self) -> bool {
        self.ground.unwrap_or(self.floor && self.bird.is_none())
    }
}

/// The models' folder (models/<name>/), the default key prefix (model.<name>.) and their pack.
#[derive(Clone, Debug)]
pub struct Family {
    pub name: &'static str,
    /// The pack's `materials` come from it.
    pub palette: Palette,
    pub models: Vec<Model>,
    /// The pack's $comment: what the family is and which module builds it.
    pub comment: &'static str,
    /// The pack's folder (packs/<pack>/manifest.json).
    pub pack: &'static str,
    /// Its files in the repo are the ones built here: cargo art builds it in Rust (`switched`).
    pub switched: bool,
}

impl Family {
    /// The palette `model`'s materials are named in: its own, or the family's.
    pub fn palette_of<'a>(&'a self, model: &'a Model) -> &'a Palette {
        model.palette.as_ref().unwrap_or(&self.palette)
    }

    pub fn new(
        name: &'static str,
        palette: Palette,
        models: Vec<Model>,
        comment: &'static str,
    ) -> Family {
        let mut ids: Vec<&str> = models.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        assert!(
            ids.windows(2).all(|w| w[0] != w[1]),
            "{name}: two models with one id"
        );
        Family {
            name,
            palette,
            models,
            comment,
            pack: name,
            switched: false,
        }
    }

    /// Its files in the repo are the ones built here: cargo art builds it in Rust, whatever is asked (a family
    /// ported and not yet switched only with --out or --check, and its script in Blender otherwise).
    pub fn switched(mut self) -> Family {
        self.switched = true;
        self
    }

    pub fn pack(mut self, pack: &'static str) -> Family {
        self.pack = pack;
        self
    }

    pub fn key(&self, model: &Model) -> String {
        model
            .key
            .map(String::from)
            .unwrap_or_else(|| format!("model.{}.{}", self.name, model.id))
    }
}

/// What building one model gives: its file, its far version's, and what the pack is told of it.
#[derive(Clone, Debug)]
pub struct Output {
    pub glb: Vec<u8>,
    pub far: Option<Vec<u8>>,
    /// The pack entry's fields (url, materials, motions ...).
    pub entry: Json,
    /// Its clips' names, sorted.
    pub clips: Vec<String>,
}

fn stats(name: &str) {
    scene::log(format!(
        "[gamekit] {name}: {} triangles in {} meshes",
        glb::triangles(),
        meshes().len()
    ));
}

/// Builds one model in an empty scene and checks it; its parts.
pub fn make(model: &Model) -> Result<Vec<Obj>, String> {
    scene::reset();
    perch::clear();
    scene::with(|s| s.centre = model.centre);
    let built = (model.build)();
    scene::with(|s| s.centre = [0.0, 0.0]);
    let objs = if built.is_empty() { meshes() } else { built };
    // (a part never joined is given its UVs and its bevel here, as Blender's export applied its modifiers)
    crate::bevel::apply_all(&objs);
    let (_, line) = check_size(
        model.size,
        model.tolerance,
        &objs,
        model.id,
        model.centre,
        model.floor,
    )?;
    scene::log(line);
    stats(model.id);
    Ok(objs)
}

/// The far version of `model`, in the scene: its parts.
pub fn make_far(model: &Model, far_build: Far) -> Result<Vec<Obj>, String> {
    scene::reset();
    let built = match far_build {
        Far::Low => far::far_of(model.build),
        Far::Own(build) => build(),
        Far::Ratio(ratio) => {
            return Err(format!(
                "{}: its far version is the model decimated to {ratio} of its triangles (far={ratio}), and the \
                 kernel has no decimator yet (gamekit/far.py `decimated`: Blender's DECIMATE modifier): give it a \
                 build of its own (`far`, `far_low`) or wait for the decimator",
                model.id
            ));
        }
    };
    let objs = if built.is_empty() { meshes() } else { built };
    crate::bevel::apply_all(&objs);
    let name = format!("{}.far", model.id);
    let tolerance = Tolerance(model.tolerance.0.map(|t| t.max(0.05)));
    let (_, line) = check_size(
        model.size,
        tolerance,
        &objs,
        &name,
        model.centre,
        model.floor,
    )?;
    scene::log(line);
    stats(&name);
    Ok(objs)
}

/// The parts shaded and written: the .glb's bytes.
fn export(objs: &[Obj], ground: bool) -> Vec<u8> {
    // The textured finishes' UVs, made when the parts were joined; a part never joined gets its own.
    let texcoords = uv::ensure(objs);
    shade::bake_ao(objs, None, ground);
    let finishes = scene::with(|s| s.finishes.clone());
    glb::write(objs, texcoords, &finishes)
}

/// Builds, checks and writes `model` of `family` (in memory): its file, its far version's and its pack entry. The
/// game's finishes must be set (`scene::set_finishes`). What the build says is in the scene's log.
pub fn build_model(family: &Family, model: &Model) -> Result<Output, String> {
    let key = family.key(model);
    let objs = make(model)?;
    let perches = perch::game(); // before a far version is built
    let mut clips: Vec<String> = motion::clips(&objs)
        .into_iter()
        .map(|(_, name, _)| name)
        .collect();
    clips.sort();
    clips.dedup();
    let glb = export(&objs, model.on_ground());
    let used = glb::materials_used(&objs);
    let url = format!("../../models/{}/{}.glb", family.name, model.id);
    let mut entry = Json::Obj(vec![
        ("url".into(), Json::Str(url)),
        (
            "materials".into(),
            family.palette_of(model).finishes(&used)?,
        ),
    ]);
    if let Some(motions) = &model.motions {
        entry.set("motions", motions.clone());
    }
    if !model.paint.is_empty() {
        let mut slots = Vec::new();
        for (material, label) in &model.paint {
            if !used.contains(material) {
                return Err(format!(
                    "paint: the model has no material {material:?} (it has {used:?})"
                ));
            }
            slots.push(Json::Obj(vec![
                ("material".into(), Json::Str(material.to_string())),
                ("label".into(), Json::Str(label.to_string())),
            ]));
        }
        entry.set("paint", Json::Arr(slots));
    }
    if let Some(parts) = &model.parts {
        entry.set("parts", parts.clone());
    }
    if model.sink != 0.0 {
        entry.set("sink", Json::Num(model.sink));
    }
    if let Some(bird) = &model.bird {
        entry.set("bird", bird.clone());
    }
    if !perches.is_empty() {
        entry.set(
            "perches",
            Json::Arr(
                perches
                    .iter()
                    .map(|p| Json::Arr(p.iter().map(|&c| Json::Num(c)).collect()))
                    .collect(),
            ),
        );
        scene::log(format!("[gamekit] {}: {} perches", model.id, perches.len()));
    }
    if let Some(habitat) = model.habitat {
        if perches.is_empty() {
            return Err(format!(
                "{}: `habitat` says what its perches are on, and it has none",
                model.id
            ));
        }
        entry.set("habitat", Json::Str(habitat.to_string()));
    }
    if let Some(draws) = model.draws {
        entry.set("draws", Json::Int(draws));
    }
    if !model.accept.is_empty() {
        entry.set(
            "accept",
            Json::Obj(
                model
                    .accept
                    .iter()
                    .map(|(c, why)| (c.to_string(), Json::Str(why.to_string())))
                    .collect(),
            ),
        );
    }
    let mut far = None;
    if let Some(far_build) = model.far {
        let far_objs = make_far(model, far_build)?;
        far = Some(export(&far_objs, model.on_ground()));
        let far_url = format!("../../models/{}/{}.far.glb", family.name, model.id);
        entry.set("far", Json::Str(far_url));
        // The far version's own materials too (a tree's lumps on `foliage`, its cards on `leafcards`).
        let mut all = used.clone();
        all.extend(glb::materials_used(&far_objs));
        entry.set("materials", family.palette_of(model).finishes(&all)?);
        scene::log(format!(
            "[gamekit] built its far version -> models/{}/{}.far.glb",
            family.name, model.id
        ));
    }
    scene::log(format!(
        "[gamekit] built {key} -> models/{}/{}.glb",
        family.name, model.id
    ));
    Ok(Output {
        glb,
        far,
        entry,
        clips,
    })
}
