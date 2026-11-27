//! `cargo review` itself.

use super::*;

pub const USAGE: &str = "usage: cargo review sizes [--md <dir>] [--check] [--update-baseline] [--seed-baseline] | uses [--md <dir>] [--check] [--update-baseline] [--seed-baseline] | colours [--all] [--check] | facets [--all] [--check] | edges [--all] [--check] [keys] | texels [--all] [--check] [keys]";

/// `cargo review <sizes|uses> [--md <dir>] [--check]`: the exit code (2: usage, or no reference for
/// `sizes`; 1: an object of no kind, a `species` that isn't one, or `--check` found a row the baseline doesn't
/// hold or a hard flag, a use the renderer can't play, or a counted flag of `uses` past the baseline).
pub fn main(args: &[String]) -> i32 {
    if args.first().map(String::as_str) == Some("colours") {
        return crate::colours::main(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("facets") {
        return crate::facets::main(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("edges") {
        return crate::edges::main(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("texels") {
        return crate::texels::main(&args[1..]);
    }
    let check = args.iter().any(|a| a == "--check");
    let md = match args.iter().position(|a| a == "--md") {
        Some(i) => match args.get(i + 1) {
            Some(dir) => Some(PathBuf::from(dir)),
            None => {
                eprintln!("{USAGE}");
                return 2;
            }
        },
        None => None,
    };
    let s = Sources::repo();
    let run = || -> Result<i32, String> {
        let reference = match std::fs::read_to_string(s.root.join(REFERENCE)) {
            Ok(text) => Some(Reference::parse(&text)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("{REFERENCE}: {e}")),
        };
        let objects = objects(Path::new(&s.paths.content))?;
        let (text, files, code) = match args.first().map(String::as_str) {
            Some("sizes") => {
                let Some(reference) = reference else {
                    eprintln!("no {REFERENCE} yet: `cargo review sizes` compares with it");
                    return Ok(2);
                };
                let measures = checkglb::measures(
                    &s.paths,
                    &tops(&objects, &reference),
                    &model_seats(&objects, &reference),
                )?;
                let mut rows = size_rows(&objects, &measures, &reference);
                measure_plants(&s, &mut rows, &reference)?;
                let (mut text, files, code) = report(&objects, &rows, &reference);
                let manifest = serde_json::from_str(&s.read(surfaces::MANIFEST)?)
                    .map_err(|e| format!("{}: {e}", surfaces::MANIFEST))?;
                let surface_rows =
                    surfaces::rows(&reference.surfaces, &manifest, &s.read(surfaces::ROADS)?)?;
                text.push_str(&surfaces::report(&surface_rows));
                (text, files, code.max(baselined(&s.paths, &rows, args)?))
            }
            Some("uses") => {
                let renderer =
                    Renderer::parse(&s.read("web/src/render/shared/characters/actions.ts")?)
                        .map_err(|e| format!("actions.ts: {e}"))?;
                let sounds = action_sounds(&s.read("web/src/audio/world.ts")?)
                    .map_err(|e| format!("world.ts: {e}"))?;
                let motions = motions(&s.paths)?;
                let (text, files, hard) =
                    uses(&objects, &motions, &renderer, &sounds, reference.as_ref());
                let sections =
                    use_sections(&objects, &motions, &renderer, &sounds, reference.as_ref());
                let counted = ratchet(&s.paths, "uses", &sections, args)?;
                let hard = if check { hard } else { Vec::new() };
                let code = if hard.is_empty() {
                    0
                } else {
                    eprintln!(
                        "review uses: {} hard flag(s), a use the renderer can't play or a model that opens without a clip:",
                        hard.len()
                    );
                    for line in hard.iter().take(20) {
                        eprintln!("  {line}");
                    }
                    1
                };
                (text, files, code.max(counted))
            }
            _ => {
                eprintln!("{USAGE}");
                return Ok(2);
            }
        };
        print!("{text}");
        if let Some(dir) = &md {
            write_md(dir, &files)?;
            eprintln!("wrote {} files to {}", files.len(), dir.display());
        }
        Ok(code)
    };
    run().unwrap_or_else(|e| {
        eprintln!("review: {e}");
        1
    })
}
