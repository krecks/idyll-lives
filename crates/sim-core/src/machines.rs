//! Machines that run on their own (content `runs`): the washing machine, the tumble dryer, the
//! washing line drying the washing in the sun.
//!
//! A machine is empty, running or done (`ObjectInstance::run`). A use that `loads` it (the
//! washing put in, a few minutes) is offered only while it's empty; once that use ends the machine
//! runs its `runs` minutes with nobody at it (the drum turns and the renderer hears it, the
//! snapshot's `running`), and whoever loaded it is free to do something else. Then it is done,
//! the load still in it, until a use that `unloads` it (offered only then) takes the load out and
//! leaves it empty again. Free will picks that up as it picks up any chore; a machine left done
//! waits, as a full washer does, and nothing else is held up by it but loading it again.
//!
//! The washing goes round the household's loads (`Household::laundry`: dirty, washed, dried; see
//! `content::WorkOn`): loading the washer takes a dirty load (`workOn`, as the use starts), taking
//! it out leaves a washed one (`makes`, as the use ends), which the one who took it out carries on
//! to the nearest dryer or line (`carry_on`, the basket in the arms: content `brings`), and the
//! dried washing goes on to be folded at the basket.
//!
//! A dishwasher is filled another way: what's washed up at it goes in its rack (content `rack`,
//! `ObjectInstance::rack`) instead of being washed there and then and put away, and it runs once
//! the rack is full, or once the washing up is done with it a third full (`rack_in`). While it
//! runs, or waits done, nothing more goes in: the washing up goes to a sink, or waits. Emptying
//! it (a use that `unloads` it) puts the clean things away. Saved: `objects[].run`,
//! `objects[].rack`.

use crate::content::{Content, Interaction, WorkOn};
use crate::world::{Ctx, ObjectInstance, Sim, Task, World};

/// Game minutes between machines' steps: one finishes at most this late.
pub const STEP_MINUTES: u64 = 5;

/// Machines run on by `minutes`; one that finishes is done, and stops (the drum, its sound).
pub(crate) fn run(w: &mut World, minutes: f32) {
    let mut changed = false;
    for o in &mut w.objects {
        let Some(left) = o.run.filter(|&m| m > 0.0) else { continue };
        let left = (left - minutes).max(0.0);
        o.run = Some(left);
        changed |= left == 0.0;
    }
    if changed {
        w.structure_version = w.structure_version.wrapping_add(1);
    }
}

/// A use of `o` done (not given up the moment it began): a load put in starts the machine; one
/// taken out empties it; and what it `makes` is the household's to work on next.
pub(crate) fn use_done(sim: &mut Sim, content: &Content, o: &mut ObjectInstance, inter: &Interaction) {
    // (The load taken up for it is used: not given back.)
    sim.taken = 0;
    if inter.loads && o.run.is_none() {
        o.run = content.objects[o.def].runs;
        sim.reshaped = true;
    } else if inter.unloads && o.done() {
        o.run = None;
        o.rack = 0;
        sim.reshaped = true;
    }
    if let Some(made) = inter.makes {
        sim.made |= made.bit();
    }
}

/// `n` dirty things washed up at `o`, a machine with a `rack`, go in it; it starts once full, or
/// with the washing up done (`more` false: nothing else of the household's waits to be cleared)
/// and it at least a third full. Started, it runs on its own as a loaded machine does.
pub(crate) fn rack_in(sim: &mut Sim, content: &Content, o: &mut ObjectInstance, n: u8, more: bool) {
    let def = &content.objects[o.def];
    if def.rack == 0 || o.run.is_some() {
        return;
    }
    o.rack = o.rack.saturating_add(n.max(1)).min(def.rack);
    if o.rack >= def.rack || (!more && o.rack * 3 >= def.rack) {
        o.run = def.runs;
        sim.reshaped = true;
    }
}

/// After taking out a load that's `made` (the wet washing, the dry): on with it to the nearest
/// thing on the same plot that works on it, free, that they may use (the dryer or the line; the
/// basket to fold it at). None: nowhere to take it, and it waits for one.
pub(crate) fn carry_on(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], object: u32, made: WorkOn) -> Option<Task> {
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    crate::world::nearest(sim, ctx, objects, plot, Some(object), |i| i.work_on == Some(made) && !i.baby && i.recipe.is_none())
}

/// How keen a resident at home is on a laundry chore, besides what it does for their needs (as
/// tidying up is keener the dirtier the spot): emptying a machine that's done, `DONE_PULL`; a use
/// that works on a load of washing, `PILE_PULL` with the pile full (content `laundry`'s `most`),
/// less with less waiting. 0 for anything else.
pub(crate) fn pull(laundry: [f32; 3], content: &Content, o: &ObjectInstance, inter: &Interaction) -> f32 {
    if inter.unloads && o.done() {
        return DONE_PULL;
    }
    let Some(k) = inter.work_on.and_then(WorkOn::load) else { return 0.0 };
    let most = content.laundry.map_or(1.0, |l| l.most);
    PILE_PULL * (laundry[k] / most).clamp(0.0, 1.0)
}

/// How keen they are on emptying a machine that's done (`pull`): about as on tidying a spot half dirty.
pub const DONE_PULL: f32 = 0.3;
/// How keen on a load of washing with the pile full (`pull`).
pub const PILE_PULL: f32 = 0.3;

/// Settles what a resident took up and left this tick with their household's loads of washing:
/// one taken from its pile, one put on the next (up to content `laundry`'s `most`).
pub(crate) fn settle(laundry: &mut [f32; 3], worked: u8, made: u8, most: f32) {
    for kind in WorkOn::ALL {
        let Some(k) = kind.load() else { continue };
        if worked & kind.bit() != 0 {
            laundry[k] = (laundry[k] - 1.0).max(0.0);
        }
        if made & kind.bit() != 0 {
            laundry[k] = (laundry[k] + 1.0).min(most);
        }
    }
}

/// A household's piles of washing with the loads its residents took up for uses under way given
/// back (`Sim::taken`, `WorkOn` bits each), as a save keeps them: those uses start again on load.
pub(crate) fn with_taken(mut laundry: [f32; 3], taken: impl Iterator<Item = u8>, most: f32) -> [f32; 3] {
    for t in taken {
        settle(&mut laundry, 0, t, most);
    }
    laundry
}
