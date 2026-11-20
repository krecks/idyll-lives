//! Serving the dish: where it goes, and carrying it there.

use super::*;

/// Somewhere to put things: the objects that may carry them.
pub(crate) struct Place<'a> {
    pub content: &'a Content,
    pub objects: &'a [ObjectInstance],
    pub lot: &'a crate::lot::Lot,
    pub candidates: &'a [u32],
}

impl Place<'_> {
    /// The room at `near` (None off the lot).
    fn room_of(&self, near: [f32; 2]) -> Option<u16> {
        let (x, z) = (near[0].floor() as i32, near[1].floor() as i32);
        self.lot.in_bounds(x, z).then(|| self.lot.room_at(x, z))
    }

    /// Whether object `o` stands in `room` (None: anywhere).
    fn in_room(&self, o: &ObjectInstance, room: Option<u16>) -> bool {
        room.is_none_or(|r| self.lot.in_bounds(o.x, o.z) && self.lot.room_at(o.x, o.z) == r)
    }

    /// A free spot for something of `radius` near `near` (within `range`), on what stands in
    /// `room` (None: anywhere): on an outdoor surface first if `outdoor`, then on a table, bar or
    /// counter, else any.
    fn spot(
        &self,
        items: &Items,
        near: [f32; 2],
        range: f32,
        radius: f32,
        outdoor: bool,
        room: Option<u16>,
    ) -> Option<SurfaceSpot> {
        use crate::content::SurfaceKind;
        let occupied = items.occupancy(self.content);
        let candidates: Vec<u32> = self
            .candidates
            .iter()
            .copied()
            .filter(|&o| self.in_room(&self.objects[o as usize], room))
            .collect();
        let near_on = |candidates: &[u32], kinds: u8| {
            surface::free_surface_spot_near(
                self.content,
                self.objects,
                candidates,
                near,
                range,
                kinds,
                radius,
                &occupied,
            )
        };
        if outdoor {
            let outside: Vec<u32> = candidates
                .iter()
                .copied()
                .filter(|&o| {
                    let c = self.objects[o as usize].centre(self.content);
                    let (x, z) = (c[0].floor() as i32, c[1].floor() as i32);
                    self.lot.in_bounds(x, z) && self.lot.room_at(x, z) == crate::lot::OUTDOORS
                })
                .collect();
            if let Some(s) = near_on(&outside, 0) {
                return Some(s);
            }
        }
        let tops = SurfaceKind::Table.bit() | SurfaceKind::Bar.bit() | SurfaceKind::Counter.bit();
        near_on(&candidates, tops).or_else(|| near_on(&candidates, 0))
    }

    /// The nearest object in `room` (None: anywhere) that keeps food (a fridge).
    fn store(&self, near: [f32; 2], room: Option<u16>) -> Option<&ObjectInstance> {
        let d = |o: &ObjectInstance| {
            let c = o.centre(self.content);
            (c[0] - near[0]).hypot(c[1] - near[1])
        };
        self.candidates
            .iter()
            .map(|&o| &self.objects[o as usize])
            .filter(|o| self.content.objects[o.def].stores && self.in_room(o, room))
            .min_by(|a, b| d(a).total_cmp(&d(b)))
    }

    /// Puts down what someone carries: food on a free surface near `near` (else in the fridge,
    /// else on the floor), dirty dishes on a surface or the floor; ingredients and bowls are
    /// used up. A surface or fridge in the same room as `near` comes first (not one through the
    /// wall that happens to be nearer). `prefer` is a spot to try first. Returns the (first) item
    /// put down.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn put_down(
        &self,
        items: &mut Items,
        held: Held,
        household: u32,
        near: [f32; 2],
        range: f32,
        outdoor: bool,
        mut prefer: Option<SurfaceSpot>,
        tick: u64,
    ) -> Option<u32> {
        // (A stack, `stack_as`, goes down as the things it's a stack of.)
        let base = (self.content.items.iter())
            .position(|d| d.stack_as == Some(held.def))
            .unwrap_or(held.def);
        let food = held.recipe.is_some() && held.servings > 0;
        // (Something being cleared away with food still on it, put down part way: as the dirty
        // thing it's on its way to being, the food scraped.)
        let base = match self.content.items[base].empty_as {
            Some(e) if !food && !self.content.items[base].dirty && self.content.items[e].dirty => e,
            _ => base,
        };
        let def = &self.content.items[base];
        if !food && !def.dirty {
            return None;
        }
        // In this room first, then anywhere.
        let here = self.room_of(near);
        let rooms = [here, None];
        let rooms = &rooms[..if here.is_some() { 2 } else { 1 }];
        // Dirty things one by one (a stack of plates is so many plates).
        let count = if food { 1 } else { held.servings.max(1) };
        let mut first = None;
        for _ in 0..count {
            let at = prefer
                .take()
                .map(|spot| ItemAt::on(&spot))
                .or_else(|| {
                    rooms.iter().find_map(|&room| {
                        self.spot(items, near, range, def.radius, outdoor, room)
                            .map(|spot| ItemAt::on(&spot))
                            .or_else(|| {
                                self.store(near, room)
                                    .filter(|_| food)
                                    .map(|o| ItemAt::Stored { x: o.x, z: o.z })
                            })
                    })
                })
                .unwrap_or_else(|| {
                    ItemAt::Floor {
                        x: 0,
                        z: 0,
                        offset: [0, 0],
                    }
                    .moved_to(near)
                });
            let id = items.add(ItemInstance {
                id: 0,
                def: base,
                at,
                servings: if food { held.servings } else { 0 },
                quality: held.quality,
                made_at: if food { held.made_at } else { tick },
                household,
                spoiled: false,
                recipe: held.recipe.filter(|_| food),
            });
            first.get_or_insert(id);
        }
        first
    }
}

