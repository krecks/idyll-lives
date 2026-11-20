//! Writing a save: `World::to_save` and the shapes it builds.
//!
//! Split from `save.rs` (see `mod.rs`).

use super::pack::*;
use super::*;

impl World {
    pub fn to_save(&self) -> SaveFile {
        let content = &self.content;
        let (h, v, diagonals) = self.lot.row_major_edges();
        let minutes_left = |until: u64| until.saturating_sub(self.tick) as f32 * MINUTES_PER_TICK;
        let objects = self
            .objects
            .iter()
            .map(|o| ObjectSave {
                id: Some(o.stable),
                def: content.objects[o.def].id.clone(),
                x: o.x,
                z: o.z,
                rot: o.rot,
                quality: o.quality,
                colors: crate::world::colors_hex(&o.colors),
                color: None,
                value: Some(o.value),
                turn: o.turn,
                offset: o.offset,
                wear: finite(o.wear, 0.0),
                size: (o.extent != crate::world::kind_size(content.objects[o.def].footprint))
                    .then_some(o.extent),
                tall: o.tall,
                open: o.open,
                growth: content.objects[o.def].grows.is_some().then_some(o.growth),
                off: o.off && content.objects[o.def].switch,
                shut: o.shut && content.objects[o.def].over_window,
                run: o.run.filter(|_| content.objects[o.def].runs.is_some()),
                rack: o.rack.min(content.objects[o.def].rack),
            })
            .collect();
        let sims = self
            .sims
            .iter()
            .map(|s| {
                let mut pos = s.pos;
                // A Sim on a bed or chair (or in the shower) stands back up where they came from.
                if (s.pose != Pose::Stand || s.seated_from.is_some())
                    && let Some(act) = s.current()
                    && let TaskKind::Use { object, .. }
                    | TaskKind::Eat {
                        seat: Some(object), ..
                    } = act.task.kind
                {
                    let (fx, fz) = s
                        .seated_from
                        .unwrap_or_else(|| self.approach_tile(&self.objects[object as usize]));
                    pos = [fx as f32 + 0.5, fz as f32 + 0.5];
                } else if let Some((fx, fz)) = s.seated_from {
                    // (Talking from their seat: up where they sat down from, as what they sat
                    // down to do isn't saved.)
                    pos = [fx as f32 + 0.5, fz as f32 + 0.5];
                }
                // (Lost somewhere: back at home's front door, or the middle of town.)
                if !pos.iter().all(|p| p.is_finite()) {
                    let home = self
                        .households
                        .get(s.household as usize)
                        .and_then(|h| h.plot);
                    pos = home.map_or(
                        [self.lot.width as f32 / 2.0, self.lot.depth as f32 / 2.0],
                        |p| self.plots[p as usize].arrival_point(),
                    );
                }
                let tasks = s
                    .current()
                    .map(|a| &a.task)
                    .into_iter()
                    .chain(s.queue())
                    .filter_map(|t| self.task_save(t))
                    .collect();
                let feelings = s
                    .feelings
                    .iter()
                    .map(|m| FeelingSave {
                        id: content.feelings[m.def].id.clone(),
                        minutes_left: minutes_left(m.expires),
                    })
                    .collect();
                SimSave {
                    name: s.name.clone(),
                    appearance: s.appearance.clone(),
                    gender: Some(s.gender.clone()),
                    attracted_to: Some(s.attracted_to.clone()),
                    household: s.household,
                    traits: s.traits.clone(),
                    perks: s.perks.clone(),
                    age: some_finite(s.age),
                    gone: s.gone,
                    retired: s.retired,
                    pension: s.pension,
                    grade: content
                        .life
                        .stage_at(s.age)
                        .is_some_and(|st| st.school)
                        .then_some(s.grade)
                        .and_then(some_finite),
                    outfit: s.outfit,
                    school_day: content
                        .life
                        .stage_at(s.age)
                        .is_some_and(|st| st.school)
                        .then_some(s.school_day),
                    pos,
                    yaw: finite(s.yaw, 0.0),
                    needs: content
                        .needs
                        .iter()
                        .enumerate()
                        .map(|(i, n)| (n.id.clone(), finite(s.needs[i], 0.75)))
                        .collect(),
                    feelings,
                    skills: content
                        .skills
                        .iter()
                        .enumerate()
                        .map(|(i, k)| (k.id.clone(), finite(s.skills[i], 0.0)))
                        .collect(),
                    job: s.job.as_ref().map(|j| JobSave {
                        career: content.careers[j.career].id.clone(),
                        level: j.level,
                        performance: finite(j.performance, 0.0),
                        last_shift_day: j.last_shift_day,
                        satisfaction: some_finite(j.satisfaction),
                        shifts: j.shifts,
                        missed: j.missed,
                        shift_mood: some_finite(j.shift_mood),
                        late: j.late,
                    }),
                    job_search_from: s.job_search_from,
                    nap_minutes: minutes_left(s.nap_until),
                    away_minutes: s.away_until.map(minutes_left),
                    visiting: s.visiting.map(|v| (v.plot, Some(minutes_left(v.until)))),
                    tasks,
                    planner: planner_save(content, &s.planner),
                    held: s.held.map(|h| HeldSave {
                        def: content.items[h.def].id.clone(),
                        servings: h.servings,
                        quality: finite(h.quality, 1.0),
                        made_at: h.made_at,
                        recipe: h.recipe.map(|r| content.recipes[r].id.clone()),
                    }),
                    night_woken: s.night_woken,
                }
            })
            .collect();
        let n = self.sims.len();
        let mut relationships = Vec::new();
        for a in 0..n {
            for b in 0..n {
                let rel = *self.relationships.get(a, b);
                if a != b && rel != Relationship::default() {
                    let rel = Relationship {
                        friendship: finite(rel.friendship, 0.0),
                        romance: finite(rel.romance, 0.0),
                        ..rel
                    };
                    relationships.push(RelationshipSave {
                        a: a as u32,
                        b: b as u32,
                        rel,
                        chemistry: None,
                    });
                }
            }
        }
        let chemistry = pack_chemistry(n, |a, b| finite(self.relationships.chemistry(a, b), 0.0));
        SaveFile {
            version: SAVE_VERSION,
            mode: self.mode,
            dirt: self
                .lot
                .tiles()
                .filter_map(|(x, z, i)| {
                    let d = self.dirt.get(i).copied().unwrap_or(0.0);
                    (d >= 1.0 / 255.0).then(|| [x as u16, z as u16, (d * 255.0).round() as u16])
                })
                .collect(),
            tick: self.tick,
            speed: self.speed,
            resume_speed: Some(self.resume_speed),
            auto_super: self.auto_super && self.speed == SUPER_SPEED,
            autonomy: self.autonomy,
            lifespan: Some(self.lifespan),
            storeys: Some(self.storeys),
            basement: (self.stairs.basement > 0).then_some(self.stairs.basement as u8),
            player_moves: (!self.player_moves).then_some(false),
            residents_most: self.residents_most_given,
            rng: self.rng.state(),
            seed: self.seed,
            lot: LotSave {
                width: self.lot.width,
                depth: self.lot.depth,
                h_edges: run_length(&encode(&h)),
                v_edges: run_length(&encode(&v)),
                diagonals: run_length(&encode_diagonals(&diagonals)),
                looks: encode_looks(&self.lot, &self.content),
                floors: encode_floors(&self.lot, &self.content),
            },
            objects,
            next_object_id: Some(self.next_object_id),
            sims,
            households: self
                .households
                .iter()
                .map(|h| HouseholdSave {
                    name: h.name.clone(),
                    plot: h.plot,
                    player: h.player,
                    funds: Some(h.funds),
                    free_will: Some(h.free_will),
                    routines: h
                        .routines
                        .iter()
                        .map(|r| routine_save(content, r))
                        .collect(),
                    expecting: h.expecting,
                    sitter_day: h.sitter_day,
                    // (With a load taken up for a use under way: the use starts again on load.)
                    laundry: Some(
                        crate::machines::with_taken(
                            h.laundry,
                            self.sims
                                .iter()
                                .filter(|s| s.household == h.id)
                                .map(|s| s.taken),
                            content.laundry.map_or(1.0, |l| l.most),
                        )
                        .to_vec(),
                    ),
                    letters: h.letters.clone(),
                })
                .collect(),
            plots: self
                .plots
                .iter()
                .map(|p| PlotSave {
                    name: p.name.clone(),
                    x: p.x,
                    z: p.z,
                    w: p.w,
                    d: p.d,
                    public: p.public,
                    entry: p.entry.filter(|e| e.iter().all(|v| v.is_finite())),
                    roof: p.roof.map(|r| {
                        let b = &content.build;
                        let id = |list: &[crate::content::BuildStyle], i: u8| {
                            list.get(i as usize).map_or(String::new(), |s| s.id.clone())
                        };
                        [id(&b.roofs, r.style), id(&b.roof_colors, r.color)]
                    }),
                    hours: p.hours,
                })
                .collect(),
            relationships,
            chemistry,
            meta: self.meta.clone(),
            exits: self
                .exits
                .iter()
                .copied()
                .filter(|e| e.iter().all(|v| v.is_finite()))
                .collect(),
            events: self
                .events
                .iter()
                .map(|e| {
                    let mut v = serde_json::to_value(e).expect("event serializes");
                    let refs = event_refs(content, e);
                    if !refs.is_empty() {
                        v["refs"] = serde_json::Value::Object(refs);
                    }
                    v
                })
                .collect(),
            next_event_id: self.events.last_id(),
            former: self.former.clone(),
            // (The households' cars: the work taxis' slots are the runs', `taxis`.)
            cars: self.cars.iter().filter(|c| !c.taxi).cloned().collect(),
            taxis: self.taxis.clone(),
            drives: self.drives.clone(),
            garages: self.garages.clone(),
            porches: self.porches.clone(),
            porch_pieces: self.porch_pieces.clone(),
            open_world: self.travel.file.clone(),
            items: self
                .items
                .list
                .iter()
                .map(|it| {
                    let (at, offset) = match it.at {
                        ItemAt::Surface { offset, .. } => ("surface", offset),
                        ItemAt::Stored { .. } => ("stored", [0, 0]),
                        ItemAt::Floor { offset, .. } => ("floor", offset),
                    };
                    let (x, z) = it.at.tile();
                    ItemSave {
                        id: it.id,
                        def: content.items[it.def].id.clone(),
                        at: at.to_owned(),
                        x,
                        z,
                        offset,
                        servings: it.servings,
                        quality: finite(it.quality, 1.0),
                        made_at: it.made_at,
                        household: it.household,
                        spoiled: it.spoiled,
                        recipe: it.recipe.map(|r| content.recipes[r].id.clone()),
                    }
                })
                .collect(),
            next_item: self.items.next_id,
            setups: self
                .setups
                .iter()
                .map(|s| SetupSave {
                    household: s.household,
                    sim: s.sim,
                    object: content.objects[s.object].id.clone(),
                    since: s.since,
                    paid: s.paid,
                })
                .collect(),
            lights: self
                .traffic
                .lights
                .iter()
                .map(|l| LightSave {
                    phase: l.phase,
                    stage: match l.stage {
                        crate::traffic::Stage::Green => 0,
                        crate::traffic::Stage::Amber => 1,
                        crate::traffic::Stage::Clear => 2,
                    },
                    since: l.since,
                })
                .collect(),
        }
    }

