//! The town's view: trips and rides, structures and junctions, cars, walls, objects and plots.

use super::*;

/// On their way somewhere (`SimView.trip`): to which plot, and about when they're there.
#[derive(Serialize)]
pub(crate) struct TripView {
    pub to: u32,
    /// Minute of day, in steps of `TRIP_STEP` (it's an estimate: and the UI isn't sent a new one
    /// every few ticks as they go). None: not worked out (`trip_view`'s `timed`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<u32>,
}

/// Minutes the arrival time of a trip is rounded to.
const TRIP_STEP: f32 = 5.0;

/// Where resident `s` is on their way to: a visit or home (until they're on that plot), or home
/// from work in the household's car (back when it is, `riders`: who is in one on the way home,
/// sorted); with the time they're there, from `Travel::eta` (by car where they're bound
/// for it). Read only: nothing here changes the game.
///
/// `timed` false: a visit or the way home without the time, which takes a search along the roads
/// (the UI updates ten times a second: that's for the residents whose details are shown; the
/// town map says where the others are off to). The car home from work has its time either way.
pub(crate) fn trip_view(world: &World, s: &Sim, riders: &[u32], timed: bool) -> Option<TripView> {
    let home = world.households.get(s.household as usize)?.plot;
    let step = |tick: u64| {
        let m = (clock::minute_of_day(tick) / TRIP_STEP).round() * TRIP_STEP;
        (m as u32) % (24 * 60)
    };
    if let Some(until) = s.away_until {
        let riding = riders.binary_search(&s.id).is_ok();
        return riding.then_some(home).flatten().map(|to| TripView {
            to,
            at: Some(step(until)),
        });
    }
    let to = match s.current()?.task.kind {
        TaskKind::Visit { plot } => plot,
        TaskKind::GoHome => home?,
        _ => return None,
    };
    let (x, z) = s.tile();
    if world.plot_at(x, z) == Some(to) {
        return None;
    }
    let door = world.plots.get(to as usize)?.arrival_point();
    if !timed {
        return Some(TripView { to, at: None });
    }
    // (From over the ground, upstairs too.)
    let from = world.stairs.ground_pos(s.pos);
    let car = s.bound_for_car.is_some();
    let mode = if car {
        crate::travel::Mode::Car
    } else {
        crate::travel::Mode::Foot
    };
    // (On a bike: as fast as it goes, `ride.rs`.)
    let walk =
        s.walk_per_minute(&world.content) * crate::ride::speed(&world.content, s, &world.objects);
    let minutes = crate::world::trip_minutes(world.travel.eta(from, door, mode, walk), car);
    let at = world.tick + (minutes.max(0.0) * crate::TICKS_PER_MINUTE as f32) as u64;
    Some(TripView {
        to,
        at: Some(step(at)),
    })
}

/// On a bike or an e-scooter of theirs (`SimView.ride`, `ride.rs`): what it is, and whether they're
/// off to work, to school, out (a visit or an outing) or home on it.
#[derive(Serialize)]
pub(crate) struct RideView {
    pub kind: crate::content::VehicleKind,
    pub to: &'static str,
}

/// The work taxi (`SimView.taxi`, `taxi::status`): `state` `wait` (at their kerb), `work` (in it,
/// `at` the minute of the day they're there) or `home` (on their way home in it, or waiting for it
/// at work: `at` about when they're back).
#[derive(Serialize)]
pub(crate) struct TaxiView {
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<f32>,
}

pub(crate) fn taxi_view(world: &World, s: &Sim) -> Option<TaxiView> {
    let (state, at) = crate::taxi::status(world, s)?;
    Some(TaxiView {
        state,
        at: at.map(|t| (clock::minute_of_day(t) / 5.0).round() * 5.0),
    })
}

