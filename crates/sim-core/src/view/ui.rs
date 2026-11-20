//! [(61, 70), (616, 1104), (2661, 2908)]

use super::*;

/// Events in the UI state (the whole log is fetched with `events_json`).
const UI_EVENTS: usize = 30;

/// The whole story log, oldest first (JSON array).
pub fn events_json(world: &World) -> String {
    let events: Vec<EventView> = world.events.iter().map(EventView::from).collect();
    serde_json::to_string(&events).expect("events serialize")
}

impl List {
    fn next(&mut self) -> &mut Vec<u8> {
        if self.len > 0 {
            self.out.push(b',');
        }
        self.len += 1;
        &mut self.out
    }

    fn push<T: Serialize + ?Sized>(&mut self, item: &T) {
        let out = self.next();
        serde_json::to_writer(out, item).expect("UI view serializes");
    }

    /// Adds the item only if its JSON differs from what `sent` remembers (and remembers it).
    fn push_changed<T: Serialize>(&mut self, item: &T, sent: &mut u64) {
        let (mark, len) = (self.out.len(), self.len);
        let out = self.next();
        let start = out.len();
        serde_json::to_writer(&mut *out, item).expect("UI view serializes");
        let print = fingerprint(&out[start..]);
        if print == *sent {
            self.out.truncate(mark);
            self.len = len;
        } else {
            *sent = print;
        }
    }
}

/// What the UI was last sent, so that each UI update (~10 Hz) carries only what changed:
/// residents, households, relationship pairs and rooms are keyed by id and sent again when
/// their JSON changes, story events by id. Values that drift every tick are rounded to what the
/// UI shows (`round_to`), so changed means visibly changed. The first update, and the next
/// one after `resync`, sends everything.
///
/// Every resident is listed, but only the player's household and the resident being inspected
/// come with their details (needs, feelings, actions, skills, plan) and relationships: nobody
/// else's are shown, and these are what change all the time.
pub struct UiSync {
    /// The next update sends everything.
    pub full: bool,
    pub inspected: Option<u32>,
    /// Per resident slot: what they were sent (None: not listed).
    pub(crate) sims: Vec<Option<SimSent>>,
    pub households: Vec<u64>,
    /// Relationship pairs and rooms sent, with the update that last found them.
    pub relationships: HashMap<(u32, u32), (u64, u32)>,
    pub rooms: HashMap<u16, (u64, u32)>,
    pub updates: u32,
    /// The newest story event sent.
    pub last_event: Option<u64>,
}

/// Fingerprints of a resident's fields as last sent (0: not sent; the UI has its default).
#[derive(Default)]
pub(crate) struct SimSent {
    pub detailed: bool,
    pub detail: u64,
    pub name: u64,
    pub household: u64,
    pub traits: u64,
    pub perks: u64,
    pub age: u64,
    pub stage: u64,
    pub pension: u64,
    pub plot: u64,
    pub away_until: u64,
    pub visiting: u64,
    pub trip: u64,
    pub ride: u64,
    pub taxi: u64,
    pub job: u64,
    /// Reset when the details stop (the UI drops them).
    pub(crate) details: DetailsSent,
}

#[derive(Default)]
pub(crate) struct DetailsSent {
    pub grade: u64,
    pub needs: u64,
    pub mood: u64,
    pub emotion: u64,
    pub feelings: u64,
    pub actions: u64,
    pub skills: u64,
    pub phone: u64,
    pub homebound: u64,
    pub plan: u64,
    pub(crate) plan_parts: PlanSent,
}

/// The plan is sent part by part: the week's history is long, and the rest changes all day.
#[derive(Default)]
pub(crate) struct PlanSent {
    pub routines: u64,
    pub skip_household: u64,
    pub current: u64,
    pub history: u64,
    pub goals: u64,
    pub suggestions: u64,
    pub wishes: u64,
    pub home_wishes: u64,
    pub adherence: u64,
    pub purchase: u64,
}

impl Default for UiSync {
    fn default() -> Self {
        Self::new()
    }
}