    pub fn save_json(&self) -> String {
        serde_json::to_string(&self.to_save()).expect("save serializes")
    }

    fn task_save(&self, t: &Task) -> Option<TaskSave> {
        let content = &self.content;
        Some(match t.kind {
            TaskKind::Use {
                object,
                interaction,
            } => TaskSave::Use {
                object,
                interaction: content.objects[self.objects[object as usize].def].interactions
                    [interaction]
                    .id
                    .clone(),
                directed: t.directed,
                routine: t.routine,
            },
            // (A walk to nowhere isn't saved.)
            TaskKind::MoveTo { x, z } if !(x.is_finite() && z.is_finite()) => return None,
            TaskKind::MoveTo { x, z } => TaskSave::MoveTo {
                x,
                z,
                directed: t.directed,
            },
            TaskKind::Social { target, social } => TaskSave::Social {
                target,
                social: content.socials[social].id.clone(),
                directed: t.directed,
            },
            TaskKind::Work => TaskSave::Work,
            TaskKind::Visit { plot } => TaskSave::Visit {
                plot,
                directed: t.directed,
            },
            TaskKind::GoHome => TaskSave::GoHome {
                directed: t.directed,
            },
            TaskKind::Clean { x, z } => TaskSave::Clean {
                x,
                z,
                directed: t.directed,
            },
            TaskKind::Repair { object } => TaskSave::Repair {
                object,
                directed: t.directed,
            },
            TaskKind::Spot { accident } => TaskSave::Spot {
                accident: content.accidents[accident].id.clone(),
            },
            TaskKind::Cook {
                recipe,
                step,
                origin,
                servings,
                skipped,
                ..
            } => TaskSave::Cook {
                recipe: content.recipes[recipe].id.clone(),
                step,
                origin,
                servings,
                skipped,
                directed: t.directed,
            },
            // (Eating and washing up start again from what's there after loading; a few minutes
            // on the phone are simply over, and so is listening in on a conversation.)
            // (Nor is a carry: a baby is back in the crib after a load, a harvest put down; nor
            // a host bringing a guest something, which is put down.)
            TaskKind::Answer { .. }
            | TaskKind::Eat { .. }
            | TaskKind::Dishes { .. }
            | TaskKind::Phone { .. }
            | TaskKind::Join { .. }
            | TaskKind::Carry { .. }
            | TaskKind::Serve { .. } => return None,
            // (Nor is setting something up: the parcel is back at the door, `setup.rs`.)
            TaskKind::SetUp { .. } => return None,
        })
    }
}