pub(crate) fn place<'a>(ctx: &Ctx<'a>, objects: &'a [ObjectInstance], household: u32) -> Place<'a> {
    Place {
        content: ctx.content,
        objects,
        lot: ctx.nav.lot,
        candidates: home_objects(ctx, household),
    }
}

/// The last step is done: the dish is carried to the dining table and served there (else served
/// near the cook), or, made to eat at once, they keep the plate and sit down with it. Called again
/// with the dish in hand (the table broke on the way), they carry on with it.
pub(crate) fn serve(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    last: Option<u32>,
) {
    let content = ctx.content;
    let TaskKind::Cook {
        recipe,
        origin,
        servings,
        skipped,
        ..
    } = task.kind
    else {
        return;
    };
    sim.current = None;
    let r = &content.recipes[recipe];
    if let Some(dish) = sim
        .held
        .filter(|h| h.recipe == Some(recipe) && h.def == r.item && h.servings > 0)
    {
        return carry_dish(sim, task, ctx, objects, items, dish);
    }
    // (Every step done without its station: the ingredients are bought now.)
    let servings = match servings {
        0 => match buy_ingredients(sim, ctx, r) {
            Some(n) => n,
            None => return,
        },
        n => n,
    };
    let inter = origin_interaction(content, objects, origin, recipe);
    let skill = inter.map_or(1.0, |i| i.skill_factor(&content.skill_rules, &sim.skills));
    let tools = last
        .and_then(|o| objects.get(o as usize))
        .map_or(1.0, |o| content.object_rules.quality_factor(o.quality));
    let quality =
        (skill * tools * (1.0 - content.cooking.skip_penalty * f32::from(skipped))).max(0.3);
    if let Some(i) = inter {
        if let Some(f) = i.earns_feeling(i.minutes, &sim.skills) {
            crate::social::add_feeling(&mut sim.feelings, f, &content.feelings, ctx.tick);
        }
        if let Some(o) = last
            .and_then(|o| objects.get(o as usize))
            .filter(|_| i.dirt > 0.0)
        {
            let (fx, fz) = approach(ctx, o);
            sim.mess = Some((fx, fz, i.dirt));
        }
    }
    if r.eat_now {
        return eat_now(sim, task, recipe, quality, ctx, objects, items);
    }
    let made = Held {
        def: r.item,
        servings,
        quality,
        made_at: ctx.tick,
        recipe: Some(recipe),
    };
    let task = Task {
        kind: TaskKind::Cook {
            recipe,
            step: r.steps.len() as u8,
            station: None,
            origin,
            servings,
            skipped,
            minutes: 0,
        },
        ..task
    };
    carry_dish(sim, task, ctx, objects, items, made);
}

/// Whether `task` is a recipe's serving (the dish carried to the table, past its last step).
pub(crate) fn serving(content: &Content, task: &Task) -> bool {
    matches!(task.kind, TaskKind::Cook { recipe, step, .. } if step as usize >= content.recipes[recipe].steps.len())
}

/// With `dish` made: in hand to the dining table, if there is one near, to be put down there
/// (`step_done`); else put down near the cook straight away.
pub(crate) fn carry_dish(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    dish: Held,
) {
    let content = ctx.content;
    let TaskKind::Cook {
        recipe,
        step,
        origin,
        servings,
        skipped,
        ..
    } = task.kind
    else {
        return;
    };
    let r = &content.recipes[recipe];
    let tags = cook_tags(content, objects, origin, recipe);
    sim.held = Some(dish);
    if let Some(spot) = serving_table(
        sim,
        ctx,
        objects,
        items,
        content.items[dish.def].radius,
        r.outdoor,
    ) {
        let minutes = content.cooking.serve_minutes.round() as u16;
        let to = Task {
            kind: TaskKind::Cook {
                recipe,
                step,
                station: Some(spot.host),
                origin,
                servings,
                skipped,
                minutes,
            },
            ..task
        };
        if walk_to_any(
            sim,
            to,
            ctx,
            &item_access(ctx, objects, &ItemAt::on(&spot)),
            tags,
        ) {
            return;
        }
    }
    sim.held = None;
    let at = place(ctx, objects, sim.household);
    if let Some(id) = at.put_down(
        items,
        dish,
        sim.household,
        sim.pos,
        content.cooking.serve_range,
        r.outdoor,
        None,
        ctx.tick,
    ) {
        served(sim, task, ctx, objects, items, id);
    }
}

