//! `cargo review edges` and the gate's check.

use super::*;

/// Every model's hard edges (but a held tool's: `checkglb::meshes::edged`), or those of `keys`.
pub fn survey(paths: &crate::checkglb::Paths, keys: &[String]) -> Result<Vec<Found>, String> {
    let mut out = Vec::new();
    crate::checkglb::meshes::each(paths, keys, false, &mut |key, family, meshes| {
        if crate::checkglb::meshes::edged(key) {
            out.push(Found {
                key: key.to_string(),
                family: family.to_string(),
                triangles: meshes.iter().map(|m| m.triangles.len()).sum(),
                edges: hard_edges(&meshes),
            });
        }
    })?;
    Ok(out)
}

/// `cargo review edges [--all] [--check] [keys…]`: the exit code (1 with `--check` when a model has one).
pub fn main(args: &[String]) -> i32 {
    let flag = |name: &str| args.iter().any(|a| a == name);
    let keys: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .collect();
    let found = match survey(&crate::checkglb::Paths::repo(), &keys) {
        Ok(found) => found,
        Err(e) => {
            eprintln!("review edges: {e}");
            return 1;
        }
    };
    print!("{}", report(&found, flag("--all"), !keys.is_empty()));
    i32::from(flag("--check") && found.iter().any(|f| !f.edges.is_empty()))
}