impl UiSync {
    pub fn new() -> Self {
        Self {
            full: true,
            inspected: None,
            sims: Vec::new(),
            households: Vec::new(),
            relationships: HashMap::new(),
            rooms: HashMap::new(),
            updates: 0,
            last_event: None,
        }
    }

    /// The next update sends everything (the UI starts over, or lost track).
    pub fn resync(&mut self) {
        self.full = true;
    }

    /// The resident whose panel is open: their details and relationships are sent too.
    pub fn inspect(&mut self, sim: Option<u32>) {
        self.inspected = sim;
    }

    /// The next UI update (JSON; `UiUpdate` in `protocol.ts`): what changed since the last one,
    /// or everything.
    pub fn update_json(&mut self, world: &World) -> String {
        let full = std::mem::take(&mut self.full);
        if full {
            *self = Self {
                full: false,
                inspected: self.inspected,
                updates: self.updates,
                ..Self::new()
            };
        }
        self.updates = self.updates.wrapping_add(1);
        let content = &world.content;
        let detailed: Vec<bool> = world
            .sims
            .iter()
            .map(|s| {
                s.here()
                    && (world.households[s.household as usize].player
                        || self.inspected == Some(s.id))
            })
            .collect();

        // (Who is in a car on the way home, `trip_view`: found once, not for everyone away.)
        let mut riders: Vec<u32> = world
            .cars
            .iter()
            .filter(|c| c.state == crate::cars::CarState::Arriving)
            .flat_map(|c| c.driver.iter().chain(&c.aboard).chain(&c.done))
            .copied()
            .collect();
        riders.sort_unstable();

        let (mut sims, mut gone) = (List::default(), List::default());
        if self.sims.len() < world.sims.len() {
            self.sims.resize_with(world.sims.len(), || None);
        }
        for (i, s) in world.sims.iter().enumerate() {
            if !s.here() {
                if self.sims[i].take().is_some() {
                    gone.push(&s.id);
                }
                continue;
            }
            let sent = self.sims[i].get_or_insert_with(SimSent::default);
            let mark = (sims.out.len(), sims.len);
            let mut o = Obj::open(sims.next());
            o.field("id", &s.id);
            sim_patch(world, s, detailed[i], &riders, sent, &mut o);
            if o.close() == 1 {
                sims.out.truncate(mark.0);
                sims.len = mark.1;
            }
        }

        let mut households = List::default();
        self.households.resize(world.households.len(), 0);
        for (i, h) in world.households.iter().enumerate() {
            let costs = life::weekly_costs(world, i);
            let view = FundsView {
                id: h.id,
                funds: h.funds,
                rent: costs.map(|c| c.0),
                bills: costs.map(|c| c.1),
                undo: world.undo_steps(i),
                redo: world.redo_steps(i),
                routines: h.player.then(|| {
                    h.routines
                        .iter()
                        .map(|r| routine_view(content, r))
                        .collect()
                }),
                expecting: h.expecting,
            };
            households.push_changed(&view, &mut self.households[i]);
        }

        // Pairs with a resident whose details are sent, both ways (nobody else's are shown).
        let (mut relationships, mut rels_gone) = (List::default(), List::default());
        let rels = &world.relationships;
        let updates = self.updates;
        let mut visit = |a: usize, b: usize| {
            let r = rels.get(a, b);
            if !r.met {
                return;
            }
            let chemistry = rels.chemistry(a, b);
            let mut h = DefaultHasher::new();
            for bits in [
                r.friendship.to_bits(),
                r.romance.to_bits(),
                chemistry.to_bits(),
                u32::from(r.partners),
                r.kin as u32,
            ] {
                h.write_u32(bits);
            }
            let print = h.finish() | 1;
            match self.relationships.entry((a as u32, b as u32)) {
                Entry::Occupied(mut e) => {
                    let (sent, seen) = e.get_mut();
                    *seen = updates;
                    if *sent == print {
                        return;
                    }
                    *sent = print;
                }
                Entry::Vacant(e) => {
                    e.insert((print, updates));
                }
            }
            relationships.push(&RelView {
                a: a as u32,
                b: b as u32,
                friendship: r.friendship,
                romance: r.romance,
                partners: r.partners,
                chemistry,
                kin: r.kin,
            });
        };
        for a in (0..world.sims.len()).filter(|&a| detailed[a]) {
            for b in (0..world.sims.len()).filter(|&b| b != a) {
                visit(a, b);
                if !detailed[b] {
                    visit(b, a);
                }
            }
        }
        self.relationships.retain(|&(a, b), &mut (_, seen)| {
            let kept = seen == updates;
            if !kept {
                rels_gone.push(&[a, b]);
            }
            kept
        });

        let newest = self.last_event;
        let mut events = List::default();
        let fresh: Vec<&SocialEvent> = if full {
            let skip = world.events.iter().count().saturating_sub(UI_EVENTS);
            world.events.iter().skip(skip).collect()
        } else {
            let mut fresh: Vec<_> = world
                .events
                .iter()
                .rev()
                .take_while(|e| newest.is_none_or(|n| e.id > n))
                .collect();
            fresh.reverse();
            fresh
        };
        for e in fresh {
            events.push(&EventView::from(e));
        }
        self.last_event = world.events.iter().next_back().map(|e| e.id).or(newest);

        let (mut rooms, mut rooms_gone) = (List::default(), List::default());
        let home = world
            .households
            .iter()
            .find(|h| h.player)
            .and_then(|h| h.plot);
        for r in world
            .rooms()
            .iter()
            .filter(|r| home.is_some() && r.plot == home)
        {
            let s = r.scores;
            let view = RoomView {
                id: r.id,
                garden: r.garden,
                kind: r.kind,
                mixed: r.mixed,
                missing: r.missing,
                tiles: r.tiles,
                windows: r.windows,
                doors: r.doors,
                lamps: r.lamps,
                dirt: round_to(r.dirt, PERCENT),
                centre: r.centre,
                scores: [s.size, s.light, s.decor, s.clean, s.function, s.overall]
                    .map(|v| round_to(v, PERCENT)),
            };
            let (sent, seen) = self.rooms.entry(r.id).or_insert((0, updates));
            *seen = updates;
            rooms.push_changed(&view, sent);
        }
        self.rooms.retain(|&id, &mut (_, seen)| {
            let kept = seen == updates;
            if !kept {
                rooms_gone.push(&id);
            }
            kept
        });

        let day = clock::day(world.tick);
        let mut out =
            Vec::with_capacity(256 + sims.out.len() + relationships.out.len() + events.out.len());
        let mut o = Obj::open(&mut out);
        o.field("full", &full);
        o.field("day", &day);
        o.field("weekday", &clock::weekday(day));
        o.field("minute", &clock::minute_of_day(world.tick));
        o.field("speed", &world.speed);
        // (Quiet at home, `World::calm`: super speed runs up to 60× now, else up to 30×.)
        o.field("quiet", &world.calm());
        o.field("autonomy", &world.autonomy);
        o.field("lifespan", &world.lifespan);
        o.field("playerMoves", &world.player_moves);
        o.list("sims", &sims, full);
        o.list("simsGone", &gone, false);
        o.list("households", &households, full);
        o.list("relationships", &relationships, full);
        o.list("relationshipsGone", &rels_gone, false);
        o.list("events", &events, full);
        o.list("rooms", &rooms, full);
        o.list("roomsGone", &rooms_gone, false);
        o.close();
        String::from_utf8(out).expect("JSON is UTF-8")
    }
}

