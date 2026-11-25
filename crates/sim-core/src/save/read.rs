//! Reading a save back: `World::from_save`.

use super::pack::*;
use super::*;

impl World {
    pub fn from_save_json(content_json: &str, save_json: &str) -> Result<Self, Error> {
        let content = Content::from_json(content_json)?;
        let save: SaveFile = serde_json::from_str(save_json)?;
        Self::from_save(content, &save)
    }

    pub fn from_save(content: Content, save: &SaveFile) -> Result<Self, Error> {
        if save.version < OLDEST_SAVE_VERSION || save.version > SAVE_VERSION {
            return Err(Error::new(format!(
                "unsupported save version {} (this build opens {OLDEST_SAVE_VERSION} to {SAVE_VERSION})",
                save.version
            )));
        }
        // Storeys before the furniture: what may stand upstairs depends on them.
        let storeys = save.storeys.unwrap_or(1).max(1);
        // The basement's block comes after the ground's, at most one past the storeys above.
        let basement = save.basement.filter(|&b| b > 0 && b <= storeys);
        let blocks = storeys as usize + usize::from(basement.is_some());
        let stacked = blocks > 1 && save.lot.depth.is_multiple_of(blocks);
        let ground = if stacked {
            save.lot.depth / blocks
        } else {
            save.lot.depth
        };
        let mut lot = Lot::from_edges(
            save.lot.width,
            save.lot.depth,
            ground,
            decode(&expand(&save.lot.h_edges)?)?,
            decode(&expand(&save.lot.v_edges)?)?,
            decode_diagonals(&expand(&save.lot.diagonals)?)?,
        )?;
        decode_looks(&mut lot, &content, &save.lot.looks);
        decode_floors(&mut lot, &content, &save.lot.floors);
        let mut world = World::empty(content, lot, Rng::new(save.rng));
        if stacked {
            world.storeys = storeys;
            world.stairs.depth = (world.lot.depth / blocks) as i32;
            world.stairs.basement = basement.map_or(0, i32::from);
        }
        world.mode = save.mode;
        world.seed = save.seed;
        world.tick = save.tick;
        let speed = save.speed.min(MAX_SPEED);
        world.speed = speed;
        world.auto_super = save.auto_super && speed == SUPER_SPEED;
        world.resume_speed = save
            .resume_speed
            .unwrap_or(if (1..SUPER_SPEED).contains(&speed) {
                speed
            } else {
                1
            })
            .clamp(1, SUPER_SPEED - 1);
        world.autonomy = save.autonomy;
        world.lifespan = save.lifespan.unwrap_or(crate::lifecycle::Lifespan::Off);
        world.player_moves = save.player_moves.unwrap_or(true);
        world.residents_most_given = save.residents_most;
        world.meta = save.meta.clone();
        world.note_paths();
        world.exits = save.exits.clone();
        world.set_travel(save.open_world.clone());
        world.plots = save
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
                roof: p.roof.as_ref().and_then(|[style, color]| {
                    let b = &world.content.build;
                    let index = |list: &[crate::content::BuildStyle], id: &str| {
                        list.iter().position(|s| s.id == id).map(|i| i as u8)
                    };
                    // A style or colour the content no longer has falls back to the first.
                    let style = index(&b.roofs, style).unwrap_or(0);
                    let color = index(&b.roof_colors, color).unwrap_or(0);
                    (!b.roofs.is_empty()).then_some(crate::world::RoofLook { style, color })
                }),
                hours: p.hours,
                kind: save.open_world.plot_kinds.get(i).copied().flatten(),
                of: p.of,
                storey: p.storey,
                door: p.door.map(|[x, z]| (x, z)),
                inside: p.inside,
            })
            .collect();
        crate::world::check_flats(&world.plots, i32::from(world.storeys.max(1)), &world.stairs)
            .map_err(|e| Error::new(format!("saved plots: {}", e.0)))?;
        world.map_plots();
        let starting_funds = world.content.starting_funds;
        world.households = if save.households.is_empty() {
            vec![Household {
                id: 0,
                name: "Household".into(),
                plot: None,
                player: true,
                funds: starting_funds,
                free_will: save.autonomy,
                routines: Vec::new(),
                expecting: None,
                sitter_day: None,
                laundry: crate::world::LAUNDRY_START,
                letters: Vec::new(),
            }]
        } else {
            save.households
                .iter()
                .enumerate()
                .map(|(i, h)| Household {
                    id: i as u32,
                    name: h.name.clone(),
                    plot: h.plot,
                    player: h.player,
                    funds: h.funds.unwrap_or(starting_funds),
                    free_will: h.free_will.unwrap_or(true),
                    routines: h
                        .routines
                        .iter()
                        .filter_map(|r| routine_load(&world.content, r))
                        .collect(),
                    expecting: h.expecting,
                    sitter_day: h.sitter_day,
                    laundry: h.laundry.as_ref().map_or(crate::world::LAUNDRY_START, |l| {
                        std::array::from_fn(|k| {
                            finite(l.get(k).copied().unwrap_or(0.0), 0.0).max(0.0)
                        })
                    }),
                    letters: h.letters.clone(),
                })
                .collect()
        };
        if let Some(h) = world
            .households
            .iter()
            .find(|h| h.plot.is_some_and(|p| p as usize >= world.plots.len()))
        {
            return Err(Error::new(format!(
                "corrupt save: household '{}' lives on an unknown plot",
                h.name
            )));
        }

        // Object ids may change from the save's (an object left out, or tops placed last): tasks are remapped.
        // Things standing on tops go in once everything else stands.
        let mut object_ids = HashMap::new();
        // Saved stable ids are kept (once each); old saves and anything else get fresh ones, past
        // every saved id.
        world.next_object_id = save
            .objects
            .iter()
            .filter_map(|o| o.id)
            .max()
            .map_or(Some(0), |m| m.checked_add(1))
            .ok_or_else(|| Error::new("corrupt save: an object id is out of range"))?;
        let mut used = HashSet::new();
        let on_top = |o: &ObjectSave| {
            world
                .content
                .object_index(&o.def)
                .is_some_and(|d| world.content.objects[d].layer == crate::content::Layer::Surface)
        };
        let tops: Vec<bool> = save.objects.iter().map(on_top).collect();
        let order: Vec<usize> = (0..tops.len())
            .filter(|&i| !tops[i])
            .chain((0..tops.len()).filter(|&i| tops[i]))
            .collect();
        for i in order {
            let o = &save.objects[i];
            // (A corrupt save's second use of an id gets a fresh one.)
            let keep = o.id.filter(|s| !used.contains(s));
            let placed = match world.content.object_index(&o.def) {
                Some(def) => {
                    let value = o
                        .value
                        .unwrap_or(world.content.objects[def].price.unwrap_or(0));
                    let colors = crate::world::parse_colors(&o.colors, None);
                    // A size the content no longer allows goes back to its kind's.
                    let own = crate::world::kind_size(world.content.objects[def].footprint);
                    let (size, tall) = world
                        .check_size(def, o.size.unwrap_or(own), o.tall)
                        .unwrap_or((own, 0));
                    let layout = crate::world::Layout {
                        offset: o.offset,
                        extent: Some(size),
                        tall,
                    };
                    world.place(def, o.x, o.z, o.rot, layout, colors, o.quality, value, keep)
                }
                None => continue,
            };
            // What can't be placed is left out, and the rest of the game loads: a kind the content
            // no longer has, a thing whose top is gone, or one that doesn't fit where it was any
            // more (building can leave a ceiling lamp outdoors, or something upstairs outside a room).
            if let Ok(id) = placed {
                let obj = &world.objects[id as usize];
                let turn = world.check_turn(obj.def, o.turn).unwrap_or(0);
                let offset = world
                    .check_offset(obj.def, obj.rot, o.offset)
                    .unwrap_or([0, 0]);
                let obj = &mut world.objects[id as usize];
                obj.turn = turn;
                obj.offset = offset;
                obj.wear = o.wear.clamp(0.0, 1.0);
                obj.open = o.open;
                obj.off = o.off && world.content.objects[obj.def].switch;
                obj.shut = o.shut && world.content.objects[obj.def].over_window;
                obj.growth = finite(o.growth.unwrap_or(1.0), 1.0).clamp(-1.0, 1.0);
                let runs = world.content.objects[obj.def].runs;
                obj.run = match o.run {
                    Some(m) => runs.map(|full| finite(m, 0.0).clamp(0.0, full)),
                    None => None,
                };
                obj.rack = o.rack.min(world.content.objects[obj.def].rack);
                // (A thing kept by a plot it doesn't stand on: one of the plots, `World::keep`.)
                world.keep(id, o.plot);
                used.extend(keep);
                object_ids.insert(i as u32, id);
            }
        }
        world.next_object_id = world.next_object_id.max(save.next_object_id.unwrap_or(0));
        // Genders, traits and perks the content no longer has are dropped (an unknown gender is
        // the first one's; with no genders in the content any is kept).
        let content = &world.content;
        let gender =
            |g: &String| content.genders.is_empty() || content.genders.iter().any(|d| &d.id == g);
        let known = |ids: &[String], list: &[&String]| -> Vec<String> {
            ids.iter().filter(|id| list.contains(id)).cloned().collect()
        };
        let traits: Vec<&String> = content.traits.iter().map(|d| &d.id).collect();
        let perks: Vec<&String> = content.perks.iter().map(|d| &d.id).collect();
        let spawns: Vec<SimSpawn> = save
            .sims
            .iter()
            .map(|s| SimSpawn {
                name: s.name.clone(),
                gender: s.gender.clone().filter(|g| gender(g)),
                attracted_to: s
                    .attracted_to
                    .as_ref()
                    .map(|a| a.iter().filter(|g| gender(g)).cloned().collect()),
                household: s.household,
                job: None,
                appearance: s.appearance.clone(),
                traits: known(&s.traits, &traits),
                perks: known(&s.perks, &perks),
                age: s.age.filter(|a| a.is_finite()),
                skills: Default::default(),
                x: s.pos[0],
                z: s.pos[1],
            })
            .collect();
        for spawn in &spawns {
            world.spawn_any(spawn, false)?;
        }
        let n = world.sims.len();
        if let Some(h) = world.households.iter().find(|h| {
            h.expecting
                .is_some_and(|e| e.parents.iter().any(|&p| p as usize >= n))
        }) {
            return Err(Error::new(format!(
                "corrupt save: household '{}' expects a baby by someone who isn't in town",
                h.name
            )));
        }
        let plot_count = world.plots.len();
        let tick = world.tick;
        let after =
            |minutes: f32| tick.saturating_add(social::minutes_to_ticks(finite(minutes, 0.0)));
        for (id, s) in save.sims.iter().enumerate() {
            let content = &world.content;
            let sim = &mut world.sims[id];
            sim.yaw = finite(s.yaw, 0.0);
            sim.gone = s.gone;
            sim.retired = s.retired;
            sim.pension = s.pension.max(0);
            if let Some(g) = s.grade {
                sim.grade = g.clamp(0.0, 100.0);
            }
            sim.outfit = s.outfit % crate::world::OUTFITS;
            // (Without a school day saved, once today's school has begun it counts as seen to, or
            // the load would mark the day missed.)
            sim.school_day = match s.school_day {
                Some(day) => day,
                _ => match crate::life::school_today(content, sim, tick) {
                    Some((_, start, _)) if tick > start => crate::clock::day(tick),
                    _ => 0,
                },
            };
            for (i, need) in content.needs.iter().enumerate() {
                sim.needs[i] = s
                    .needs
                    .get(&need.id)
                    .copied()
                    .unwrap_or(0.75)
                    .clamp(0.0, 1.0);
            }
            for m in &s.feelings {
                if let Some(def) = content.feeling_index(&m.id) {
                    let expires = after(m.minutes_left.min(content.feelings[def].minutes));
                    sim.feelings.push(social::ActiveFeeling { def, expires });
                }
            }
            for (a, def) in content.accidents.iter().enumerate() {
                if let Some(&m) = s.accidents.get(&def.id) {
                    sim.accident_at[a] = after(m).min(tick.saturating_add(def.cooldown_ticks));
                }
            }
            for (i, skill) in content.skills.iter().enumerate() {
                if let Some(&v) = s.skills.get(&skill.id) {
                    sim.skills[i] = v.clamp(0.0, content.skill_rules.max_level);
                }
            }
            sim.job = s.job.as_ref().and_then(|j| {
                let career = content.career_index(&j.career)?;
                let level = j.level.min(content.careers[career].levels.len() - 1);
                let mut job = Job::new(career, level, j.last_shift_day);
                job.performance = j.performance;
                job.satisfaction = j.satisfaction.unwrap_or(job.satisfaction);
                job.shifts = j.shifts;
                job.missed = j.missed;
                job.shift_mood = j.shift_mood.unwrap_or(job.shift_mood);
                job.late = j.late;
                Some(job)
            });
            sim.job_search_from = s.job_search_from;
            if s.nap_minutes > 0.0 {
                sim.nap_until = after(s.nap_minutes);
            }
            if let Some(p) = &s.planner {
                sim.planner = planner_load(content, p);
            }
            sim.held = s.held.as_ref().and_then(|h| {
                Some(Held {
                    def: content.item_index(&h.def)?,
                    servings: h.servings,
                    quality: h.quality,
                    made_at: h.made_at,
                    recipe: h.recipe.as_deref().and_then(|r| content.recipe_index(r)),
                })
            });
            sim.night_woken = s.night_woken;
            // At work, or a pupil at school.
            let pupil = content
                .life
                .stage_at(s.age.unwrap_or(30.0))
                .is_some_and(|st| st.school);
            sim.away_until = s
                .away_minutes
                .filter(|_| sim.job.is_some() || pupil)
                .map(after);
            sim.visiting = s.visiting.and_then(|(plot, left)| {
                ((plot as usize) < plot_count).then(|| Visit {
                    plot,
                    // Visits without an end (player visits before v8) last as long as a
                    // player visit does now.
                    until: after(left.unwrap_or(content.visits.directed_hours * 60.0)),
                    idle: 0,
                    farewell: 0,
                    offer_at: 0,
                    arrived: 0,
                    leaving: false,
                    home_by: false,
                })
            });
            for t in &s.tasks {
                let task = match t {
                    TaskSave::Use {
                        object,
                        interaction,
                        directed,
                        routine,
                    } => {
                        let Some(&object) = object_ids.get(object) else {
                            continue;
                        };
                        let def = &content.objects[world.objects[object as usize].def];
                        let Some(interaction) =
                            def.interactions.iter().position(|i| &i.id == interaction)
                        else {
                            continue;
                        };
                        Task {
                            kind: TaskKind::Use {
                                object,
                                interaction,
                            },
                            directed: *directed,
                            routine: *routine,
                        }
                    }
                    TaskSave::MoveTo { x, z, directed } => Task {
                        kind: TaskKind::MoveTo { x: *x, z: *z },
                        directed: *directed,
                        routine: false,
                    },
                    TaskSave::Social {
                        target,
                        social,
                        directed,
                    } => {
                        let Some(social) = content.social_index(social) else {
                            continue;
                        };
                        if *target as usize >= n || *target as usize == id {
                            continue;
                        }
                        Task {
                            kind: TaskKind::Social {
                                target: *target,
                                social,
                            },
                            directed: *directed,
                            routine: false,
                        }
                    }
                    TaskSave::Work => Task {
                        kind: TaskKind::Work,
                        directed: true,
                        routine: false,
                    },
                    TaskSave::Visit { plot, directed } => {
                        if *plot as usize >= plot_count {
                            continue;
                        }
                        Task {
                            kind: TaskKind::Visit { plot: *plot },
                            directed: *directed,
                            routine: false,
                        }
                    }
                    TaskSave::GoHome { directed } => Task {
                        kind: TaskKind::GoHome,
                        directed: *directed,
                        routine: false,
                    },
                    TaskSave::Clean { x, z, directed } => Task {
                        kind: TaskKind::Clean { x: *x, z: *z },
                        directed: *directed,
                        routine: false,
                    },
                    // (An accident's while on the spot doesn't outlast a reload.)
                    TaskSave::Spot { .. } => continue,
                    TaskSave::Repair { object, directed } => {
                        let Some(&object) = object_ids.get(object) else {
                            continue;
                        };
                        Task {
                            kind: TaskKind::Repair { object },
                            directed: *directed,
                            routine: false,
                        }
                    }
                    TaskSave::Cook {
                        recipe,
                        step,
                        origin,
                        servings,
                        skipped,
                        directed,
                    } => {
                        let Some(recipe) = content.recipe_index(recipe) else {
                            continue;
                        };
                        // (At most one past the last step: the dish on its way to the table.)
                        if *step as usize > content.recipes[recipe].steps.len() {
                            continue;
                        }
                        Task {
                            kind: TaskKind::Cook {
                                recipe,
                                step: *step,
                                station: None,
                                origin: origin.and_then(|o| object_ids.get(&o).copied()),
                                servings: *servings,
                                skipped: *skipped,
                                minutes: 0,
                            },
                            directed: *directed,
                            routine: false,
                        }
                    }
                };
                sim.queue.push_back(task);
            }
        }

        // (Since v35 a save whose pairs are all as new lists none, but keeps their chemistry.)
        if save.relationships.is_empty() && save.chemistry.is_empty() {
            // No relationships saved: start the household as housemates.
            world.init_relationships(&[])?;
        } else {
            for r in &save.relationships {
                let (a, b) = (r.a as usize, r.b as usize);
                if a < n && b < n && a != b {
                    let rel = Relationship {
                        friendship: r.rel.friendship.clamp(-100.0, 100.0),
                        romance: r.rel.romance.clamp(-100.0, 100.0),
                        ..r.rel
                    };
                    world.relationships.set(a, b, rel);
                }
            }
        }
        if !save.chemistry.is_empty() {
            let chemistry = unpack_chemistry(&save.chemistry, n)?;
            let mut k = 0;
            for a in 0..n {
                for b in a + 1..n {
                    world
                        .relationships
                        .set_chemistry(a, b, finite(chemistry[k], 0.0));
                    k += 1;
                }
            }
        }
        // Buffs from saved feelings apply straight away.
        for sim in &mut world.sims {
            sim.refresh_buffs(&world.content);
        }
        // Spawning consumed random numbers; restore the saved stream.
        world.rng = Rng::new(save.rng);
        // The story: events about residents who no longer exist (or of kinds this version
        // doesn't know) are dropped.
        let content = &world.content;
        let events = save.events.iter().filter_map(|v| {
            let mut e: social::SocialEvent = serde_json::from_value(v.clone()).ok()?;
            if let Some(refs) = v.get("refs") {
                resolve_refs(content, &mut e, refs)?;
            }
            let formers = save.former.len();
            let known = |s: u32| {
                if s & social::FORMER != 0 {
                    ((s & !social::FORMER) as usize) < formers
                } else {
                    (s as usize) < n
                }
            };
            (known(e.a) && known(e.b) && e.c.is_none_or(known)).then_some(e)
        });
        world.events = social::EventLog::restore(save.next_event_id, events);
        world.former = save.former.clone();
        world.dirt = vec![0.0; world.lot.tile_slots()];
        for &[x, z, d] in &save.dirt {
            let (x, z) = (x as i32, z as i32);
            if world.lot.in_bounds(x, z) {
                let i = world.tile_slot_mut(x, z);
                world.dirt[i] = (f32::from(d) / 255.0).min(1.0);
            }
        }
        let households = world.households.len();
        let content = &world.content;
        world.items.list = save
            .items
            .iter()
            .filter_map(|it| {
                let at = match it.at.as_str() {
                    "surface" => ItemAt::Surface {
                        x: it.x,
                        z: it.z,
                        offset: it.offset,
                    },
                    "stored" => ItemAt::Stored { x: it.x, z: it.z },
                    "floor" => ItemAt::Floor {
                        x: it.x,
                        z: it.z,
                        offset: it.offset,
                    },
                    _ => return None,
                };
                ((it.household as usize) < households).then_some(())?;
                Some(ItemInstance {
                    id: it.id,
                    def: content.item_index(&it.def)?,
                    at,
                    servings: it.servings,
                    quality: it.quality,
                    made_at: it.made_at,
                    household: it.household,
                    spoiled: it.spoiled,
                    recipe: it.recipe.as_deref().and_then(|r| content.recipe_index(r)),
                })
            })
            .collect();
        world.items.list.sort_by_key(|it| it.id);
        // (Two with one id would lose one here without a word: a save from elsewhere may have
        // them.)
        if world.items.list.windows(2).any(|w| w[0].id == w[1].id) {
            return Err(Error::new("corrupt save: two items share an id"));
        }
        let after_last = match world.items.list.last() {
            Some(it) => it
                .id
                .checked_add(1)
                .ok_or_else(|| Error::new("corrupt save: an item id is out of range"))?,
            None => 0,
        };
        // (`Items::add` counts up from this, and ids from `PARCEL_IDS` up are the view's parcels:
        // a real item there would be drawn as one, and the top of the range would overflow.)
        let next_id = save.next_item.max(after_last);
        if next_id >= crate::setup::PARCEL_IDS {
            return Err(Error::new("corrupt save: the next item id is out of range"));
        }
        world.items.next_id = next_id;
        for s in &save.setups {
            let object = world.content.object_index(&s.object);
            let Some(h) = world.households.get_mut(s.household as usize) else {
                continue;
            };
            match object {
                Some(object) if (s.sim as usize) < world.sims.len() => world.setups.push(
                    crate::setup::Setup::new(s.household, s.sim, object, s.since, s.paid),
                ),
                // (Gone from the content, or a sim the save doesn't have: the money back. It was
                // paid at the purchase and the object is only placed when the setup is done.)
                _ => h.funds = h.funds.saturating_add(s.paid.max(0)),
            }
        }
        world.make_storeys();
        world.refresh_rooms();
        // The porches and the pieces, before the cars: a post's tile is kept as a garage's
        // (`cars::reblock`).
        let plots = world.plots.len();
        world.porches = save
            .porches
            .iter()
            .filter(|p| (p.plot as usize) < plots)
            .copied()
            .collect();
        world.porch_pieces = save
            .porch_pieces
            .iter()
            .filter(|p| (p.plot as usize) < plots && p.w > 0 && p.d > 0)
            .copied()
            .collect();
        crate::cars::load(&mut world, &save.cars, &save.drives, &save.garages);
        // (The junctions lit are worked out again from the roads: `traffic::settle`.)
        crate::traffic::settle(&mut world);
        // (The lights where they'd got to, for the same lit junctions: each a turn it has.)
        if save.lights.len() == world.travel.signals.len()
            && save
                .lights
                .iter()
                .zip(&world.travel.signals)
                .all(|(l, s)| l.phase < s.turns())
        {
            world.traffic.lights = save
                .lights
                .iter()
                .map(|l| crate::traffic::Light {
                    phase: l.phase,
                    stage: match l.stage {
                        1 => crate::traffic::Stage::Amber,
                        2 => crate::traffic::Stage::Clear,
                        _ => crate::traffic::Stage::Green,
                    },
                    since: l.since,
                })
                .collect();
        }
        // The work taxis' runs, their riders booked and the taxis under way where they were.
        world.taxis = save.taxis.clone();
        crate::taxi::settle(&mut world);
        // What someone was eating or washing up is put down (those don't go on after loading).
        crate::cooking::settle(&mut world);
        Ok(world)
    }
}

