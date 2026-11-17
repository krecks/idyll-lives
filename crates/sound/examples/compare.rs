//! (For the port from TypeScript; removed with it.) Each sound `tools/audio/dump.ts` wrote, made here and held
//! against it: `cargo run -p sound --release --example compare -- <dir> [prefix] [tolerance]`. Prints each sound's
//! largest difference and how many samples differ at all; exits 1 if any is over the tolerance (default 1e-6) or
//! missing.

use sound::{Filter, Made, birdsong, blips, mono, rooms, steps};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = args.first().expect("compare <dir> [prefix] [tolerance]");
    let prefix = args.get(1).map(String::as_str).unwrap_or("");
    let tolerance: f64 = args.get(2).map(|t| t.parse().unwrap()).unwrap_or(1e-6);
    let manifest = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let names = parse(&manifest);
    let noise = |rate: f64| {
        let mut r = sound::dsp::Rng::new(99.0);
        (0..rate as usize).map(|_| (r.next() * 2.0 - 1.0) as f32).collect::<Vec<f32>>()
    };
    let (mut bad, mut checked) = (0, 0);
    for (name, channels, length) in names {
        if !name.starts_with(prefix) {
            continue;
        }
        let bytes = std::fs::read(format!("{dir}/{}.f32", name.replace('/', "__"))).unwrap();
        let want: Vec<f32> = bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
        let p: Vec<&str> = name.split('/').collect();
        let rate: f64 = p[1].parse().unwrap();
        let t = std::time::Instant::now();
        let got: Option<Vec<Vec<f32>>> = match p[0] {
            "make" => sound::make(p[2], rate).map(|m| match m {
                Made::Mono(x) => vec![x],
                Made::Stereo(s) => vec![s.left, s.right],
            }),
            "mono" => mono::make(p[2], rate).map(|x| vec![x]),
            "blip" => blips::make(p[2], rate).map(|x| vec![x]),
            "step" => steps::make(p[2], p[3], p[4].parse().unwrap(), rate).map(|x| vec![x]),
            "song" => {
                let v = birdsong::Voice { species: p[2].into(), seed: p[3].parse().unwrap(), pitch: p[4].parse().unwrap(), pace: p[5].parse().unwrap(), near: p[6].parse().unwrap() };
                Some(vec![birdsong::song(&v, p[7].parse().unwrap(), rate)])
            }
            "room" => rooms::response(p[2], rate).map(|s| vec![s.left, s.right]),
            "filter" => {
                let kind = match p[2] {
                    "lowpass" => Filter::Lowpass,
                    "highpass" => Filter::Highpass,
                    _ => Filter::Bandpass,
                };
                Some(vec![sound::node_filter(&noise(rate), rate, kind, p[3].parse().unwrap(), p[4].parse().unwrap(), p[5] == "true")])
            }
            _ => None,
        };
        let ms = t.elapsed().as_millis();
        checked += 1;
        let Some(got) = got.filter(|g| !g.is_empty() && !g[0].is_empty()) else {
            println!("MISSING {name}");
            bad += 1;
            continue;
        };
        if got.len() != channels || got.iter().any(|c| c.len() != length) {
            println!("SHAPE   {name}: {} channels of {:?}, want {channels} of {length}", got.len(), got.iter().map(Vec::len).collect::<Vec<_>>());
            bad += 1;
            continue;
        }
        let (mut worst, mut differ, mut first) = (0.0f64, 0usize, None);
        for (i, (a, b)) in got.concat().iter().zip(&want).enumerate() {
            let d = (*a as f64 - *b as f64).abs();
            if d > 0.0 || a.is_nan() != b.is_nan() {
                differ += 1;
                first.get_or_insert(i);
            }
            if d > worst || d.is_nan() {
                worst = if d.is_nan() { f64::INFINITY } else { d };
            }
        }
        let ok = worst <= tolerance;
        bad += !ok as usize;
        println!("{} {name}: largest difference {worst:.3e}, {differ} of {} samples differ (first at {first:?}), {ms} ms", if ok { "ok     " } else { "DIFFERS" }, want.len());
    }
    println!("{checked} sounds, {bad} not within {tolerance:e}");
    std::process::exit(if bad > 0 || checked == 0 { 1 } else { 0 });
}

/// [(name, channels, length)] from the manifest (`[{"name": …, "channels": …, "length": …}, …]`).
fn parse(text: &str) -> Vec<(String, usize, usize)> {
    let field = |obj: &str, key: &str| -> String {
        let at = obj.find(&format!("\"{key}\":")).unwrap() + key.len() + 3;
        let rest = obj[at..].trim_start();
        if let Some(s) = rest.strip_prefix('"') {
            s[..s.find('"').unwrap()].to_string()
        } else {
            rest.chars().take_while(|c| c.is_ascii_digit()).collect()
        }
    };
    text.split('{')
        .skip(1)
        .map(|obj| (field(obj, "name"), field(obj, "channels").parse().unwrap(), field(obj, "length").parse().unwrap()))
        .collect()
}