/// Everything the UI is sent at the start (a fresh `UiSync`'s first update).
pub fn ui_state_json(world: &World) -> String {
    UiSync::new().update_json(world)
}

/// Writes the fields of resident `s` that changed (`SimView` in `protocol.ts`); with `detail`
/// false only the summary (who, where, which job), the details being the UI's defaults.
fn sim_patch(
    world: &World,
    s: &Sim,
    detail: bool,
    riders: &[u32],
    sent: &mut SimSent,
    o: &mut Obj,
) {
    let content = &world.content;
    if sent.detailed && !detail {
        sent.details = DetailsSent::default();
    }
    sent.detailed = detail;
    o.changed("detail", &detail, &mut sent.detail);
    o.changed("name", &s.name, &mut sent.name);
    o.changed("household", &s.household, &mut sent.household);
    o.changed("traits", &s.traits, &mut sent.traits);
    o.changed("perks", &s.perks, &mut sent.perks);
    // Age in whole years, and the life stage (content `life.stages` id).
    o.changed("age", &(s.age.floor() as u32), &mut sent.age);
    let stage = content
        .life
        .stages
        .get(content.life.stage(s.age))
        .map(|st| st.id.as_str());
    o.changed("stage", &stage, &mut sent.stage);
    // Retired, with this weekly pension.
    o.changed_opt(
        "pension",
        s.retired.then_some(&s.pension),
        &mut sent.pension,
    );
    // Plot the Sim is on (None on the street or at work); minute of day they're back from work.
    // (In the flats' stair hall over a public place, or on its landings, on none: the hall is
    // the flats', not the place's, and the place isn't opened for one walking through it.)
    let (x, z) = s.tile();
    let plot = if s.away_until.is_some() {
        None
    } else {
        world
            .plot_at(x, z)
            .filter(|&p| world.halls.at(p, x, world.ground_z(z)).is_none())
    };
    o.changed("plot", &plot, &mut sent.plot);
    o.changed(
        "awayUntil",
        &s.away_until.map(clock::minute_of_day),
        &mut sent.away_until,
    );
    o.changed("visiting", &s.visiting.map(|v| v.plot), &mut sent.visiting);
    // (The time only with the details: it's a search along the roads, for a handful, not the town.)
    let trip = trip_view(world, s, riders, detail);
    o.changed_opt("trip", trip.as_ref(), &mut sent.trip);
    o.changed_opt("ride", ride_view(world, s).as_ref(), &mut sent.ride);
    // The work taxi: waiting for it, in it to work (and when they're there), or home in it.
    o.changed_opt("taxi", taxi_view(world, s).as_ref(), &mut sent.taxi);
    o.changed("job", &job_view(content, s), &mut sent.job);
    if !detail {
        return;
    }

    let d = &mut sent.details;
    // A pupil's school grade (0..100).
    let grade = content
        .life
        .stage_at(s.age)
        .filter(|st| st.school && content.life.school.is_some())
        .map(|_| round_to(s.grade, 10.0));
    o.changed_opt("grade", grade.as_ref(), &mut d.grade);
    let needs: Vec<f32> = s.needs[..content.needs.len()]
        .iter()
        .map(|&v| round_to(v, PERCENT))
        .collect();
    o.changed("needs", &needs, &mut d.needs);
    o.changed("mood", &round_to(s.mood(content), PERCENT), &mut d.mood);
    o.changed(
        "emotion",
        &s.emotion(content).map(|e| content.emotions[e].id.as_str()),
        &mut d.emotion,
    );
    o.changed("feelings", &feelings(world, s), &mut d.feelings);
    o.changed("actions", &actions(world, s), &mut d.actions);
    // Skill levels in content order (rounded down: the whole number is the level).
    let skills: Vec<f32> = s.skills[..content.skills.len()]
        .iter()
        .map(|&v| floor_to(v, PERCENT))
        .collect();
    o.changed("skills", &skills, &mut d.skills);
    o.changed_opt("phone", phone_view(world, s).as_ref(), &mut d.phone);
    // Why they can't be sent out (visiting) right now, as the command handler decides.
    o.changed_opt(
        "homebound",
        crate::life::homebound(world, s.id as usize).as_ref(),
        &mut d.homebound,
    );

    // Routines, goals and wishes (the player's household only), part by part.
    if !world.households[s.household as usize].player {
        o.changed_opt::<()>("plan", None, &mut d.plan);
        d.plan_parts = PlanSent::default();
        return;
    }
    d.plan = 1;
    let plan = plan_view(world, s);
    let p = &mut d.plan_parts;
    let (mark, fields) = (o.out.len(), o.fields);
    o.key("plan");
    let mut parts = Obj::open(&mut *o.out);
    parts.changed("routines", &plan.routines, &mut p.routines);
    parts.changed("skipHousehold", &plan.skip_household, &mut p.skip_household);
    parts.changed("current", &plan.current, &mut p.current);
    parts.changed("history", &plan.history, &mut p.history);
    parts.changed("goals", &plan.goals, &mut p.goals);
    parts.changed("suggestions", &plan.suggestions, &mut p.suggestions);
    parts.changed("wishes", &plan.wishes, &mut p.wishes);
    parts.changed("homeWishes", &plan.home_wishes, &mut p.home_wishes);
    parts.changed("adherence", &plan.adherence, &mut p.adherence);
    parts.changed("purchase", &plan.purchase, &mut p.purchase);
    if parts.close() == 0 {
        o.out.truncate(mark);
        o.fields = fields;
    }
}