/// What resident `s` rides now, on their way (not while away at work with it).
pub(crate) fn ride_view(world: &World, s: &Sim) -> Option<RideView> {
    let bike = world.objects.get(s.riding? as usize)?;
    let kind = world.content.objects[bike.def].vehicle?.kind;
    if s.away_until.is_some() {
        return None;
    }
    let task = s.current().map(|a| a.task.kind);
    let home = matches!(task, Some(TaskKind::GoHome));
    let out = matches!(task, Some(TaskKind::Visit { .. }));
    let school = world.content.life.school.is_some() && s.job.is_none();
    let to = match (home, out, school) {
        (true, ..) => "home",
        (false, true, _) => "out",
        (false, false, true) => "school",
        (false, false, false) => "work",
    };
    Some(RideView { kind, to })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructureView<'a> {
    pub version: u32,
    /// `World::lot_version`: the lot parts below are left out when the caller already has them.
    pub lot_version: u32,
    /// `living` or `creative`.
    pub mode: crate::world::GameMode,
    pub width: usize,
    pub depth: usize,
    /// Storeys the lot holds at and above the ground, and rows per storey: storey `k` is rows
    /// `k * storeyDepth ..` of `depth` (see `storeys.rs`).
    pub storeys: u8,
    pub storey_depth: i32,
    /// The block of rows holding the basement (storey -1: rows `basement * storeyDepth ..`;
    /// storeys above it after that block are one lower than their place); absent: none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basement: Option<i32>,
    pub(crate) objects: Vec<ObjectView<'a>>,
    pub sims: Vec<SimInfo<'a>>,
    /// Names of former residents (story events name them with `social::FORMER`).
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub former: &'a [crate::world::Former],
    pub(crate) households: Vec<HouseholdView<'a>>,
    pub(crate) plots: Vec<PlotView<'a>>,
    pub exits: &'a [[f32; 2]],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<u16>>,
    /// Every edge with a wall on it (plain walls, and walls with a door or window).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) walls: Option<Vec<EdgeView>>,
    /// Doors and windows (also listed in `walls`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) openings: Option<Vec<OpeningView>>,
    /// Diagonal walls (with or without a door or window), one per tile at most.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) diagonals: Option<Vec<DiagonalView>>,
    /// Fences and gates (not walls: they make no rooms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fences: Option<Vec<FenceView>>,
    /// Floor coverings: `[x, z, covering]` per tile that has one (a floor covering + 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floors: Option<Vec<[u16; 3]>>,
    pub meta: &'a serde_json::Value,
    /// The town's servers (`staff.rs`), in their snapshot rows' order (after the residents').
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) staff: Vec<StaffView>,
    /// Households' cars (`cars.rs`), in their snapshot rows' order (`snapshot::car`). Left out
    /// only for a game saved before cars, which has no driveways either (the renderer parks its
    /// own by the neighbours' driveways); a town where nobody has one sends none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cars: Option<Vec<CarView>>,
    /// Garages at the inner ends of driveways (`cars::Garage`); left out when there are none.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub garages: &'a [crate::cars::Garage],
    /// Porches over doors (`porches::Porch`); left out when there are none.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub porches: &'a [crate::porches::Porch],
    /// Porch pieces standing on their own (`porches::PorchPiece`); left out when there are none.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub porch_pieces: &'a [crate::porches::PorchPiece],
    /// The streets where the kerbside furniture keeps back for cars to pass pulled over
    /// (`cars::pull_over_streets`); left out when there are none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) pull_over_streets: Vec<PullOverStreetView>,
    /// Where the traffic lights' poles stand and which way their heads face (`traffic::SignalArm`:
    /// x, z and yaw, in their snapshot rows' order): the kerbside furniture keeps clear of them;
    /// left out when there are none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub light_poles: Vec<[f32; 3]>,
    /// Every junction where `traffic::SIGNAL_ARMS` roads for cars or more meet (`traffic::Signal`),
    /// for its pad and markings: its middle, whether it's on a roundabout, and its arms, each with
    /// its crosswalk (whether it has one) and stop line and its signal head's snapshot row; the lit
    /// ones first, in their heads' rows' order, then those giving way (no rows); left out when
    /// there are none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) junctions: Vec<JunctionView>,
    /// Public parking: every marked bay, where a car parked in it stands and faces (`parking_views`);
    /// left out when there are none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parking: Vec<ParkingView>,
    /// The car parks the bays are grouped into, as the town file has them; left out when none.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    car_parks: &'a [crate::travel::CarPark],
    /// The flats (`flat_views`), each a plot of its own after the ground lots; left out when none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    flats: Vec<FlatView<'a>>,
}

/// A junction (`StructureView::junctions`, `traffic::Signal`): its middle (m), on a roundabout's
/// ring (`roundabout`: the renderer leaves its pad and markings out), and its arms.
#[derive(Serialize)]
pub(crate) struct JunctionView {
    pub x: f32,
    pub z: f32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub roundabout: bool,
    pub(crate) arms: Vec<JunctionArmView>,
}