fn skill_id(content: &Content, skill: Option<usize>) -> Option<String> {
    skill.map(|s| content.skills[s].id.clone())
}

/// A saved routine, if its activity (and skill) still exist.
fn routine_save(content: &Content, r: &Routine) -> RoutineSave {
    RoutineSave {
        id: r.id,
        activity: content.activities[r.activity].id.clone(),
        skill: skill_id(content, r.skill),
        days: r.days,
        start: r.start,
        minutes: r.minutes,
    }
}

fn goal_save(content: &Content, g: &Goal) -> GoalSave {
    GoalSave {
        def: content.goals[g.def].id.clone(),
        skill: skill_id(content, g.skill),
        category: g.category.map(|c| content.career_categories[c].id.clone()),
        object: g.object.map(|o| content.objects[o].id.clone()),
        target: finite(g.target, 0.0),
        since_day: g.since_day,
        start: finite(g.start, 0.0),
        progress: finite(g.progress, 0.0),
        history: g.history.map(|h| finite(h, 0.0)),
    }
}

fn reason_save(content: &Content, r: Reason) -> String {
    match r {
        Reason::Trait(t) => format!("trait:{}", content.traits[t].id),
        Reason::Need(n) => format!("need:{}", content.needs[n].id),
        Reason::Mood => "mood".into(),
        Reason::Away => "away".into(),
        Reason::NoPlace => "noPlace".into(),
    }
}

