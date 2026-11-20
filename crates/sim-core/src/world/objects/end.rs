//! Stopping what they are doing and stepping away from it.

use crate::content::{Content, Pose};
use crate::world::{LADDER_DOWN_TICKS, ObjectInstance, Sim, TaskKind, held_object, wrap_angle};
use std::f32::consts::FRAC_PI_2;

/// Stops what they're doing: the object they used (or cooked at, sat at to eat, washed up at)
/// is free again, and they stand up in front of it (or step out of the shower; climbing down from
/// the upper bunk first: `Sim::ladder_ticks`). What they carry is put down after the step
/// (`cooking::after_step`).
pub(crate) fn end_activity(sim: &mut Sim, content: &Content, objects: &mut [ObjectInstance]) {
    sim.join_spot = None;
    sim.sweep = None;
    sim.kept_out = 0;
    // A talk from the seat over: back to what they sat down to do, still sitting.
    if let Some(paused) = sim.paused.take() {
        if matches!(
            sim.current.as_ref().map(|a| a.task.kind),
            Some(TaskKind::Social { .. } | TaskKind::Join { .. })
        ) {
            sim.current = Some(paused);
            sim.idle_ticks = 0;
            return;
        }
        // (Ended some other way: what they sat down to do ends too.)
        if let Some(o) = held_object(paused.task.kind).filter(|&o| (o as usize) < objects.len()) {
            objects[o as usize].release(sim.id);
        }
        if let Some((fx, fz)) = sim.seated_from {
            sim.pos = [fx as f32 + 0.5, fz as f32 + 0.5];
        }
    }
    // A load of washing taken up for a use that didn't get done goes back on its pile.
    sim.made |= std::mem::take(&mut sim.taken);
    let held = sim
        .current
        .take()
        .and_then(|act| held_object(act.task.kind));
    if let Some(o) = held.map(|o| o as usize).filter(|&o| o < objects.len()) {
        // Up from the upper bunk (its second slot, see `ObjectInstance::slot_position`): down its ladder first.
        if sim.pose == Pose::Lie
            && content.objects[objects[o].def].bunk
            && objects[o].users[1] == Some(sim.id)
        {
            sim.ladder_ticks = LADDER_DOWN_TICKS;
        }
        objects[o].release(sim.id);
        if sim.pose != Pose::Stand || sim.seated_from.is_some() {
            let (fx, fz) = sim
                .seated_from
                .unwrap_or_else(|| objects[o].front_tile(content));
            let to = [fx as f32 + 0.5, fz as f32 + 0.5];
            // Out of bed facing the way they get out, square to it: off its side, sitting up
            // on the edge with their back to the pillows, or over its foot.
            if sim.pose == Pose::Lie {
                let a = objects[o].angle();
                let d = (to[0] - sim.pos[0]).atan2(to[1] - sim.pos[1]) - a;
                sim.yaw = wrap_angle(a + (d / FRAC_PI_2).round() * FRAC_PI_2);
            }
            sim.pos = to;
        }
    }
    if let Some(seat) = sim.use_seat.take()
        && let Some(o) = objects.get_mut(seat as usize)
    {
        o.release(sim.id);
    }
    sim.seated_from = None;
    sim.pose = Pose::Stand;
    sim.idle_ticks = 0;
}
