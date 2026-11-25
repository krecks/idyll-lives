//! The town's travel state: entries and doors, ETAs and walk plans.

use super::*;
/// How a way is searched for (`Graph::path_by`): setting off only towards this node (an end of the
/// start's edge), coming in only from this one (`arrive`: an end of the goal's edge), and these
/// edges costing this much more each.
#[derive(Debug, Clone, Default)]
pub struct Way {
    pub toward: Option<u32>,
    pub arrive: Option<u32>,
    pub avoid: Vec<(u32, f32)>,
}

/// A stretch of a road: the road, and from and to how far along it (m).
pub(super) type Piece = (u32, f32, f32);

/// Road searches' state kept between them (`Graph::search`): a big world's network has
/// thousands of nodes, too many to allocate for every estimate.
#[derive(Default)]
pub(super) struct RoadScratch {
    pub(super) search: crate::path::Scratch<f32, Option<u32>, (u64, u32)>,
    /// The stretches of road the way found goes along.
    pub(super) pieces: Vec<Piece>,
}

thread_local! {
    pub(super) static ROAD_SCRATCH: std::cell::RefCell<RoadScratch> = std::cell::RefCell::default();
}

/// Everything the sim knows about getting about (`World::travel`).
#[derive(Debug, Clone, Default)]
pub struct Travel {
    /// The town file's open-world fields, as they came (saved).
    pub file: OpenWorld,
    pub graph: Graph,
    /// Cars' top speed on country roads (m/s; content `cars.countrySpeed`).
    pub country_speed: f32,
    /// The tiles the town file's `terrain` masks (water, rock): blocked under everything, so a
    /// tile freed of what stood on it stays blocked (`World::set_travel`). Sorted tile indices.
    pub base_blocked: Vec<usize>,
    /// The lit junctions' boxes and their carriageways out past their stop lines but for the
    /// crosswalks (`traffic::settle`, `traffic::KEEP_OFF`): blocked to those on foot, who cross on
    /// the zebras. Sorted tile indices; not saved.
    pub(crate) junction_tiles: Vec<usize>,
    /// The junctions with traffic lights, the big ones (`traffic::Signal`, `traffic::settle`). Not
    /// saved: worked out again on load.
    pub signals: Vec<crate::traffic::Signal>,
    /// The other junctions, giving way (`Signal::lit` false: no lights, zebras on their streets'
    /// arms). Not saved.
    pub give_way: Vec<crate::traffic::Signal>,
    /// The lit junctions' crosswalks by where they are (`traffic::kerb_stop`), and the zebras of
    /// those giving way (`traffic::zebra_walkers`). Not saved.
    pub(crate) crossings: crate::traffic::Crossings,
    pub(crate) zebras: crate::traffic::Crossings,
    /// By public parking spot (`OpenWorld::parking`), whether a car parked there would stand where
    /// a household's car turns into or out of its driveway, or on the driveway (`cars::settle_all`):
    /// not parked at on a trip. Not saved: worked out again on load.
    pub(crate) by_a_mouth: Vec<bool>,
    /// By plot: its door and where it gets on the roads on foot and by car (`entry`), kept as
    /// each tick starts (`note_doors`): a resident weighing an outing asks how long the way to
    /// every public place is, one out on one how long the way home. Not saved.
    doors: Vec<Option<Door>>,
    /// By public parking spot (`OpenWorld::parking`): where a car parked there stands, which way
    /// it faces and whether it's a bay off the road (`cars::park_spot`), worked out once: the
    /// spots and the roads stay as the town file has them. Not saved.
    pub(crate) parked: Vec<Option<([f32; 2], f32, bool)>>,
    /// The trails' indices into `graph.roads`, ascending: `outings::trail_step` projects only
    /// onto these, not onto every road in town. Not saved: the graph is the town file's.
    pub(crate) trails: Vec<u32>,
}

/// A plot's door (its bits) and where it gets on the roads on foot and by car
/// (`Travel::doors`).
type Door = ([u32; 2], [Option<RoadPoint>; 2]);