/// A junction's arm (`traffic::SignalArm`): its road (`meta.roads` index), the way out along it
/// from the middle (a unit vector, x and z), the road's width between its kerbs (m), its crosswalk
/// (m out from the junction's middle, `walk_from` to `walk_to`; whether there's one, a zebra:
/// `crosswalk`), its stop line (m out, past the crosswalk: cars stop on red with their noses there,
/// or for those on foot on the zebra) and its signal head's row in the snapshot
/// (`snapshot::light`; none: not lit).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JunctionArmView {
    pub road: u32,
    pub dir: [f32; 2],
    pub width: f32,
    pub walk_from: f32,
    pub walk_to: f32,
    pub stop: f32,
    pub crosswalk: bool,
    pub light: Option<u32>,
}

/// `StructureView::junctions`: the lit junctions in their heads' rows' order, then those giving
/// way.
pub(crate) fn junction_views(world: &World) -> Vec<JunctionView> {
    let g = &world.travel.graph;
    let mut head = 0u32;
    world
        .travel
        .signals
        .iter()
        .chain(&world.travel.give_way)
        .map(|s| JunctionView {
            x: s.at[0],
            z: s.at[1],
            roundabout: s.roundabout,
            arms: s
                .arms
                .iter()
                .map(|a| {
                    let row = head;
                    head += u32::from(s.lit);
                    JunctionArmView {
                        road: g.roads.get(a.road as usize).map_or(a.road, |r| r.source),
                        dir: a.way,
                        width: 2.0 * a.kerb,
                        walk_from: a.walk_from,
                        walk_to: a.walk_to,
                        stop: a.stop,
                        crosswalk: a.crosswalk,
                        light: (s.lit && (row as usize) < crate::snapshot::MAX_LIGHTS)
                            .then_some(row),
                    }
                })
                .collect(),
        })
        .collect()
}

/// `StructureView::light_poles`.
pub(crate) fn light_pole_views(world: &World) -> Vec<[f32; 3]> {
    world
        .travel
        .signals
        .iter()
        .flat_map(|s| s.arms.iter().map(|a| [a.pole[0], a.pole[1], a.yaw]))
        .take(crate::snapshot::MAX_LIGHTS)
        .collect()
}

/// A street cars pull over on (`StructureView::pull_over_streets`): the town file's road
/// (`meta.roads` index) and how far in from its edges its kerbside furniture may reach (m,
/// `cars::kerbside`).
#[derive(Serialize)]
pub(crate) struct PullOverStreetView {
    pub road: u32,
    pub keep: f32,
}

/// `StructureView::pull_over_streets`.
pub(crate) fn pull_over_street_views(world: &World) -> Vec<PullOverStreetView> {
    crate::cars::pull_over_streets(&world.travel)
        .into_iter()
        .map(|(road, keep)| PullOverStreetView { road, keep })
        .collect()
}

/// A household's car: where it's parked (its middle, its yaw) and its look.
#[derive(Serialize)]
pub(crate) struct CarView {
    pub plot: u32,
    pub x: f32,
    pub z: f32,
    pub yaw: f32,
    pub body: u8,
    pub paint: u8,
    /// How old it is: 0 old, 1 used, 2 new (`content::CarAge`).
    pub age: u8,
}

/// One of the town's servers: what they look like (`seed`, the renderer makes the look) and the
/// posts they serve at (stable object ids).
#[derive(Serialize)]
pub(crate) struct StaffView {
    pub seed: u32,
    pub posts: Vec<u32>,
}

/// A diagonal wall across tile `(x, z)`: `dp` (`/`) from `(x, z)` to `(x + 1, z + 1)`, `dn`
/// (`\`) from `(x, z + 1)` to `(x + 1, z)`. `rooms` are the rooms of the tile's two halves:
/// half 0 touches the tile's `-z` side, half 1 its `+z` side.
#[derive(Serialize)]
pub(crate) struct DiagonalView {
    pub x: usize,
    pub z: usize,
    pub axis: &'static str,
    pub kind: &'static str,
    pub rooms: [u16; 2],
    #[serde(flatten)]
    pub(crate) look: LookView,
}

/// A wall's look, when it isn't the default: face coverings (0 = automatic, else wall covering
/// + 1; face 0 towards `-z` / `-x` / half 0), `form` 1 for a half wall, door or window `style`.
#[derive(Serialize, Default)]
pub(crate) struct LookView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faces: Option<[u8; 2]>,
    #[serde(skip_serializing_if = "is_zero")]
    pub form: u8,
    #[serde(skip_serializing_if = "is_zero")]
    pub style: u8,
    /// Doors and windows the player resized: sill and top (centimetres).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sill: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<u16>,
}

