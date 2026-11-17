//! `cargo m1-table <run>`: the M1 estimate's report (tools/m1/run.sh, README.md), from the run's logs, as
//! markdown on stdout: per tier and world, the sim's ticks a second measured on this machine (natively, in the
//! container, under the M1's quota, and as wasm under Node and Bun), the M1's estimated from them, and the
//! margin against the 600 that 10× needs (the fastest speed the player picks at any hour; super speed runs as
//! fast as the machine can).
//!
//!   cargo m1-table bench-results/m1/<time>
//!
//! A log the run skipped (run.sh's SKIP) is taken from the newest earlier run that has it. Reads run.sh's
//! settings from the environment as it passes them: WATCH, HOURS, ROUNDS, TIERS, WORLDS_AT, M1_QUOTA.

use regex::Regex;
use std::path::Path;

/// The M1's single-core speed against the M3 Pro's (the plan's research): expected, and its band (sustained in
/// a fanless Air, and at best).
const RATIO: f64 = 0.72;
const BAND: [f64; 2] = [0.62, 0.78];
/// Ticks a second 10× needs (10 a step, 60 steps a second), and the tier rule's floor (`web/src/game/people.ts`
/// `TIER_FOR`): the same.
const NEED: f64 = 600.0;
const TIER_FLOOR: f64 = 600.0;
const KINDS: [&str; 3] = ["seaside", "river", "forest"];

/// run.sh's settings (its environment): each as given, empty too, else its default.
pub struct Settings {
    pub tiers: Vec<String>,
    pub at: Vec<usize>,
    pub watch: String,
    pub hours: String,
    pub rounds: String,
    pub quota: String,
}

impl Settings {
    pub fn from_env() -> Result<Settings, String> {
        Settings::from(|name| std::env::var(name).ok())
    }