fn routine_load(content: &Content, r: &RoutineSave) -> Option<Routine> {
    let activity = content.activities.iter().position(|a| a.id == r.activity)?;
    Some(Routine {
        id: r.id,
        activity,
        skill: r.skill.as_deref().and_then(|s| content.skill_index(s)),
        days: r.days & 0x7f,
        start: r.start.min(1439),
        minutes: r.minutes,
    })
}

fn goal_load(content: &Content, g: &GoalSave) -> Option<Goal> {
    Some(Goal {
        def: content.goals.iter().position(|d| d.id == g.def)?,
        skill: match &g.skill {
            Some(s) => Some(content.skill_index(s)?),
            None => None,
        },
        category: g
            .category
            .as_deref()
            .and_then(|c| content.career_categories.iter().position(|d| d.id == c)),
        object: match &g.object {
            Some(o) => Some(content.object_index(o)?),
            None => None,
        },
        target: g.target,
        since_day: g.since_day,
        start: g.start,
        progress: g.progress,
        history: g.history,
    })
}

fn reason_load(content: &Content, r: &str) -> Option<Reason> {
    Some(match r.split_once(':') {
        Some(("trait", id)) => Reason::Trait(content.traits.iter().position(|t| t.id == id)?),
        Some(("need", id)) => Reason::Need(content.needs.iter().position(|n| n.id == id)?),
        _ => match r {
            "away" => Reason::Away,
            "noPlace" => Reason::NoPlace,
            _ => Reason::Mood,
        },
    })
}