/// The dining table to serve a dish of `radius` at, and a free spot on it, near its middle: of
/// the home's tops with a chair pulled up to them or seats of their own (where the household
/// will sit down to it), the nearest within `cooking.tableRange` on this storey, an outdoor one
/// first for a dish cooked `outdoor` (a barbecue on the picnic table), an indoor one first
/// otherwise; a table before a counter or bar with stools.
pub(crate) fn serving_table(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    items: &Items,
    radius: f32,
    outdoor: bool,
) -> Option<SurfaceSpot> {
    let content = ctx.content;
    let rules = &content.cooking;
    if sim.visiting.is_some() || rules.table_range <= 0.0 {
        return None;
    }
    let lot = ctx.nav.lot;
    let stairs = &ctx.nav.stairs;
    let storey = stairs.storey(sim.pos[1].floor() as i32);
    let stage = content.life.stage(sim.age);
    let outside = |o: &ObjectInstance| {
        let c = o.centre(content);
        let (x, z) = (c[0].floor() as i32, c[1].floor() as i32);
        lot.in_bounds(x, z) && lot.room_at(x, z) == crate::lot::OUTDOORS
    };
    // (Rank: outdoors or not as the dish, then a table before a counter or bar.)
    let mut tables: Vec<(u8, u32)> = Vec::new();
    for &o in home_objects(ctx, sim.household) {
        let obj = &objects[o as usize];
        let def = &content.objects[obj.def];
        if obj.broken() {
            continue;
        }
        let table = match ctx.index.links.table_of(o) {
            Some(t) if def.seat_at != 0 => t,
            _ if def.surface.is_some()
                && obj.sits(def, stage, rules.eat_tags, rules.seat_avoid) =>
            {
                o
            }
            _ => continue,
        };
        let t = &objects[table as usize];
        let kind = content.objects[t.def].surface.as_ref().map(|s| s.kind);
        let rank = u8::from(outside(t) != outdoor) * 2
            + u8::from(kind != Some(crate::content::SurfaceKind::Table));
        if !t.broken() && stairs.storey(t.z) == storey && !tables.iter().any(|&(_, x)| x == table) {
            tables.push((rank, table));
        }
    }
    let occupied = items.occupancy(content);
    let middle = |t: u32| surface::middle(content, &objects[t as usize]);
    (0..4).find_map(|rank| {
        let here: Vec<u32> = tables
            .iter()
            .filter(|&&(r, _)| r == rank)
            .map(|&(_, t)| t)
            .collect();
        // The nearest with room for it, then its spot nearest its middle.
        let near = surface::free_surface_spot_near(
            content,
            objects,
            &here,
            sim.pos,
            rules.table_range,
            0,
            radius,
            &occupied,
        )?;
        surface::free_surface_spot_near(
            content,
            objects,
            &[near.host],
            middle(near.host),
            rules.table_range,
            0,
            radius,
            &occupied,
        )
        .or(Some(near))
    })
}

/// The dish is on the table (or wherever it went): "dinner's ready", and hungry themselves, they
/// have some too.
pub(crate) fn served(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    id: u32,
) {
    sim.served = Some(id);
    let Some(recipe) = items.get(id).and_then(|it| it.recipe) else {
        return;
    };
    if hungry_for(sim, ctx.content, &ctx.content.recipes[recipe]) {
        let eat = Task {
            kind: TaskKind::Eat {
                item: Some(id),
                seat: None,
                plate: None,
                stage: 0,
            },
            ..task
        };
        eat_next(sim, eat, ctx, objects, items);
    }
}

/// At the table with the dish (`serving`): it goes down on a free spot of it, near its middle
/// (else wherever there's room near), and is served.
pub(crate) fn put_dish_down(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    table: Option<u32>,
) {
    let content = ctx.content;
    let Some(dish) = sim.held.take() else {
        return;
    };
    let outdoor =
        matches!(task.kind, TaskKind::Cook { recipe, .. } if content.recipes[recipe].outdoor);
    let occupied = items.occupancy(content);
    let radius = content.items[dish.def].radius;
    let spot = table
        .filter(|&t| !objects[t as usize].broken())
        .and_then(|t| {
            let middle = surface::middle(content, &objects[t as usize]);
            surface::free_surface_spot_near(
                content,
                objects,
                &[t],
                middle,
                content.cooking.table_range.max(2.0),
                0,
                radius,
                &occupied,
            )
        });
    let at = place(ctx, objects, sim.household);
    if let Some(id) = at.put_down(
        items,
        dish,
        sim.household,
        sim.pos,
        content.cooking.serve_range,
        outdoor,
        spot,
        ctx.tick,
    ) {
        served(sim, task, ctx, objects, items, id);
    }
}
