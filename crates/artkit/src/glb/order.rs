//! The order objects go into the file in.
//!
//! Split from `glb.rs` (see `mod.rs`).

use super::*;

/// A name as Blender's exporter sorts nodes by it: its runs of digits as numbers, the text between without case
/// (`door1` before `door-1`, `water` before `water.001`, `leg2` before `leg10`).
fn natural_key(name: &str) -> Vec<(String, u128)> {
    // (text, the number after it) pairs: as Python compares the split list, text and numbers in turn
    let mut out = Vec::new();
    let mut text = String::new();
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            let mut digits = String::from(c);
            while let Some(d) = chars.peek().copied().filter(char::is_ascii_digit) {
                digits.push(d);
                chars.next();
            }
            // (one more than the number, so that text with no number after it sorts first)
            out.push((
                std::mem::take(&mut text),
                digits.parse::<u128>().unwrap_or(u128::MAX - 1) + 1,
            ));
        } else {
            text.extend(c.to_lowercase());
        }
    }
    out.push((text, 0));
    out
}

/// The parts as the file has its nodes: the ones with no parent among them by name (as `natural_key` sorts),
/// each after its children, they by name too; and the order the clips are met in: each parent before its
/// children. Blender's exporter walks the scene so, whatever order the parts were made or given in.
pub(crate) fn file_order(objs: &[Obj]) -> (Vec<Obj>, Vec<usize>) {
    let by_name = |list: &mut Vec<Obj>| list.sort_by_cached_key(|o| natural_key(&o.name()));
    let mut roots: Vec<Obj> = Vec::new();
    let mut children: BTreeMap<Obj, Vec<Obj>> = BTreeMap::new();
    for &o in objs {
        match o.read(|p| p.parent).filter(|p| objs.contains(p)) {
            Some(parent) => children.entry(parent).or_default().push(o),
            None => roots.push(o),
        }
    }
    by_name(&mut roots);
    children.values_mut().for_each(by_name);
    fn walk(o: Obj, children: &BTreeMap<Obj, Vec<Obj>>, post: &mut Vec<Obj>, pre: &mut Vec<Obj>) {
        pre.push(o);
        for &child in children.get(&o).map(Vec::as_slice).unwrap_or(&[]) {
            walk(child, children, post, pre);
        }
        post.push(o);
    }
    let (mut post, mut pre) = (Vec::new(), Vec::new());
    for o in roots {
        walk(o, &children, &mut post, &mut pre);
    }
    let met = pre
        .iter()
        .map(|o| post.iter().position(|p| p == o).unwrap())
        .collect();
    (post, met)
}

/// An order for floats that sorts them as numbers.
pub(crate) fn key(x: f32) -> i64 {
    let b = x.to_bits() as i64;
    if b < 0x8000_0000 {
        b
    } else {
        -(b - 0x8000_0000)
    }
}