impl LookView {
    fn of(lot: &crate::lot::Lot, at: crate::lot::EdgeRef) -> Self {
        let l = lot.look(at);
        Self {
            faces: (l.sides != [0, 0]).then_some(l.sides),
            form: l.form,
            style: l.style,
            sill: l.sill,
            head: l.head,
        }
    }
}

pub(crate) fn diagonals(
    lot: &crate::lot::Lot,
    list: impl Iterator<Item = (usize, usize, crate::lot::Diagonal)>,
) -> Vec<DiagonalView> {
    list.map(|(x, z, d)| DiagonalView {
        x,
        z,
        axis: match d.dir {
            crate::lot::DiagDir::Dp => "dp",
            crate::lot::DiagDir::Dn => "dn",
        },
        kind: match d.edge {
            Edge::Door => "door",
            Edge::Window => "window",
            _ => "wall",
        },
        rooms: lot.half_rooms()[lot.tile_index(x as i32, z as i32)],
        look: LookView::of(lot, crate::lot::EdgeRef::Diag(x as u16, z as u16)),
    })
    .collect()
}

/// A wall edge: `h` runs from `(x, z)` to `(x + 1, z)`, `v` from `(x, z)` to `(x, z + 1)`.
#[derive(Serialize)]
pub(crate) struct EdgeView {
    pub axis: &'static str,
    pub x: usize,
    pub z: usize,
    /// Coverings and form (the style is on the opening).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faces: Option<[u8; 2]>,
    #[serde(skip_serializing_if = "is_zero")]
    pub form: u8,
}

#[derive(Serialize)]
pub(crate) struct OpeningView {
    pub axis: &'static str,
    pub x: usize,
    pub z: usize,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "is_zero")]
    pub style: u8,
    /// Resized by the player: sill and top, centimetres above the floor (absent: the style's).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sill: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<u16>,
}

/// A fence or gate on edge `axis` `(x, z)` (as walls), in a fence style.
#[derive(Serialize)]
pub(crate) struct FenceView {
    pub axis: &'static str,
    pub x: usize,
    pub z: usize,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "is_zero")]
    pub style: u8,
}