impl Travel {
    pub fn new(file: OpenWorld, meta: &serde_json::Value) -> Self {
        // Every town is one (cosy and normal were dropped in 0.8.0): a town without open-world
        // fields takes its meta's roads, or its streets, as the network.
        let graph = Graph::from_town(&file, meta);
        let mut travel = Self {
            file,
            graph,
            country_speed: COUNTRY_SPEED,
            base_blocked: Vec::new(),
            junction_tiles: Vec::new(),
            by_a_mouth: Vec::new(),
            signals: Vec::new(),
            give_way: Vec::new(),
            crossings: Default::default(),
            zebras: Default::default(),
            doors: Vec::new(),
            parked: Vec::new(),
            trails: Vec::new(),
        };
        travel.trails = travel
            .graph
            .roads
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == RoadKind::Trail)
            .map(|(i, _)| i as u32)
            .collect();
        travel.parked = (0..travel.file.parking.len())
            .map(|k| crate::cars::park_spot(&travel, k))
            .collect();
        travel
    }

    /// Where a trip from or to `p` gets on the roads for `mode` (the nearest road within
    /// `ENTRY_REACH`), as `eta` finds it.
    pub fn entry(&self, p: [f32; 2], mode: Mode) -> Option<RoadPoint> {
        self.graph.nearest(p, ENTRY_REACH, mode)
    }

    /// Whether those on foot or on a bike may leave town by the way out at `exit`: one at the end
    /// of a road they go along, or of none. Not a motorway's carriageway's end: only a road for
    /// cars there, which nobody walks or rides along (`Road::walkable`), its ground kept from them.
    pub fn exit_on_foot(&self, exit: [f32; 2]) -> bool {
        self.graph.nearest(exit, EXIT_ROAD, Mode::Foot).is_some()
            || self.graph.nearest(exit, EXIT_ROAD, Mode::Car).is_none()
    }

    /// Notes the plots' doors and where they get on the roads (`doors`), again only for a door
    /// that moved.
    pub(crate) fn note_doors(&mut self, plots: &[crate::world::Plot]) {
        self.doors.resize(plots.len(), None);
        for (i, p) in plots.iter().enumerate() {
            let door = p.arrival_point();
            let bits = door.map(f32::to_bits);
            if self.doors[i].is_none_or(|(at, _)| at != bits) {
                let entries = [self.entry(door, Mode::Foot), self.entry(door, Mode::Car)];
                self.doors[i] = Some((bits, entries));
            }
        }
    }

    /// `entry` for the door `door` of plot `plot`, as noted (`note_doors`), or found.
    /// (The sim tests check it against `entry`: `common::check_lookups`.)
    pub fn door_entry(&self, plot: u32, door: [f32; 2], mode: Mode) -> Option<RoadPoint> {
        match self.doors.get(plot as usize) {
            Some(Some((at, entries))) if *at == door.map(f32::to_bits) => entries[mode as usize],
            _ => self.entry(door, mode),
        }
    }

    /// Whether tile `i` (`Lot::tile_index`) is blocked by the land itself (`base_blocked`), or is
    /// a lit junction's off-limits to those on foot (`junction_tiles`).
    pub fn ground_blocked(&self, i: usize) -> bool {
        self.base_blocked.binary_search(&i).is_ok() || self.junction_tiles.binary_search(&i).is_ok()
    }

    /// Whether `p` is in a road's carriageway, between the kerbs of one for cars (a country
    /// road's, its edges), `margin` m in from them at least (`Graph::carriageways_near`: not the
    /// motorway's).
    pub fn on_carriageway(&self, p: [f32; 2], margin: f32) -> bool {
        self.graph
            .carriageways_near(p, CARRIAGEWAY_LOOK, 4)
            .any(|(off, kerb)| off <= kerb - margin)
    }

    /// Whether a trip from `a` to `b` on foot goes by road.
    pub fn by_road(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        !self.graph.is_empty() && octile(a, b) >= ROAD_TRIP_TILES
    }

    /// About how many minutes a trip from `from` to `to` takes: on foot at `walk` m a minute
    /// (game minutes, as `walk` is), or by car (minutes of drive time: times `DRIVE_MINUTES` in
    /// game minutes). No search of the tiles: the roads for long trips, the
    /// straight line (with a detour) for the rest.
    pub fn eta(&self, from: [f32; 2], to: [f32; 2], mode: Mode, walk: f32) -> f32 {
        self.eta_via(
            from,
            to,
            mode,
            walk,
            || self.entry(from, mode),
            || self.entry(to, mode),
        )
    }

    /// `eta` to the door `door` of plot `plot`, where it gets off the roads as noted for it
    /// (`door_entry`).
    pub fn eta_to_door(
        &self,
        from: [f32; 2],
        plot: u32,
        door: [f32; 2],
        mode: Mode,
        walk: f32,
    ) -> f32 {
        self.eta_via(
            from,
            door,
            mode,
            walk,
            || self.entry(from, mode),
            || self.door_entry(plot, door, mode),
        )
    }

    /// `eta`, with where the trip gets on and off the roads (`entry` of `from` and of `to`, asked
    /// for only if it goes by road) found by the caller: the same for many trips from one place.
    pub fn eta_via(
        &self,
        from: [f32; 2],
        to: [f32; 2],
        mode: Mode,
        walk: f32,
        on: impl FnOnce() -> Option<RoadPoint>,
        off: impl FnOnce() -> Option<RoadPoint>,
    ) -> f32 {
        let walk = walk.max(1e-3);
        match mode {
            Mode::Foot => {
                if !self.by_road(from, to) {
                    return octile(from, to) * DETOUR / walk;
                }
                self.road_estimate(on, off, Mode::Foot)
                    .map_or(octile(from, to) * DETOUR, |(m, _)| m)
                    / walk
            }
            Mode::Car => {
                let straight = dist(from, to) / TOWN_SPEED / 60.0;
                self.road_estimate(on, off, Mode::Car)
                    .map_or(straight, |(_, s)| s / 60.0)
            }
        }
    }

    /// At least as many minutes as `eta` says the trip takes (in its units): the straight line at
    /// the fastest way along it (no search at all). For finding the quickest of several without
    /// working out every one (`world::quickest_exit`).
    pub fn eta_at_least(&self, from: [f32; 2], to: [f32; 2], mode: Mode, walk: f32) -> f32 {
        let d = dist(from, to);
        match mode {
            Mode::Foot => d / walk.max(1e-3),
            Mode::Car => d / TOWN_SPEED.max(self.country_speed) / 60.0,
        }
    }

    /// Along the roads from where a trip gets on them (`on`) to where it gets off: the metres on foot (tile legs with a detour), and the
    /// seconds of drive time by car (`mode`).
    fn road_estimate(
        &self,
        on: impl FnOnce() -> Option<RoadPoint>,
        off: impl FnOnce() -> Option<RoadPoint>,
        mode: Mode,
    ) -> Option<(f32, f32)> {
        let a = on()?;
        let b = off()?;
        let (len, cost) = self.graph.path_cost(a, b, mode, self.country_speed)?;
        let legs = (a.off + b.off) * DETOUR;
        Some((len + legs, cost + legs / TOWN_SPEED.min(1.5)))
    }

    /// Where a trip on foot from `pos` to `target` gets on and off the roads, and the way along
    /// them in between (on the walkers' side, thinned): `ons` and `offs` are the candidates to
    /// try in turn (the caller finds the tile legs). None: no network, or no way along it.
    pub fn walk_plan(&self, pos: [f32; 2], target: [f32; 2]) -> Option<WalkPlan> {
        let ons = self.graph.near(pos, ENTRY_REACH, ENTRIES, Mode::Foot);
        let offs = self.graph.near(target, ENTRY_REACH, ENTRIES, Mode::Foot);
        if ons.is_empty() || offs.is_empty() {
            return None;
        }
        Some(WalkPlan { ons, offs })
    }

    /// The way along the roads from `on` to `off` on foot, as waypoints.
    pub fn walk_between(&self, on: RoadPoint, off: RoadPoint) -> Option<(Vec<[f32; 2]>, f32)> {
        let path = self.graph.path(on, off, Mode::Foot, self.country_speed)?;
        Some((thin(&walk_side(&path.pts)), path.len))
    }
}