/// The world structure as updates for a view that keeps what it was sent (the sim worker's
/// `world` messages): the first update has everything; later ones only what changed since the
/// last (`structure_json`'s parts by name, `unset` naming those now left out). The things: the
/// whole list the first time, then `objectSplice` (`[start, deleted, [things]]`, as
/// `Array.splice`: the things bought, sold or set up, between the things the list starts and
/// ends with as last sent) and `objectChanges`, the other things that changed, by their index in
/// the list after the splice. `meta`, which the simulation carries unchanged, comes only with
/// everything. The rooms never come (`Lot::rooms_row_major` goes as a typed array); the other lot
/// parts only when asked for (`lot`: the lot changed, see `lotVersion`), and then only those that
/// changed (a fence built: the fences).
///
/// A big world has some 19 000 things and a quarter of a megabyte of presentation data (`meta`),
/// and its structure changes with every door opened or crop grown: sent whole, that was 2 MB to
/// parse each time on the main thread (and a thing bought sent the whole list again). Each thing
/// is still written out to see whether it changed (the world doesn't say which did).
#[derive(Default)]
pub struct StructureSync {
    /// Fingerprints of the parts as last sent, by name (left out: not sent, or unset since).
    pub parts: HashMap<&'static str, u64>,
    /// Each thing as last sent, in order: its stable id and fingerprint.
    pub objects: Vec<(u32, u64)>,
    /// The plots' part as last made, with a fingerprint of what it was made from and of it:
    /// their houses' bounds scan every plot's tiles, and only the lot (or the plots themselves)
    /// changes them.
    pub plots: Option<(u64, u64, String)>,
    /// The things' lifts onto surfaces (`surface_lifts`, a lot-sized scan) as last found, for
    /// the objects' and the lot's versions.
    pub lifts: Option<((u32, u32), Lifts)>,
}