/// Wall edges, openings (doors, windows) and fences among `list` (`Lot::walls` for the whole
/// lot, in grid order).
pub(crate) fn wall_edges(
    lot: &crate::lot::Lot,
    list: impl Iterator<Item = (crate::lot::EdgeRef, Edge)>,
) -> (Vec<EdgeView>, Vec<OpeningView>, Vec<FenceView>) {
    let mut walls = Vec::new();
    let mut openings = Vec::new();
    let mut fences = Vec::new();
    let mut add = |axis: &'static str, x: usize, z: usize, e: Edge| {
        let at = if axis == "h" {
            crate::lot::EdgeRef::H(x as u16, z as u16)
        } else {
            crate::lot::EdgeRef::V(x as u16, z as u16)
        };
        if e.is_fence() {
            let kind = if e == Edge::Gate { "gate" } else { "fence" };
            let style = lot.look(at).style;
            fences.push(FenceView {
                axis,
                x,
                z,
                kind,
                style,
            });
            return;
        }
        if !e.is_wall() {
            return;
        }
        let look = LookView::of(lot, at);
        walls.push(EdgeView {
            axis,
            x,
            z,
            faces: look.faces,
            form: look.form,
        });
        let kind = match e {
            Edge::Door => "door",
            Edge::Window => "window",
            _ => return,
        };
        openings.push(OpeningView {
            axis,
            x,
            z,
            kind,
            style: look.style,
            sill: look.sill,
            head: look.head,
        });
    };
    for (at, e) in list {
        match at {
            crate::lot::EdgeRef::H(x, z) => add("h", x.into(), z.into(), e),
            crate::lot::EdgeRef::V(x, z) => add("v", x.into(), z.into(), e),
            crate::lot::EdgeRef::Diag(..) => {}
        }
    }
    (walls, openings, fences)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectView<'a> {
    pub id: u32,
    pub def: &'a str,
    pub x: i32,
    pub z: i32,
    pub rot: u8,
    /// Degrees past `rot` (objects that turn freely).
    pub turn: u8,
    /// Centimetres (x, z) off the middle of its footprint.
    #[serde(skip_serializing_if = "crate::world::centred")]
    pub offset: [i8; 2],
    /// Resized by the player: its own size, centimetres across and deep before turning (it
    /// stands in the middle of its tiles `w` × `d`, moved by `offset`), and centimetres taller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<[u16; 2]>,
    #[serde(skip_serializing_if = "is_level")]
    pub tall: i16,
    /// Things on the `surface` layer: the height (m) of the top they stand on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lift: Option<f32>,
    /// Wear 0..1 (broken at 1).
    pub wear: f32,
    /// Open (a wall bed folded down): see `ObjectInstance::open`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub open: bool,
    /// A crop's stage (`GrowthStage`: 0 planted, 1 growing, 2 ripe, 3 picked); none for other things.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub growth: Option<u8>,
    /// A machine running on its own (content `runs`: the drum turning, its sound); see
    /// `ObjectInstance::run`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub running: bool,
    pub w: i32,
    pub d: i32,
    pub quality: u8,
    /// The colours the player chose, `#rrggbb` by part (the model's material); absent: as made.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub colors: std::collections::BTreeMap<String, String>,
    /// What selling it returns (None if it can't be sold).
    pub sell_value: Option<i64>,
    /// What the quick fix costs (worn things; as of the last structure change, so exact for
    /// broken ones).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair_cost: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SimInfo<'a> {
    pub id: u32,
    pub name: &'a str,
    pub household: u32,
    pub gender: &'a str,
    pub attracted_to: &'a [String],
    pub appearance: &'a serde_json::Value,
    pub traits: &'a [String],
    pub perks: &'a [String],
    /// Died or moved away (kept so ids and snapshot rows match; not listed or drawn).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gone: Option<crate::world::Gone>,
    /// Bumped when someone new takes the slot.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub generation: u32,
    /// Life stage (content `life.stages` id), for how they look.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<&'a str>,
    /// The outfit they're in (`Sim::outfit`; 0, the creator's, left out).
    #[serde(skip_serializing_if = "is_zero")]
    pub outfit: u8,
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HouseholdView<'a> {
    pub id: u32,
    pub name: &'a str,
    pub plot: Option<u32>,
    pub player: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlotView<'a> {
    pub id: u32,
    pub name: &'a str,
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
    pub public: bool,
    pub entry: Option<[f32; 2]>,
    /// Bounding box of the walls on this plot `[x0, z0, x1, z1]`, if any (for silhouettes).
    pub house: Option<[i32; 4]>,
    /// The roof the player chose: `[style, colour]` (indices into `roofStyles`, `roofColors`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roof: Option<[u8; 2]>,
    /// Storeys with walls on this plot (for silhouettes).
    pub storeys: u8,
    /// When the place on it is open (`world::PlotHours`: `{open, close, days}`, minutes of the day
    /// and the days from Monday a bit each); absent: always (a home, a park).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<crate::world::PlotHours>,
    /// What the lot is (the town file's `plots[].kind`); absent: a file without kinds.
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<crate::travel::PlotKind>,
}

/// A flat (`StructureView::flats`, `world::Plot::of`): its plot id (after the last ground lot's),
/// its building's lot, its storey, its rectangle on the ground's rows, its name, and the landing
/// tile before its door, on its storey's rows.
#[derive(Serialize)]
pub(crate) struct FlatView<'a> {
    id: u32,
    of: u32,
    storey: i8,
    x: i32,
    z: i32,
    w: i32,
    d: i32,
    name: &'a str,
    door: Option<[i32; 2]>,
}

/// `StructureView::flats`: every flat, in id order.
pub(crate) fn flat_views(world: &World) -> Vec<FlatView<'_>> {
    world
        .flats()
        .iter()
        .filter_map(|p| {
            Some(FlatView {
                id: p.id,
                of: p.of?,
                storey: p.storey,
                x: p.x,
                z: p.z,
                w: p.w,
                d: p.d,
                name: &p.name,
                door: p.door.map(|(x, z)| [x, z]),
            })
        })
        .collect()
}

/// A marked parking bay (`StructureView::parking`, `travel::Parking`): where a car parked in it
/// stands and faces (the pose `Travel::parked` holds, the one cars park at), its area and car park.
#[derive(Serialize)]
pub(crate) struct ParkingView {
    x: f32,
    z: f32,
    yaw: f32,
    area: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    park: Option<u32>,
}