fn planner_save(content: &Content, p: &Planner) -> Option<PlannerSave> {
    let save = PlannerSave {
        routines: p
            .routines
            .iter()
            .map(|r| routine_save(content, r))
            .collect(),
        skip_household: p.skip_household.clone(),
        goals: p.goals.iter().map(|g| goal_save(content, g)).collect(),
        suggestions: p
            .suggestions
            .iter()
            .map(|g| goal_save(content, g))
            .collect(),
        history: p
            .history
            .iter()
            .map(|b| BlockResultSave {
                routine: b.routine,
                household: b.household,
                activity: content.activities[b.activity].id.clone(),
                skill: skill_id(content, b.skill),
                day: b.day,
                start: b.start,
                minutes: b.minutes,
                outcome: match b.outcome {
                    Outcome::Kept => "kept",
                    Outcome::Cut => "cut",
                    Outcome::Skipped => "skipped",
                    Outcome::NoPlace => "noPlace",
                }
                .into(),
                reason: b.reason.map(|r| reason_save(content, r)),
                done: finite(b.done, 0.0),
            })
            .collect(),
        wishes: p
            .wishes
            .iter()
            .map(|&(a, s)| (content.activities[a].id.clone(), skill_id(content, s)))
            .collect(),
        home_wishes: p
            .home_wishes
            .iter()
            .map(|w| home_wish_save(content, w))
            .collect(),
        waited_day: p.waited_day,
        reviewed: p.reviewed,
        purchase: p.purchase.map(|r| PurchaseSave {
            object: content.objects[r.object].id.clone(),
            open: r.open,
            day: r.day,
            declines: r.declines,
        }),
    };
    let empty = save.routines.is_empty()
        && save.skip_household.is_empty()
        && save.goals.is_empty()
        && save.suggestions.is_empty()
        && save.history.is_empty()
        && save.wishes.is_empty()
        && save.home_wishes.is_empty()
        && save.waited_day.is_none()
        && save.purchase.is_none();
    (!empty || save.reviewed != 0).then_some(save)
}

