//! The looks of a wall edge: what an edit may set and what it costs to repair.

use super::*;

/// Whether an edit's look exists in the content (form, covering, door or window style).
pub(super) fn check_look(rules: &crate::content::BuildRules, e: &EdgeEdit) -> Result<(), Error> {
    if e.form.is_some_and(|f| f > FORM_HALF) {
        return Err(Error::new("a wall is full height (0) or a half wall (1)"));
    }
    if e.cover.is_some_and(|c| c as usize > rules.coverings.len()) {
        return Err(Error::new("unknown wall covering"));
    }
    let styles = match e.kind {
        EdgeKind::Door => rules.doors.len(),
        EdgeKind::Window => rules.windows.len(),
        EdgeKind::Fence | EdgeKind::Gate => rules.fences.len(),
        _ => return Ok(()),
    };
    if e.style.is_some_and(|s| s as usize >= styles.max(1)) {
        return Err(Error::new("unknown style for that"));
    }
    if matches!(e.kind, EdgeKind::Door | EdgeKind::Window) && e.form == Some(FORM_HALF) {
        return Err(Error::new("doors and windows need a full-height wall"));
    }
    match e.kind {
        EdgeKind::Door => check_opening(false, e.sill, e.head),
        EdgeKind::Window => check_opening(true, e.sill, e.head),
        _ if e.sill.is_none() && e.head.is_none() => Ok(()),
        _ => Err(Error::new("only doors and windows have a sill or a top")),
    }
}

/// Whether a door's or (`window`) a window's own sill and top, centimetres (absent: its style's),
/// are a size it may have: as Build sets them, and as a save brings them back (`save::pack`).
pub(crate) fn check_opening(
    window: bool,
    sill: Option<u16>,
    head: Option<u16>,
) -> Result<(), Error> {
    use crate::command::{MAX_HEAD, MIN_DOOR_HEAD, MIN_WINDOW};
    if !window {
        if sill.is_some() {
            return Err(Error::new("a door starts at the floor"));
        }
        if head.is_some_and(|h| !(MIN_DOOR_HEAD..=MAX_HEAD).contains(&h)) {
            return Err(Error::new(format!(
                "a door is {} to {} m tall",
                MIN_DOOR_HEAD as f32 / 100.0,
                MAX_HEAD as f32 / 100.0
            )));
        }
        return Ok(());
    }
    if head.is_some_and(|h| h > MAX_HEAD) || sill.is_some_and(|s| s > MAX_HEAD - MIN_WINDOW) {
        return Err(Error::new(format!(
            "a window reaches {} m at most",
            MAX_HEAD as f32 / 100.0
        )));
    }
    // (The sill is at most `MAX_HEAD - MIN_WINDOW` here: adding to it can't overflow.)
    if let (Some(s), Some(h)) = (sill, head)
        && s + MIN_WINDOW > h
    {
        return Err(Error::new(format!(
            "a window is at least {MIN_WINDOW} cm tall"
        )));
    }
    Ok(())
}

/// The look a wall has after an edit: new walls take the edit's form and covering, new doors
/// and windows its style; standing ones keep their faces and change only what the edit names.
pub(super) fn next_look(current: Edge, target: Edge, old: EdgeLook, e: &EdgeEdit) -> EdgeLook {
    match target {
        Edge::Open => EdgeLook::default(),
        // Fences and gates have only a style; a gate put into a fence keeps the fence's.
        Edge::Fence | Edge::Gate => EdgeLook {
            style: e
                .style
                .unwrap_or(if current.is_fence() { old.style } else { 0 }),
            ..EdgeLook::default()
        },
        _ if current == Edge::Open => {
            let cover = e.cover.unwrap_or(0);
            EdgeLook {
                sides: [cover, cover],
                form: if target == Edge::Wall {
                    e.form.unwrap_or(0)
                } else {
                    0
                },
                style: if target == Edge::Wall {
                    0
                } else {
                    e.style.unwrap_or(0)
                },
                sill: if target == Edge::Window { e.sill } else { None },
                head: if target == Edge::Wall { None } else { e.head },
            }
        }
        Edge::Wall => EdgeLook {
            form: e
                .form
                .unwrap_or(if current == Edge::Wall { old.form } else { 0 }),
            style: 0,
            sill: None,
            head: None,
            ..old
        },
        // A door or window keeps its size when restyled; one put in for another kind starts as its style is.
        _ => EdgeLook {
            form: 0,
            style: e
                .style
                .unwrap_or(if current == target { old.style } else { 0 }),
            sill: if target == Edge::Window {
                e.sill.or(if current == target { old.sill } else { None })
            } else {
                None
            },
            head: e.head.or(if current == target { old.head } else { None }),
            ..old
        },
    }
}

/// What the player's quick fix costs (Buy mode): a share of the price, by wear.
pub(crate) fn repair_cost(
    content: &crate::content::Content,
    obj: &crate::world::ObjectInstance,
) -> i64 {
    let price = content.objects[obj.def].price.unwrap_or(0) as f32;
    (price * content.object_rules.repair.cost * obj.wear)
        .round()
        .max(1.0) as i64
}