    pub fn from(get: impl Fn(&str) -> Option<String>) -> Result<Settings, String> {
        let env = |name: &str, fallback: &str| get(name).unwrap_or_else(|| fallback.to_string());
        let words = |s: String| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        let at = words(env("WORLDS_AT", "0 1 2"))
            .iter()
            .map(|w| {
                w.parse::<usize>()
                    .map_err(|_| format!("WORLDS_AT: {w} is no world"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Settings {
            tiers: words(env("TIERS", "full half near")),
            at,
            watch: env("WATCH", "320"),
            hours: env("HOURS", "24"),
            rounds: env("ROUNDS", "1"),
            quota: env("M1_QUOTA", "7200"),
        })
    }
}

/// JavaScript's `Math.round`: halves up (-2.5 is -2).
fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

/// JavaScript's `v.toFixed(digits)`: halves away from zero on the exact value (12.25 is 12.3).
fn to_fixed(v: f64, digits: usize) -> String {
    let exact = format!("{:.80}", v.abs());
    let (int, frac) = exact.split_once('.').unwrap();
    let mut kept: Vec<u8> = format!("{int}{}", &frac[..digits]).into_bytes();
    if frac.as_bytes()[digits] >= b'5' {
        let mut i = kept.len();
        loop {
            if i == 0 {
                kept.insert(0, b'1');
                break;
            }
            i -= 1;
            if kept[i] == b'9' {
                kept[i] = b'0';
            } else {
                kept[i] += 1;
                break;
            }
        }
    }
    let s = String::from_utf8(kept).unwrap();
    let (i, f) = s.split_at(s.len() - digits);
    let sign = if v < 0.0 && s.bytes().any(|b| b != b'0') {
        "-"
    } else {
        ""
    };
    if digits == 0 {
        format!("{sign}{i}")
    } else {
        format!("{sign}{i}.{f}")
    }
}

/// A count: rounded, thousands apart (`toLocaleString('en-GB')`); none: a dash.
fn n(v: Option<f64>) -> String {
    match v {
        Some(v) if !v.is_nan() => {
            let r = js_round(v) as i64;
            let digits = r.unsigned_abs().to_string();
            let mut out = String::new();
            for (i, c) in digits.chars().enumerate() {
                if i > 0 && (digits.len() - i).is_multiple_of(3) {
                    out.push(',');
                }
                out.push(c);
            }
            format!("{}{out}", if r < 0 { "-" } else { "" })
        }
        _ => "–".into(),
    }
}

/// A fraction as a signed percentage (`+12 %`); none: a dash.
fn pc(v: Option<f64>) -> String {
    match v {
        None => "–".into(),
        Some(v) => format!(
            "{}{} %",
            if v >= 0.0 { "+" } else { "" },
            js_round(v * 100.0) as i64
        ),
    }
}

struct Bench {
    rate: f64,
    residents: Option<f64>,
}

struct Wasm {
    rate: f64,
    cpu: f64,
    worst: f64,
    at: String,
}

struct Row {
    est: f64,
    low: f64,
    m3: f64,
}

/// The report for the run in `dir` (`bench-results/m1/<time>`).
pub fn report(dir: &Path, s: &Settings) -> String {
    let runs = dir.parent().unwrap_or(Path::new("."));
    let tiers_dir = runs.join("../../bench-results/tiers");
    let base = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sorted = |d: &Path| -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(d)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    };
    // The file `name` of this run, else of the newest earlier run that has it.
    let log_of = |name: &str| -> Option<String> {
        if let Ok(text) = std::fs::read_to_string(dir.join(name)) {
            return Some(text);
        }
        let earlier = sorted(runs)
            .into_iter()
            .rfind(|d| *d < base && runs.join(d).join(name).exists())?;
        std::fs::read_to_string(runs.join(earlier).join(name)).ok()
    };
    let residents_re = Regex::new(r"(\d+) residents, grid").unwrap();
    let bench_re = Regex::new(
        r"(?m)^bench: rate (\d+) worst_hour (\d+) at (\d+) step_p99 ([\d.]+) full (\d+) full_most (\d+)",
    )
    .unwrap();
    // tiers.sh's best run of `label` on tier `t`'s world `k`, and the residents (of the first log: NaN if it has none).
    let bench = |label: &str, t: &str, k: usize| -> Option<Bench> {
        let d = tiers_dir.join(label);
        let suffix = if s.watch.is_empty() {
            String::new()
        } else {
            format!("-w{}", s.watch)
        };
        let prefix = format!("{t}{k}{suffix}-r");
        let mut best: Option<f64> = None;
        let mut residents: Option<f64> = None;
        for f in sorted(&d)
            .into_iter()
            .filter(|f| f.starts_with(&prefix) && f.ends_with(".log"))
        {
            let text = std::fs::read_to_string(d.join(&f)).unwrap_or_default();
            residents.get_or_insert_with(|| {
                residents_re
                    .captures(&text)
                    .map_or(f64::NAN, |c| c[1].parse().unwrap())
            });
            if let Some(m) = bench_re.captures(&text) {
                let rate: f64 = m[1].parse().unwrap();
                if best.is_none_or(|b| rate > b) {
                    best = Some(rate);
                }
            }
        }
        best.map(|rate| Bench { rate, residents })
    };
    let wasm_log = log_of("wasm.log").unwrap_or_default();
    let wasm_re = Regex::new(r"(?m)^(\w+) (\d+) simtick: engine (\w+) rate (\d+) worst_hour (\d+) at (\d+) step_p99 ([\d.]+)(?: ticks \d+ cpu_rate (\d+))?").unwrap();
    // simtick's best line for `engine` on tier `t`'s world `k`, by ticks a second of CPU time: what it ran, not
    // what it waited for a core on a busy machine.
    let wasm = |engine: &str, t: &str, k: usize| -> Option<Wasm> {
        let mut best: Option<Wasm> = None;
        for m in wasm_re.captures_iter(&wasm_log) {
            if &m[1] != t || m[2].parse::<usize>().ok() != Some(k) || &m[3] != engine {
                continue;
            }
            let rate: f64 = m[4].parse().unwrap();
            let cpu = m.get(8).map_or(rate, |c| c.as_str().parse().unwrap());
            if best.as_ref().is_none_or(|b| cpu > b.cpu) {
                best = Some(Wasm {
                    rate,
                    cpu,
                    worst: m[5].parse().unwrap(),
                    at: m[6].to_string(),
                });
            }
        }
        best
    };
    let calibration = |name: &str| -> Option<(f64, f64)> {
        let re = Regex::new(r"calibrate: 7z (\d+) tick_us ([\d.]+)").unwrap();
        let text = log_of(name)?;
        let c = re.captures(&text)?;
        Some((c[1].parse().unwrap(), c[2].parse().unwrap()))
    };
    let (m3, m1) = (
        calibration("calibrate-m3.log"),
        calibration("calibrate-m1.log"),
    );

    let mut out: Vec<String> = Vec::new();
    let mut say = |line: String| out.push(line);
    say(format!("# M1 estimate, {base}"));
    say(String::new());
    let view = if s.watch.is_empty() {
        "the view on the player's lot".to_string()
    } else {
        format!(
            "the camera's {}-tile square round the player's home at full detail at 10× (big_world's WATCH, simtick's --watch)",
            s.watch
        )
    };
    say(format!(
        "The sim's ticks a second over {} game hours, {view}; best of {} each.",
        s.hours, s.rounds
    ));
    say(format!(
        "10× needs {}; a preset gets a tier when the estimated wasm rate on its slowest intended machine is at least {}.",
        n(Some(NEED)),
        n(Some(TIER_FLOOR))
    ));
    say(String::new());
    say("## Calibration".into());
    say(String::new());
    if let (Some(m3), Some(m1)) = (m3, m1) {
        let quota: f64 = s.quota.trim().parse().unwrap_or(f64::NAN);
        say("| | 7-Zip 1 thread (MIPS) | tickbench (µs a tick) |".into());
        say("|---|---:|---:|".into());
        say(format!(
            "| container (m3) | {} | {} |",
            n(Some(m3.0)),
            to_fixed(m3.1, 1)
        ));
        say(format!(
            "| under the quota (m1, {} %) | {} | {} |",
            to_fixed(quota / 100.0, 0),
            n(Some(m1.0)),
            to_fixed(m1.1, 1)
        ));
        say(format!(
            "| ratio | {} | {} |",
            to_fixed(m1.0 / m3.0, 2),
            to_fixed(m3.1 / m1.1, 2)
        ));
    } else {
        say("No calibration logs.".into());
    }
    say(String::new());
    say(format!(
        "The estimate scales this machine's numbers by {RATIO} (an M1's single core against an M3 Pro's), with a band of {} (sustained, fanless) to {}. The quota's own ratio above shows the container's \"M1\" runs at about that.",
        BAND[0], BAND[1]
    ));
    if let Some(limits) = log_of("limits.log").filter(|l| !l.is_empty()) {
        say(format!("The m1 container: {}.", limits.trim()));
    }
    say(String::new());
    say("## Ticks a second".into());
    say(String::new());
    say("Wasm: ticks a second by the wall clock, and (in brackets) of the process's CPU time, which leaves out the waits for a core on a busy machine; the M1 wasm estimate scales the latter.".into());
    say(String::new());
    say(format!(
        "| tier | world | residents | M3 native | container | under quota | wasm Node | wasm Bun | M1 native est. | M1 wasm est. (band) | margin vs {} | slowest hour (M1 wasm est.) |",
        n(Some(NEED))
    ));
    say("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|".into());
    let mut verdict: Vec<(String, Vec<Row>)> = Vec::new();
    for t in &s.tiers {
        let mut rows = Vec::new();
        for &k in &s.at {
            let native = bench("m1-native", t, k);
            let ctr = bench("m1-ctr-m3", t, k);
            let quota_run = bench("m1-ctr-m1", t, k);
            let node = wasm("node", t, k);
            let bun = wasm("bun", t, k);
            let est = node.as_ref().map(|w| w.cpu * RATIO);
            let band = node.as_ref().map(|w| BAND.map(|r| w.cpu * r));
            // (The slowest hour by the wall clock, scaled as the CPU time scaled the run.)
            let worst = node.as_ref().map(|w| w.worst * (w.cpu / w.rate) * RATIO);
            if let (Some(est), Some(band), Some(w)) = (est, band, node.as_ref()) {
                rows.push(Row {
                    est,
                    low: band[0],
                    m3: w.cpu,
                });
            }
            let residents = match &native {
                None => "–".to_string(),
                Some(b) => match b.residents {
                    Some(r) if !r.is_nan() => format!("{r}"),
                    _ => "NaN".into(),
                },
            };
            let pair = |w: &Option<Wasm>| {
                format!(
                    "{} ({})",
                    n(w.as_ref().map(|w| w.rate)),
                    n(w.as_ref().map(|w| w.cpu))
                )
            };
            say(format!(
                "| {t} | {} | {residents} | {} | {} | {} | {} | {} | {} | {} ({}) | {} | {} ({}:00) |",
                KINDS.get(k).copied().unwrap_or("undefined"),
                n(native.as_ref().map(|b| b.rate)),
                n(ctr.as_ref().map(|b| b.rate)),
                n(quota_run.as_ref().map(|b| b.rate)),
                pair(&node),
                pair(&bun),
                n(native.as_ref().map(|b| b.rate * RATIO)),
                n(est),
                band.map_or("–".to_string(), |b| format!(
                    "{}–{}",
                    n(Some(b[0])),
                    n(Some(b[1]))
                )),
                pc(est.map(|e| e / NEED - 1.0)),
                n(worst),
                node.as_ref().map_or("–", |w| w.at.as_str()),
            ));
        }
        verdict.push((t.clone(), rows));
    }
    say(String::new());
    say("## Per tier (the slowest world)".into());
    say(String::new());
    say(format!(
        "| tier | M3 wasm | M1 wasm est. | at the band's low end | ≥ {} on an M1? | on an M3? |",
        n(Some(TIER_FLOOR))
    ));
    say("|---|---:|---:|---:|---|---|".into());
    let yes = |b: bool| if b { "yes" } else { "no" };
    for (t, rows) in &verdict {
        let Some(slowest) = rows.iter().reduce(|a, r| if r.est < a.est { r } else { a }) else {
            continue;
        };
        say(format!(
            "| {t} | {} | {} | {} | {} | {} |",
            n(Some(slowest.m3)),
            n(Some(slowest.est)),
            n(Some(slowest.low)),
            yes(slowest.est >= TIER_FLOOR),
            yes(slowest.m3 >= TIER_FLOOR)
        ));
    }
    say(String::new());
    say("## What this cannot say".into());
    say(String::new());
    for line in [
        "- **The GPU and the frame time.** Docker has no GPU: the renderer is not run, and the sim worker shares the M1 with it and the main thread in the game.",
        "- **Unified-memory pressure.** The quota container is given 8 GB (as much as Docker Desktop has), but an M1 shares its 8 GB with the GPU, the browser and the system; swapping is not modelled.",
        "- **Thermals.** A fanless M1 Air throttles under a sustained load; the band's low end stands for that, not a measurement.",
        "- **The browser.** Node runs the same V8 as Chrome, but not Safari's or Firefox's engines; the worker's messaging and the UI updates are not counted.",
        "- **A shared machine.** Other work on this machine slows every run; the best run of each is kept, but the numbers are as good as the quietest of them.",
    ] {
        say(line.into());
    }
    out.join("\n") + "\n"
}

/// `cargo m1-table <run>`: the exit code.
pub fn main(args: &[String]) -> i32 {
    let Some(dir) = args.first() else {
        eprintln!("m1-table: which run? (cargo m1-table bench-results/m1/<time>)");
        return 2;
    };
    match Settings::from_env() {
        Ok(s) => {
            print!("{}", report(Path::new(dir), &s));
            0
        }
        Err(err) => {
            eprintln!("m1-table: {err}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_fixed_rounds_halves_up_as_javascript_does() {
        assert_eq!(to_fixed(12.25, 1), "12.3");
        assert_eq!(to_fixed(0.125, 2), "0.13");
        assert_eq!(to_fixed(17.05, 1), "17.1");
        assert_eq!(to_fixed(9.96, 1), "10.0");
        assert_eq!(to_fixed(72.0, 0), "72");
        assert_eq!(to_fixed(-0.25, 1), "-0.3");
        assert_eq!(to_fixed(-0.01, 1), "0.0");
    }

    #[test]
    fn counts_and_percentages_round_as_javascript_does() {
        assert_eq!(n(Some(1234567.5)), "1,234,568");
        assert_eq!(n(Some(-2.5)), "-2");
        assert_eq!(n(Some(999.4)), "999");
        assert_eq!(n(None), "–");
        assert_eq!(n(Some(f64::NAN)), "–");
        assert_eq!(pc(Some(-0.125)), "-12 %");
        assert_eq!(pc(Some(4.16)), "+416 %");
    }
}