impl StructureSync {
    /// The next update (JSON): what changed since the last; with `lot`, the lot's walls, openings,
    /// diagonals, fences and floors too.
    pub fn update_json(&mut self, world: &World, lot: bool) -> String {
        use std::io::Write;
        // (Nothing sent yet: `mode` and the like always go.)
        let fresh = self.parts.is_empty();
        let mut parts = Parts {
            sent: &mut self.parts,
            out: Vec::with_capacity(1024),
            buf: Vec::new(),
            unset: Vec::new(),
        };
        let _ = write!(
            parts.out,
            "{{\"version\":{},\"lotVersion\":{}",
            world.structure_version(),
            world.lot_version()
        );
        parts.part("mode", Some(&world.mode));
        parts.part("width", Some(&world.lot.width));
        parts.part("depth", Some(&world.lot.depth));
        parts.part("storeys", Some(&world.storeys));
        parts.part("storeyDepth", Some(&world.storey_depth()));
        parts.part("basement", basement_block(world).as_ref());
        parts.part("sims", Some(&sim_infos(world)));
        parts.part("former", Some(&world.former).filter(|f| !f.is_empty()));
        parts.part("households", Some(&household_views(world)));
        let (print, plots) = plots_json(&mut self.plots, world);
        parts.put("plots", Some(print), plots.as_bytes());
        parts.part("exits", Some(&world.exits));
        // (What the simulation carries unchanged once loaded: sent the first time only. The
        // town's bays, car parks and flats are as its file laid them.)
        if fresh {
            parts.part("meta", Some(&world.meta));
            let parking = parking_views(world);
            parts.part("parking", Some(&parking).filter(|p| !p.is_empty()));
            let car_parks = &world.travel.file.car_parks;
            parts.part("carParks", Some(car_parks).filter(|c| !c.is_empty()));
            let flats = flat_views(world);
            parts.part("flats", Some(&flats).filter(|f| !f.is_empty()));
        }
        let staff = staff_views(world);
        parts.part("staff", Some(&staff).filter(|s| !s.is_empty()));
        parts.part("cars", car_views(world).as_ref());
        parts.part("garages", Some(&world.garages).filter(|g| !g.is_empty()));
        parts.part("porches", Some(&world.porches).filter(|p| !p.is_empty()));
        parts.part(
            "porchPieces",
            Some(&world.porch_pieces).filter(|p| !p.is_empty()),
        );
        let streets = pull_over_street_views(world);
        parts.part("pullOverStreets", Some(&streets).filter(|s| !s.is_empty()));
        let poles = light_pole_views(world);
        parts.part("lightPoles", Some(&poles).filter(|p| !p.is_empty()));
        let junctions = junction_views(world);
        parts.part("junctions", Some(&junctions).filter(|j| !j.is_empty()));
        // The lot's parts, those that changed (asked for only when the lot did; never unset).
        if lot {
            let (walls, openings, fences) = wall_edges(&world.lot, world.lot.walls());
            parts.part("walls", Some(&walls));
            parts.part("openings", Some(&openings));
            parts.part("fences", Some(&fences));
            parts.part(
                "diagonals",
                Some(&diagonals(&world.lot, world.lot.diagonal_walls())),
            );
            parts.part("floors", Some(&floor_views(world)));
        }

        // The things: those that changed, by index, while the list is the same things in the same
        // order (a door opened, a crop grown); else the whole list (one bought, sold or moved in).
        let key = (world.objects_version, world.lot_version());
        let lifts = match self.lifts.take() {
            Some((k, lifts)) if k == key => lifts,
            _ => surface_lifts(world),
        };
        let Parts {
            mut out,
            mut buf,
            unset,
            ..
        } = parts;
        // The things the list starts and ends with as last sent (by stable id); those between are
        // new (bought, set up), gone (sold) or reordered: they go as a splice (`objectSplice`, the
        // whole list the first time), the rest by index as they changed.
        let (start, end) = if fresh {
            self.objects.clear();
            (0, 0)
        } else {
            let start = self
                .objects
                .iter()
                .zip(&world.objects)
                .take_while(|(s, o)| s.0 == o.stable)
                .count();
            let end = self.objects[start..]
                .iter()
                .rev()
                .zip(world.objects[start..].iter().rev())
                .take_while(|(s, o)| s.0 == o.stable)
                .count();
            (start, end)
        };
        let spliced = start..world.objects.len() - end;
        let sent_spliced = start..self.objects.len() - end;
        let splice = fresh || !spliced.is_empty() || !sent_spliced.is_empty();
        if fresh {
            out.extend_from_slice(b",\"objects\":[");
        } else if splice {
            let _ = write!(out, ",\"objectSplice\":[{start},{},[", sent_spliced.len());
        }
        let new = if fresh {
            0..world.objects.len()
        } else {
            spliced.clone()
        };
        let mut news = Vec::with_capacity(new.len());
        for o in &world.objects[new] {
            buf.clear();
            serde_json::to_writer(&mut buf, &object_view(world, &lifts, o))
                .expect("object serializes");
            if !news.is_empty() {
                out.push(b',');
            }
            out.extend_from_slice(&buf);
            news.push((o.stable, fingerprint(&buf)));
        }
        if splice {
            out.extend_from_slice(if fresh { b"]" } else { b"]]" });
            self.objects.splice(sent_spliced, news);
        }
        // (Those outside the splice: at the same index in the list as it is now.)
        let mut changed = 0;
        for (i, o) in world.objects.iter().enumerate() {
            if fresh || spliced.contains(&i) {
                continue;
            }
            buf.clear();
            serde_json::to_writer(&mut buf, &object_view(world, &lifts, o))
                .expect("object serializes");
            let p = fingerprint(&buf);
            let sent = &mut self.objects[i].1;
            if *sent == p {
                continue;
            }
            *sent = p;
            out.extend_from_slice(if changed == 0 {
                b",\"objectChanges\":["
            } else {
                b","
            });
            changed += 1;
            let _ = write!(out, "[{i},");
            out.extend_from_slice(&buf);
            out.push(b']');
        }
        if changed > 0 {
            out.push(b']');
        }
        self.lifts = Some((key, lifts));
        fn field<T: Serialize + ?Sized>(out: &mut Vec<u8>, key: &str, v: &T) {
            let _ = write!(out, ",\"{key}\":");
            serde_json::to_writer(out, v).expect("structure part serializes");
        }

        if !unset.is_empty() {
            field(&mut out, "unset", &unset);
        }
        out.push(b'}');
        String::from_utf8(out).expect("JSON is UTF-8")
    }
}