/// `StructureView::parking`: every bay a car may park at, in the town file's order.
pub(crate) fn parking_views(world: &World) -> Vec<ParkingView> {
    let travel = &world.travel;
    travel
        .file
        .parking
        .iter()
        .enumerate()
        .filter_map(|(k, p)| {
            let (at, yaw, _) = crate::cars::parked_at(travel, k)?;
            Some(ParkingView {
                x: at[0],
                z: at[1],
                yaw,
                area: p.area,
                park: p.park,
            })
        })
        .collect()
}

/// Bounding box of all wall/door edges (and diagonal walls) inside a plot. A wall on the plot's own
/// edge counts only where the plot's tile beside it is indoors: the wall of the house next door,
/// standing on the shared line (a terrace's), is that plot's, not this one's.
pub(crate) fn house_bounds(world: &World, x0: i32, z0: i32, x1: i32, z1: i32) -> Option<[i32; 4]> {
    let lot = &world.lot;
    let mut b: Option<[i32; 4]> = None;
    let mut add = |ax: i32, az: i32, bx: i32, bz: i32| {
        b = Some(match b {
            None => [ax, az, bx, bz],
            Some([a0, b0, a1, b1]) => [a0.min(ax), b0.min(az), a1.max(bx), b1.max(bz)],
        });
    };
    let indoors = |x: i32, z: i32| lot.room_at(x, z) != 0;
    for z in z0..=z1.min(lot.depth as i32) {
        for x in x0..x1.min(lot.width as i32) {
            if z < lot.depth as i32 + 1 && lot.h_edge(x as usize, z as usize).is_wall() {
                if (z == z0 && !indoors(x, z)) || (z == z1 && !indoors(x, z - 1)) {
                    continue;
                }
                add(x, z, x + 1, z);
            }
        }
    }
    for z in z0..z1.min(lot.depth as i32) {
        for x in x0..=x1.min(lot.width as i32) {
            if lot.v_edge(x as usize, z as usize).is_wall()
                && !((x == x0 && !indoors(x, z)) || (x == x1 && !indoors(x - 1, z)))
            {
                add(x, z, x, z + 1);
            }
            if x < x1 && lot.diag(x, z).is_some() {
                add(x, z, x + 1, z + 1);
            }
        }
    }
    b
}

/// Per object, the height of the top it stands on (`surface_lifts`).
pub(crate) type Lifts = Vec<Option<f32>>;

/// Per object, the height of the top it stands on (things on the `surface` layer; None for the
/// rest, or on a top at floor level).
pub(crate) fn surface_lifts(world: &World) -> Lifts {
    let content = &world.content;
    let on_top = |o: &crate::world::ObjectInstance| {
        content.objects[o.def].layer == crate::content::Layer::Surface
    };
    if !world.objects.iter().any(on_top) {
        return Vec::new();
    }
    // The top over each tile (furniture never shares tiles), checked against where each thing's
    // middle is; a thing over the edge of its tile's top looks through them all.
    let lot = &world.lot;
    let mut tops = vec![u32::MAX; lot.tile_slots()];
    for o in world
        .objects
        .iter()
        .filter(|o| content.objects[o.def].surface.is_some())
    {
        for (x, z) in o.tiles(content).filter(|&(x, z)| lot.in_bounds(x, z)) {
            tops[lot.tile_index(x, z)] = o.id;
        }
    }
    world
        .objects
        .iter()
        .map(|o| {
            if !on_top(o) {
                return None;
            }
            let p = crate::surface::middle(content, o);
            let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
            let near = lot
                .in_bounds(x, z)
                .then(|| tops[lot.tile_index(x, z)])
                .filter(|&h| h != u32::MAX);
            let top = near
                .and_then(|h| crate::surface::Top::of(content, &world.objects[h as usize]))
                .filter(|t| t.covers(p))
                .or_else(|| {
                    crate::surface::host_of(content, &world.objects, o)
                        .and_then(|h| crate::surface::Top::of(content, &world.objects[h as usize]))
                })?;
            Some(top.surface.height).filter(|&h| h > 0.0)
        })
        .collect()
}