fn planner_load(content: &Content, p: &PlannerSave) -> Planner {
    let activity = |id: &str| content.activities.iter().position(|a| a.id == id);
    Planner {
        routines: p
            .routines
            .iter()
            .filter_map(|r| routine_load(content, r))
            .collect(),
        skip_household: p.skip_household.clone(),
        // Older saves may hold the same goal twice: keep the first.
        goals: p.goals.iter().filter_map(|g| goal_load(content, g)).fold(
            Vec::new(),
            |mut goals: Vec<Goal>, g| {
                if !goals.iter().any(|o| o.same(&g)) {
                    goals.push(g);
                }
                goals
            },
        ),
        suggestions: p
            .suggestions
            .iter()
            .filter_map(|g| goal_load(content, g))
            .collect(),
        run: None,
        history: p
            .history
            .iter()
            .filter_map(|b| {
                Some(BlockResult {
                    routine: b.routine,
                    household: b.household,
                    activity: activity(&b.activity)?,
                    skill: b.skill.as_deref().and_then(|s| content.skill_index(s)),
                    day: b.day,
                    start: b.start,
                    minutes: b.minutes,
                    outcome: match b.outcome.as_str() {
                        "kept" => Outcome::Kept,
                        "cut" => Outcome::Cut,
                        "noPlace" => Outcome::NoPlace,
                        _ => Outcome::Skipped,
                    },
                    reason: b.reason.as_deref().and_then(|r| reason_load(content, r)),
                    done: b.done,
                })
            })
            .collect(),
        wishes: p
            .wishes
            .iter()
            .filter_map(|(a, s)| {
                Some((
                    activity(a)?,
                    s.as_deref().and_then(|s| content.skill_index(s)),
                ))
            })
            .collect(),
        home_wishes: p
            .home_wishes
            .iter()
            .filter_map(|w| home_wish_load(content, w))
            .collect(),
        waited_day: p.waited_day,
        thought: None,
        reviewed: p.reviewed,
        recheck: false,
        purchase: p.purchase.as_ref().and_then(|r| {
            Some(crate::planner::PurchaseRequest {
                object: content.object_index(&r.object)?,
                open: r.open,
                day: r.day,
                declines: r.declines,
            })
        }),
    }
}