fn event_refs(
    content: &Content,
    e: &social::SocialEvent,
) -> serde_json::Map<String, serde_json::Value> {
    use social::EventKind::*;
    let mut refs = serde_json::Map::new();
    let mut put = |key: &str, id: Option<&String>| {
        if let Some(id) = id {
            refs.insert(key.into(), id.clone().into());
        }
    };
    let n = e.n.and_then(|n| usize::try_from(n).ok());
    match e.kind {
        Broke | Repaired => put(
            "object",
            n.and_then(|n| content.objects.get(n)).map(|d| &d.id),
        ),
        Accident => put(
            "accident",
            n.and_then(|n| content.accidents.get(n)).map(|d| &d.id),
        ),
        GrewOlder => put(
            "stage",
            n.and_then(|n| content.life.stages.get(n)).map(|d| &d.id),
        ),
        Served => put("item", n.and_then(|n| content.items.get(n)).map(|d| &d.id)),
        BoughtCar => put(
            "car",
            n.and_then(|n| content.car_bodies.get(n)).map(|d| &d.id),
        ),
        _ => {}
    }
    put(
        "skill",
        e.skill
            .and_then(|s| content.skills.get(s as usize))
            .map(|d| &d.id),
    );
    if matches!(e.kind, GoalReached | GoalSuggested) {
        put(
            "category",
            e.career
                .and_then(|c| content.career_categories.get(c as usize))
                .map(|d| &d.id),
        );
    } else {
        put(
            "career",
            e.career
                .and_then(|c| content.careers.get(c as usize))
                .map(|d| &d.id),
        );
    }
    put(
        "goal",
        e.goal
            .and_then(|g| content.goals.get(g as usize))
            .map(|d| &d.id),
    );
    refs
}