/// `StructureSync::update_json`'s parts as it writes them: each only if it changed since it was
/// last sent (`sent`).
pub(crate) struct Parts<'a> {
    pub sent: &'a mut HashMap<&'static str, u64>,
    pub out: Vec<u8>,
    /// A part as it is now, to see whether it changed.
    pub buf: Vec<u8>,
    /// The parts left out now that were sent before.
    pub unset: Vec<&'static str>,
}

impl Parts<'_> {
    /// Part `key` (`None`: left out).
    fn part<T: Serialize + ?Sized>(&mut self, key: &'static str, v: Option<&T>) {
        let mut buf = std::mem::take(&mut self.buf);
        buf.clear();
        let print = v.map(|v| {
            serde_json::to_writer(&mut buf, v).expect("structure part serializes");
            fingerprint(&buf)
        });
        self.put(key, print, &buf);
        self.buf = buf;
    }

    /// Part `key` as JSON with its fingerprint (`None`: left out).
    fn put(&mut self, key: &'static str, print: Option<u64>, json: &[u8]) {
        match print {
            Some(p) => {
                if self.sent.insert(key, p) != Some(p) {
                    self.out.extend_from_slice(b",\"");
                    self.out.extend_from_slice(key.as_bytes());
                    self.out.extend_from_slice(b"\":");
                    self.out.extend_from_slice(json);
                }
            }
            None => {
                if self.sent.remove(key).is_some() {
                    self.unset.push(key);
                }
            }
        }
    }
}

/// The plots' part (JSON) and its fingerprint, made again only when the lot or a plot changed
/// (`StructureSync::plots`).
fn plots_json<'a>(cached: &'a mut Option<(u64, u64, String)>, world: &World) -> (u64, &'a str) {
    use std::hash::Hash;
    let mut h = DefaultHasher::new();
    (world.lot_version(), world.storeys, world.storey_depth()).hash(&mut h);
    for p in &world.plots {
        (p.id, &p.name, p.x, p.z, p.w, p.d, p.public).hash(&mut h);
        p.entry.map(|e| e.map(f32::to_bits)).hash(&mut h);
        p.roof.map(|r| [r.style, r.color]).hash(&mut h);
        p.hours.hash(&mut h);
    }
    let key = h.finish();
    if cached.as_ref().is_none_or(|c| c.0 != key) {
        let json = serde_json::to_string(&plot_views(world)).expect("plots serialize");
        *cached = Some((key, fingerprint(json.as_bytes()), json));
    }
    let (_, print, json) = cached.as_ref().expect("made just now");
    (*print, json)
}
