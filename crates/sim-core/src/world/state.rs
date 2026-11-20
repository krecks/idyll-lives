//! Building a world, and the read-only view of it: what is where, and what is open.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

impl World {
    pub fn from_json(content_json: &str, lot_json: &str, seed: u32) -> Result<Self, Error> {
        let content = Content::from_json(content_json)?;
        let lot_file: LotFile = serde_json::from_str(lot_json)?;
        let world = Self::new(content, &lot_file, seed)?;
        Ok(world)
    }
    pub fn new(content: Content, lot_file: &LotFile, seed: u32) -> Result<Self, Error> {
        let styles: Vec<&str> = content.build.fences.iter().map(|s| s.id.as_str()).collect();
        let lot = Lot::from_file_styled(lot_file, &styles)?;
        let mut world = Self::empty(content, lot, Rng::new(seed));
        world.seed = seed;
        // Storeys from the file before its objects (stairs, furniture upstairs).
        let storeys = lot_file.storeys.unwrap_or(1).clamp(1, 4);
        if storeys > 1 || lot_file.basement {
            world.storeys = storeys;
            world.stairs.depth = lot_file.depth as i32;
        }
        // The basement's block after the storeys' (storeys the content adds go after it).
        if lot_file.basement {
            world.stairs.basement = storeys as i32;
        }
        world.mode = lot_file.mode;
        world.lifespan = lot_file.lifespan;
        world.residents_most_given = lot_file.residents_most;
        world.meta = lot_file.meta.clone();
        world.note_paths();
        // (A bus stop in an older town file is no way out: left out.)
        world.exits = lot_file
            .exits
            .iter()
            .filter(|e| e.kind() != crate::travel::ExitKind::Stop)
            .map(|e| e.point())
            .collect();
        world.plots = lot_file
            .plots
            .iter()
            .enumerate()
            .map(|(i, p)| Plot {
                id: i as u32,
                name: p.name.clone(),
                x: p.x,
                z: p.z,
                w: p.w,
                d: p.d,
                public: p.public,
                entry: p.entry,
                roof: None,
                hours: p.hours,
            })
            .collect();
        let starting_funds = world.content.starting_funds;
        world.households = if lot_file.households.is_empty() {
            vec![Household {
                id: 0,
                name: "Household".into(),
                plot: None,
                player: true,
                funds: starting_funds,
                free_will: true,
                routines: Vec::new(),
                expecting: None,
                sitter_day: None,
                laundry: LAUNDRY_START,
                letters: Vec::new(),
            }]
        } else {
            lot_file
                .households
                .iter()
                .enumerate()
                .map(|(i, h)| Household {
                    id: i as u32,
                    name: h.name.clone(),
                    plot: h.plot,
                    player: h.player,
                    funds: h.funds.unwrap_or(starting_funds),
                    free_will: true,
                    routines: Vec::new(),
                    expecting: None,
                    sitter_day: None,
                    laundry: LAUNDRY_START,
                    letters: Vec::new(),
                })
                .collect()
        };
        if let Some(h) = world
            .households
            .iter()
            .find(|h| h.plot.is_some_and(|p| p as usize >= world.plots.len()))
        {
            return Err(Error::new(format!(
                "household '{}' lives on an unknown plot",
                h.name
            )));
        }
        // The road network and the town's other open-world fields (`travel.rs`).
        let mut open_world = lot_file.open_world.clone();
        if lot_file
            .plots
            .iter()
            .any(|p| p.area.is_some() || p.kind.is_some())
        {
            open_world.plot_areas = lot_file.plots.iter().map(|p| p.area).collect();
            open_world.plot_kinds = lot_file.plots.iter().map(|p| p.kind).collect();
        }
        world.set_travel(open_world);
        for p in &lot_file.objects {
            let def = world
                .content
                .object_index(&p.def)
                .ok_or_else(|| Error::new(format!("unknown object '{}'", p.def)))?;
            // Turned (only what turns freely may be), set off its tile's middle or resized.
            let at = |e: Error| Error::new(format!("'{}' at {},{}: {e}", p.def, p.x, p.z));
            let turn = world.check_turn(def, p.turn).map_err(at)?;
            let layout = world
                .check_layout(
                    def,
                    p.rot,
                    Layout {
                        offset: p.offset,
                        extent: p.size,
                        tall: p.tall,
                    },
                )
                .map_err(at)?;
            let value = world.content.objects[def].price.unwrap_or(0);
            let id = world.place(
                def,
                p.x,
                p.z,
                p.rot,
                layout,
                Default::default(),
                0,
                value,
                None,
            )?;
            world.objects[id as usize].turn = turn;
        }
        if lot_file.sims.len() > MAX_SIMS {
            return Err(Error::new(format!("at most {MAX_SIMS} residents per town")));
        }
        for s in &lot_file.sims {
            world.spawn_sim(s)?;
        }
        world.init_relationships(&lot_file.relationships)?;
        // Porches as the file has them (a file from before them: as they were drawn), and the
        // pieces (before the cars: a post's tile is kept as a garage's, `cars::reblock`).
        world.porches = match &lot_file.porches {
            Some(list) => list
                .iter()
                .filter(|p| (p.plot as usize) < world.plots.len())
                .copied()
                .collect(),
            None => world.guess_porches(),
        };
        world.porch_pieces = lot_file
            .porch_pieces
            .iter()
            .filter(|p| (p.plot as usize) < world.plots.len())
            .copied()
            .collect();
        crate::cars::from_file(
            &mut world,
            &lot_file.cars,
            &lot_file.drives,
            &lot_file.garages,
        );
        if !world.porch_pieces.is_empty() {
            crate::cars::posts_changed(&mut world);
        }
        crate::traffic::settle(&mut world);
        // Babies come with a crib (a crowded home is reported to the player as time starts).
        for h in 0..world.households.len() {
            crate::lifecycle::cribs_for_babies(&mut world, h);
        }
        world.make_storeys();
        world.refresh_rooms();
        // A new lot's lamps come on only where someone is (`lights::Watch::fresh`).
        world.light_watch.fresh = true;
        // And its curtains are drawn as fits the hour (`curtains::Watch::fresh`).
        world.curtain_watch.fresh = true;
        Ok(world)
    }
    pub(crate) fn empty(content: Content, lot: Lot, rng: Rng) -> Self {
        let tiles = lot.tile_slots();
        Self {
            content,
            mode: GameMode::Living,
            lot,
            objects: Vec::new(),
            next_object_id: 0,
            items: Default::default(),
            setups: Vec::new(),
            object_plot: Vec::new(),
            object_index: ObjectIndex::default(),
            sims: Vec::new(),
            households: Vec::new(),
            plots: Vec::new(),
            tile_plot: Vec::new(),
            relationships: Relationships::default(),
            events: EventLog::default(),
            blocked: vec![false; tiles],
            tick: 0,
            speed: 1,
            resume_speed: 1,
            was_calm: false,
            auto_super: false,
            step_ticks: 0,
            autonomy: true,
            lifespan: Default::default(),
            player_moves: true,
            residents_most_given: None,
            former: Vec::new(),
            generations: Vec::new(),
            rng,
            seed: 0,
            structure_version: 1,
            nav_version: 1,
            nav_log: Default::default(),
            search_mark: 0,
            search_budget: SEARCH_BUDGET,
            objects_version: 1,
            reach: Default::default(),
            reach_version: 0,
            lot_version: 1,
            briefs: Vec::new(),
            members: Members::default(),
            shut: Vec::new(),
            shut_by: Vec::new(),
            private_rooms: Vec::new(),
            claimed_rooms: Vec::new(),
            claimed_places: Vec::new(),
            claimants_cache: Vec::new(),
            work: Vec::new(),
            first: Vec::new(),
            turns: Vec::new(),
            turns_fresh: false,
            broadcast_scratch: BroadcastScratch::default(),
            exits: Vec::new(),
            travel: Default::default(),
            cars: Vec::new(),
            traffic: Default::default(),
            drives: Vec::new(),
            taxis: Default::default(),
            garages: Vec::new(),
            porches: Vec::new(),
            porch_pieces: Vec::new(),
            car_tiles: Vec::new(),
            cars_nav: 0,
            cars_parked: None,
            cars_near: Default::default(),
            meta: serde_json::Value::Null,
            paths: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            auto_fast: false,
            offers: Vec::new(),
            offers_version: 0,
            dirt: vec![0.0; tiles],
            rooms: Vec::new(),
            rooms_version: 0,
            room_layout_version: 1,
            rooms_things: false,
            rooms_things_at: Vec::new(),
            rooms_things_all: false,
            rooms_out: Vec::new(),
            rooms_dirty: false,
            rooms_rescore: false,
            room_score: Vec::new(),
            room_tiles: Default::default(),
            storeys: 1,
            stairs: crate::path::StairMap::default(),
            lod: true,
            view: None,
            watch: None,
            watched: Vec::new(),
            lights_version: 0,
            light_watch: Default::default(),
            curtains_version: 0,
            curtain_watch: Default::default(),
            while_lit: Vec::new(),
            staff: Default::default(),
            walk_memo: Vec::new(),
        }
    }
    /// Bumped whenever walls or objects change, so renderers know to rebuild.
    pub fn structure_version(&self) -> u32 {
        self.structure_version
    }
    /// Bumped whenever a lamp is switched (see `lights::lamps_off`).
    pub fn lights_version(&self) -> u32 {
        self.lights_version
    }
    /// Bumped whenever curtains are drawn or opened (see `curtains::shut`).
    pub fn curtains_version(&self) -> u32 {
        self.curtains_version
    }
    /// Changes whenever the lot does (see `lot_version`); never while only objects move.
    pub fn lot_version(&self) -> u32 {
        self.lot_version
    }
    pub fn blocked(&self) -> &[bool] {
        &self.blocked
    }
    /// Whether someone at `b` hears someone at `a` call their name (see
    /// `conversation::within_earshot`).
    pub fn within_earshot(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        conversation::within_earshot(&self.nav(), a, b)
    }
    /// The most residents the town grows to by newcomers and births: the town file's
    /// `residentsMost`, at most `MAX_SIMS`.
    pub fn residents_most(&self) -> usize {
        self.residents_most_given
            .map_or(MAX_SIMS, |n| (n as usize).min(MAX_SIMS))
    }
    /// Whether a newcomer or a baby would find room in town (fewer here than `residents_most`).
    pub(crate) fn room_in_town(&self) -> bool {
        self.sims.iter().filter(|s| s.here()).count() < self.residents_most()
    }
    /// Whether people at `a` and `b` see each other, as two talking must (see
    /// `conversation::in_sight`).
    pub fn in_sight(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        conversation::in_sight(&self.nav(), a, b)
    }
    /// Who to look through for the residents of household `h` as they are now (here or not; a
    /// caller still asks each's household): its members by the tick's briefs (`Members`) while
    /// nobody has moved house or come to town since, else everyone.
    pub(crate) fn household_sims(&self, h: u32) -> impl Iterator<Item = usize> + '_ {
        let fresh = !self.members.moved && self.members_cover();
        let (list, all) = if fresh {
            (self.members.of(h), 0..0)
        } else {
            (&[][..], 0..self.sims.len())
        };
        list.iter().map(|&j| j as usize).chain(all)
    }
    /// Whether resident `i` would set off now to visit `plot` (an invitation over the phone):
    /// the way isn't too long, as they'd go (`leisure_way`), they'd be there in visiting hours
    /// with time for the stay before they leave for work or school themselves (`visit_fits`,
    /// `free_before_work`), and a household living there isn't all off to work or school before
    /// they could stay `visits.leastMinutes` (`hosts_about_till`).
    pub(crate) fn visit_fits_now(&self, i: usize, plot: u32) -> bool {
        let Some(eta) = self.leisure_eta_now(i, plot) else {
            return false;
        };
        let stay = crate::mins(eta + self.content.visits.least_minutes);
        // (With time for it before they leave for work or school, as a visit they'd choose for
        // themselves: invited round, a guest let in was off to work 39 minutes on, Brightton.)
        let ctx = self.ctx();
        visit_fits(
            &self.content,
            eta,
            ctx.hour,
            free_before_work(&self.sims[i], &ctx),
        ) && self
            .households
            .iter()
            .find(|h| h.plot == Some(plot))
            .is_none_or(|h| hosts_about_till(&ctx, h.id) >= self.tick + stay)
    }
    /// Why resident `i` can't be sent to `plot` now for its hours (`Plot::open_at`): it's shut,
    /// or it shuts before they could get there (the way they'd go, `leisure_eta_now`). Ok: open
    /// for them (or a place with no hours).
    pub(crate) fn check_open(&self, i: usize, plot: u32) -> Result<(), Error> {
        let p = &self.plots[plot as usize];
        if p.hours.is_none() {
            return Ok(());
        }
        let (day, minute) = (
            crate::clock::day(self.tick),
            crate::clock::minute_of_day(self.tick),
        );
        let clock = |m: u16| format!("{:02}:{:02}", m / 60 % 24, m % 60);
        if !p.open_at(day, minute) {
            let when = p.opens_next(day, minute).map(|(d, m)| match d - day {
                0 => format!(" (opens at {})", clock(m)),
                1 => format!(" (opens tomorrow at {})", clock(m)),
                _ => format!(
                    " (opens on {} at {})",
                    WEEKDAYS[crate::clock::weekday(d) as usize],
                    clock(m)
                ),
            });
            return Err(Error::new(format!(
                "{} is closed{}",
                p.name,
                when.unwrap_or_default()
            )));
        }
        let open = p.open_for(day, minute);
        if let Some(eta) = self.leisure_eta_now(i, plot)
            && open < eta
        {
            let shuts = (minute + open).round() as u16;
            return Err(Error::new(format!(
                "{} closes at {}, before {} could get there",
                p.name,
                clock(shuts % (24 * 60)),
                self.sims[i].name
            )));
        }
        Ok(())
    }
    /// About how many minutes resident `i` would take to get to `plot` from where they are now,
    /// the way they'd go (`leisure_way`: the car, a bike of theirs, the taxi, else on foot).
    pub(crate) fn leisure_eta_now(&self, i: usize, plot: u32) -> Option<f32> {
        let (Some(sim), Some(p)) = (self.sims.get(i), self.plots.get(plot as usize)) else {
            return None;
        };
        let pos = self.nav().stairs.ground_pos(sim.pos);
        let goer = Goer {
            drives: crate::cars::can_drive(
                &self.travel,
                &self.households,
                &self.briefs,
                &self.cars,
                sim,
            ),
            bike: crate::ride::leisure_bike(
                &self.content,
                &self.object_index,
                &self.briefs,
                &self.households,
                &self.objects,
                sim,
            )
            .map(|(_, speed, min)| (speed, min)),
        };
        let door = p.arrival_point();
        let taxi = self
            .taxis
            .takes(sim)
            .then(|| {
                crate::taxi::leisure_eta(&self.travel, &self.exits, &self.content, sim, pos, door)
            })
            .flatten();
        let (_, eta) = leisure_way(
            &self.travel,
            &self.content,
            pos,
            sim.walk_per_minute(&self.content),
            plot,
            door,
            goer,
            taxi,
            &mut [None, None],
        );
        Some(eta)
    }
    /// The plot a tile is on (on any storey).
    pub fn plot_at(&self, x: i32, z: i32) -> Option<u32> {
        let z = self.ground_z(z);
        let width = self.lot.width as i32;
        if (0..width).contains(&x)
            && z >= 0
            && let Some(&plot) = self.tile_plot.get((z * width + x) as usize)
        {
            return (plot != crate::path::NO_PLOT).then_some(plot as u32);
        }
        self.plots.iter().find(|p| p.contains(x, z)).map(|p| p.id)
    }
    /// The index in `World::objects` of the object with stable id `stable`, if it still exists.
    pub fn index_of(&self, stable: u32) -> Option<usize> {
        self.objects.iter().position(|o| o.stable == stable)
    }
    /// The stable id of the object at `index` in `World::objects` (the inverse of `index_of`).
    pub fn stable_of(&self, index: u32) -> Option<u32> {
        self.objects.get(index as usize).map(|o| o.stable)
    }
    /// Whether time may skip ahead: everyone in the player's household is asleep or away at
    /// work, and nothing notable happened to them in the last game hour.
    pub fn calm(&self) -> bool {
        let home = |h: u32| self.households[h as usize].player;
        let mut members = self
            .sims
            .iter()
            .filter(|s| s.here() && home(s.household))
            .peekable();
        if members.peek().is_none() {
            return false;
        }
        let quiet = members.all(|s| s.away_until.is_some() || s.asleep(&self.content));
        let hour_ago = self.tick.saturating_sub(TICKS_PER_HOUR);
        quiet
            && !self
                .events
                .iter()
                .rev()
                .take_while(|e| e.tick >= hour_ago)
                .any(|e| {
                    e.kind.importance() >= 1
                        && [Some(e.a), Some(e.b), e.c]
                            .into_iter()
                            .flatten()
                            .any(|s| self.sims.get(s as usize).is_some_and(|s| home(s.household)))
                })
    }
    /// The navigation version: bumped whenever something walkers can or can't step through
    /// changes (see `nav_changed_in`).
    pub fn nav_version(&self) -> u32 {
        self.nav_version
    }
    /// Whether walking may have changed within tiles `[x0, z0, x1, z1)` (stacked rows, as
    /// positions are) since navigation version `seen`: a change there was logged since, or the
    /// log (`path::NAV_LOG` changes) doesn't go back that far. A walker whose way lies outside
    /// every change since it planned it can keep it.
    pub fn nav_changed_since(&self, seen: u32, rect: [i32; 4]) -> bool {
        self.nav_log.changed_since(seen, self.nav_version, rect)
    }
}
