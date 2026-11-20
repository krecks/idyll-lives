//! The report and the command line.
//!
//! Split from `built.rs` (see `mod.rs`).

use super::*;

// ---- the report ---------------------------------------------------------------------------------------

/// What a run measured, as tables (`--report`): each sample's counts, where the hard edges are, how each material's
/// texture is laid, every far version's look beside its near one's, and the houses' detail.
pub(crate) fn report(r: &Report) -> String {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "## Samples\n\n| sample | triangles | draws | hard edges | metres | texel findings | far against near |\n|---|---|---|---|---|---|---|"
    );
    let dash = || "—".to_string();
    for m in &r.samples {
        let _ = writeln!(
            text,
            "| {} | {} | {} | {} | {} | {} | {} |",
            m.sample.key,
            m.triangles,
            m.draws,
            m.edges.as_ref().map_or(dash(), |e| e.count().to_string()),
            m.edges
                .as_ref()
                .map_or(dash(), |e| format!("{:.1}", e.length())),
            m.materials.as_ref().map_or(dash(), |ms| ms
                .iter()
                .map(|x| texels::findings(x).len())
                .sum::<usize>()
                .to_string()),
            m.far_near.as_ref().map_or(dash(), |f| f.len().to_string()),
        );
    }
    // By kind: the most a sample of it has.
    let mut kinds: BTreeMap<&str, (usize, usize, usize, usize)> = BTreeMap::new();
    for m in &r.samples {
        let k = kinds.entry(&m.sample.kind).or_default();
        *k = (
            k.0 + 1,
            k.1.max(m.triangles),
            k.2.max(m.draws),
            k.3 + m.triangles,
        );
    }
    let _ = writeln!(
        text,
        "\n| kind | samples | triangles (mean) | triangles (most) | draws (most) |\n|---|---|---|---|---|"
    );
    for (kind, (n, most, draws, sum)) in &kinds {
        let _ = writeln!(
            text,
            "| {kind} | {n} | {} | {most} | {draws} |",
            sum / n.max(&1)
        );
    }

    let _ = writeln!(
        text,
        "\n## Hard edges, by where\n\n| kind | part | runs | edges | metres | the longest |\n|---|---|---|---|---|---|"
    );
    // By kind, part and the way it runs: how many edges, how long together, the longest.
    type Found<'a> = (usize, f64, Option<&'a Edge>);
    let mut places: BTreeMap<(&str, &str, &str), Found> = BTreeMap::new();
    for m in &r.samples {
        for (count, length, longest) in m.edges.iter().flat_map(|e| e.0.values()) {
            let Some(e) = longest.as_ref() else {
                continue;
            };
            let slot = places
                .entry((&m.sample.kind, &e.part, runs(e)))
                .or_default();
            slot.0 += count;
            slot.1 += length;
            if slot.2.is_none_or(|l| e.length > l.length) {
                slot.2 = Some(e);
            }
        }
    }
    let mut rows: Vec<_> = places.iter().collect();
    rows.sort_by(|a, b| a.0.0.cmp(b.0.0).then(b.1.1.total_cmp(&a.1.1)));
    for ((kind, part, way), (count, length, longest)) in rows {
        let _ = writeln!(
            text,
            "| {kind} | {part} | {way} | {count} | {length:.1} | {} |",
            longest.map_or(dash(), place)
        );
    }

    let _ = writeln!(
        text,
        "\n## Texels, by material\n\n| material | texture px | uvScale | parts | m² | px a metre (median) | 5th to 95th | stretched | smeared | findings |\n|---|---|---|---|---|---|---|---|---|---|"
    );
    for (material, texture) in &r.textures {
        let laid: Vec<&texels::Material> = r
            .samples
            .iter()
            .flat_map(|m| m.materials.iter().flatten())
            .filter(|m| &m.finish == material)
            .collect();
        if laid.is_empty() {
            continue;
        }
        let area: f64 = laid.iter().map(|m| m.area).sum();
        let mut spread: Vec<(f64, f64)> = laid.iter().flat_map(|m| m.spread.clone()).collect();
        let px = |at: f64, spread: &mut [(f64, f64)]| match (texture.px, texels::median(spread, at))
        {
            (Some(px), Some(v)) => format!("{:.0}", v * texture.uv_scale * px),
            _ => dash(),
        };
        let share = |of: &dyn Fn(&texels::Material) -> f64| {
            format!(
                "{:.1} %",
                laid.iter().map(|m| of(m) * m.area).sum::<f64>() / area.max(1e-9) * 100.0
            )
        };
        let _ = writeln!(
            text,
            "| {material} | {} | {} | {} | {area:.0} | {} | {} to {} | {} | {} | {} |",
            texture.px.map_or("unread".into(), |px| format!("{px:.0}")),
            texture.uv_scale,
            laid.len(),
            px(0.5, &mut spread),
            px(0.05, &mut spread),
            px(0.95, &mut spread),
            share(&|m| m.stretched),
            share(&|m| m.smeared),
            laid.iter()
                .map(|m| texels::findings(m).len())
                .sum::<usize>(),
        );
    }

    let _ = writeln!(
        text,
        "\n## Far against near\n\n| sample | plan m² | plan box m | top m | roof | chimney | walls | roof colour | disagreements |\n|---|---|---|---|---|---|---|---|---|"
    );
    let row = |key: &str, l: &Look, found: Option<&Vec<String>>| {
        format!(
            "| {key} | {:.0} | {:.1} × {:.1} | {:.2} | {}{} | {} | {} | {} | {} |",
            l.area,
            l.plan[2] - l.plan[0],
            l.plan[3] - l.plan[1],
            l.top,
            roof_name(l.roof),
            if l.roof == Roof::Flat {
                String::new()
            } else {
                format!(" {:.0}°", l.pitch)
            },
            if l.chimneys > 0 { "yes" } else { "no" },
            hex_of(l.wall),
            hex_of(l.roof_colour),
            found.map_or(dash(), |f| {
                if f.is_empty() {
                    "none".into()
                } else {
                    f.iter()
                        .map(|line| line.split(':').next().unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            })
        )
    };
    let mut shown: Vec<&str> = Vec::new();
    for m in &r.samples {
        let (Some(look), Some(of)) = (&m.look, &m.sample.of) else {
            continue;
        };
        if let Some(near) = r.near.get(of).filter(|_| !shown.contains(&of.as_str())) {
            shown.push(of);
            let _ = writeln!(text, "{}", row(of, near, None));
        }
        let _ = writeln!(text, "{}", row(&m.sample.key, look, m.far_near.as_ref()));
    }

    let _ = writeln!(
        text,
        "\n## The houses' detail\n\n| house | trim m² | of it neither plumb nor level | eased edges, upright m | eased edges, others m | roof trim looking up m² | each storey's roof: ridges m, hips m, cap shingles m², slopes under its eaves m |\n|---|---|---|---|---|---|---|"
    );
    for m in &r.samples {
        if let Some(d) = &m.detail {
            let roofs: Vec<String> = d
                .roofs
                .iter()
                .map(|(storey, [ridges, hips, caps, sunk])| {
                    format!("storey {storey}: {ridges:.1}, {hips:.1}, {caps:.2}, {sunk:.2}")
                })
                .collect();
            let _ = writeln!(
                text,
                "| {} | {:.1} | {:.2} | {:.1} | {:.1} | {:.2} | {} |",
                m.sample.key,
                d.trim,
                d.trim_shaped,
                d.eased_upright,
                d.eased,
                d.troughs,
                roofs.join("; ")
            );
        }
    }
    text
}

pub(crate) const USAGE: &str = "usage: cargo check-glb --built <dir> [sample keys…] [--report] [--update-baseline] [--seed-baseline] (the dir `bun tools/built/dump.ts <dir>` wrote)";

/// `cargo check-glb --built <dir> …`: the exit code.
pub(crate) fn main(args: &[String]) -> i32 {
    let dir = args
        .iter()
        .position(|a| a == "--built")
        .and_then(|i| args.get(i + 1))
        .filter(|d| !d.starts_with("--"));
    let Some(dir) = dir else {
        eprintln!("{USAGE}");
        return 2;
    };
    let keys: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--") && *a != dir)
        .cloned()
        .collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let paths = Paths::repo();
    // A change to the baseline from what a run counted: the baseline after it, a line for each change.
    type Change = dyn Fn(&Baseline, &Counts) -> Result<(Baseline, Vec<String>), String>;
    let rewrite = |what: &str, change: &Change| -> Result<(), String> {
        let run = gate(&paths, dir, &keys);
        let (next, changes) = change(&Baseline::read(&paths.baseline)?, &run.counts)?;
        if changes.is_empty() {
            println!("{}: nothing to {what}", baseline::FILE);
        } else {
            next.write(&paths.baseline)?;
            for c in changes {
                println!("{}: {c}", baseline::FILE);
            }
        }
        Ok(())
    };
    let mut done = Ok(());
    if has("--seed-baseline") {
        done = rewrite("seed", &seeded);
    }
    if done.is_ok() && has("--update-baseline") {
        done = rewrite("lower", &|baseline, counts| Ok(baseline.lowered(counts)));
    }
    if let Err(err) = done {
        eprintln!("cannot write {}: {err}", baseline::FILE);
        return 1;
    }
    let run = gate(&paths, dir, &keys);
    print!("{}", run.text);
    if has("--report") {
        print!("\n{}", report(&run));
    }
    i32::from(run.failed > 0)
}