/// Points a loaded event at the content it names (by the ids saved with it). None when the
/// content no longer has one of them: the event is dropped.
///
/// The content a story event names, by id (its indices change when content does): the object,
/// accident or life stage in `n`, the skill, the career (or career category, for goals) and the
/// goal.
fn resolve_refs(
    content: &Content,
    e: &mut social::SocialEvent,
    refs: &serde_json::Value,
) -> Option<()> {
    /// The index of the saved id for `key` among `ids`: Some(None) without one, None if gone.
    fn find<'a>(
        refs: &serde_json::Value,
        key: &str,
        mut ids: impl Iterator<Item = &'a String>,
    ) -> Option<Option<u32>> {
        match refs.get(key).and_then(|v| v.as_str()) {
            None => Some(None),
            Some(want) => ids.position(|i| i == want).map(|p| Some(p as u32)),
        }
    }
    let n = [
        find(refs, "object", content.objects.iter().map(|d| &d.id))?,
        find(refs, "accident", content.accidents.iter().map(|d| &d.id))?,
        find(refs, "stage", content.life.stages.iter().map(|d| &d.id))?,
        find(refs, "item", content.items.iter().map(|d| &d.id))?,
        find(refs, "car", content.car_bodies.iter().map(|d| &d.id))?,
    ];
    if let Some(i) = n.into_iter().flatten().next() {
        e.n = Some(i64::from(i));
    }
    if let Some(s) = find(refs, "skill", content.skills.iter().map(|d| &d.id))? {
        e.skill = Some(s);
    }
    if let Some(c) = find(refs, "career", content.careers.iter().map(|d| &d.id))? {
        e.career = Some(c);
    }
    if let Some(c) = find(
        refs,
        "category",
        content.career_categories.iter().map(|d| &d.id),
    )? {
        e.career = Some(c);
    }
    if let Some(g) = find(refs, "goal", content.goals.iter().map(|d| &d.id))? {
        e.goal = Some(g);
    }
    Some(())
}