/// An object as views send it.
pub(crate) fn object_view<'a>(
    world: &'a World,
    lifts: &[Option<f32>],
    o: &'a crate::world::ObjectInstance,
) -> ObjectView<'a> {
    let content = &world.content;
    let (w, d) = o.size(content);
    ObjectView {
        id: o.stable,
        def: &content.objects[o.def].id,
        x: o.x,
        z: o.z,
        rot: o.rot,
        turn: o.turn,
        offset: o.offset,
        size: (o.extent != crate::world::kind_size(content.objects[o.def].footprint))
            .then_some(o.extent),
        tall: o.tall,
        lift: lifts.get(o.id as usize).copied().flatten(),
        wear: o.wear,
        open: o.open,
        growth: world.content.objects[o.def]
            .grows
            .is_some()
            .then(|| o.growth_stage() as u8),
        running: o.running(),
        w,
        d,
        quality: o.quality,
        colors: crate::world::colors_hex(&o.colors),
        sell_value: content.objects[o.def].price.map(|_| {
            let broken = if o.broken() { 0.5 } else { 1.0 };
            (o.value as f32 * content.object_rules.resale * broken).round() as i64
        }),
        repair_cost: (o.wear > 0.0).then(|| crate::home::repair_cost(content, o)),
    }
}

mod json;
pub use json::*;

/// `StructureView::basement`.
pub(crate) fn basement_block(world: &World) -> Option<i32> {
    (world.stair_map().basement > 0).then_some(world.stair_map().basement)
}

pub(crate) fn sim_infos(world: &World) -> Vec<SimInfo<'_>> {
    let content = &world.content;
    world
        .sims
        .iter()
        .map(|s| SimInfo {
            id: s.id,
            name: &s.name,
            household: s.household,
            gender: &s.gender,
            attracted_to: &s.attracted_to,
            appearance: &s.appearance,
            traits: &s.traits,
            perks: &s.perks,
            gone: s.gone,
            generation: world.generations.get(s.id as usize).copied().unwrap_or(0),
            stage: content
                .life
                .stages
                .get(content.life.stage(s.age))
                .map(|st| st.id.as_str()),
            outfit: s.outfit,
        })
        .collect()
}

pub(crate) fn household_views(world: &World) -> Vec<HouseholdView<'_>> {
    world
        .households
        .iter()
        .map(|h| HouseholdView {
            id: h.id,
            name: &h.name,
            plot: h.plot,
            player: h.player,
        })
        .collect()
}

/// `StructureView::plots`: the lots on the ground, in id order (flats go in `flats`).
pub(crate) fn plot_views(world: &World) -> Vec<PlotView<'_>> {
    world
        .lots()
        .iter()
        .map(|p| PlotView {
            id: p.id,
            name: &p.name,
            x: p.x,
            z: p.z,
            w: p.w,
            d: p.d,
            public: p.public,
            entry: p.entry,
            house: house_bounds(world, p.x, p.z, p.x + p.w, p.z + p.d),
            roof: p.roof.map(|r| [r.style, r.color]),
            storeys: 1
                + (1..world.storeys as i32)
                    .filter(|k| {
                        let up = world.storey_row(*k);
                        house_bounds(world, p.x, p.z + up, p.x + p.w, p.z + p.d + up).is_some()
                    })
                    .count() as u8,
            hours: p.hours,
            kind: p.kind,
        })
        .collect()
}

pub(crate) fn floor_views(world: &World) -> Vec<[u16; 3]> {
    world
        .lot
        .floors()
        .map(|(x, z, c)| [x, z, u16::from(c)])
        .collect()
}

pub(crate) fn staff_views(world: &World) -> Vec<StaffView> {
    crate::staff::current(world)
        .servers
        .iter()
        .take(crate::staff::MAX_STAFF)
        .map(|s| StaffView {
            seed: s.seed,
            posts: s
                .posts
                .iter()
                .map(|&o| world.objects[o as usize].stable)
                .collect(),
        })
        .collect()
}

/// `StructureView::cars`.
pub(crate) fn car_views(world: &World) -> Option<Vec<CarView>> {
    (!world.cars.is_empty() || !world.drives.is_empty()).then(|| {
        world
            .cars
            .iter()
            .take(crate::cars::MAX_CARS)
            .filter(|c| !c.taxi)
            .map(|c| CarView {
                plot: c.plot,
                x: c.spot[0],
                z: c.spot[1],
                yaw: c.yaw,
                body: c.body,
                paint: c.paint,
                age: c.age.index(),
            })
            .collect()
    })
}

pub(crate) fn is_zero(n: &u8) -> bool {
    *n == 0
}
