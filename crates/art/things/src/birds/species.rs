//! The species. Length bill to tail tip and wingspan (m) are the field guides'; the rest shapes it: `body` the
//! torso's length, `width`/`depth` across it, `body_at` how far the body's middle stands ahead of the feet, `leg` the
//! leg showing, `pitch` the torso's rise from level (degrees), `head` its radius, `neck` how far the head's middle
//! stands off the neck (head radii) at `neck_up` degrees, `bill`, `bill_w`, `bill_h`, `bill_droop`, `bill_low`, `eye`
//! (head radii), `eye_at` how far forward the eye is (-1 .. 1 along the head), `tail_down` (degrees below level),
//! `tip_over` how far down the tail the folded wingtips reach, `chord` the spread wing's depth at the body,
//! `planform` its outline [(along the span 0 .. 1, leading edge, trailing edge: chords forward of the shoulder)],
//! `tail` its shape (`SQUARE` ...) and `tail_end` its end (a notch; negative: rounded), `leg_r` and `toe` the legs'
//! thickness and the toes' length, `ring` an eye ring, `rings` the points round the body, the head and a spread wing
//! (10, 10, 5; the bigger birds 12, 12, 6), `stations` the body's (`STATIONS`), and `zones`: which material each face
//! of the body (by `t` rump 0 .. neck 1 and degrees round from the top), the head (by `s` back -1 .. front 1 and
//! degrees), the folded wing (outer face and lower edge, for each band from the shoulder to the tip), the tail (by how
//! far along and 'top', 'outer' or 'under') and the spread wing (by how far out, 0 .. 1, and 'lead' (the coverts),
//! 'bar', 'flight' or 'under') takes. Face centres round from the top: the body's 0, 36, 72, 108, 144, 180 (12 round:
//! 0, 30 ... 180); the head's 0, 32, 57, 84, 116, 180 (12 round: 0, 24, 44, 63, 85, 113, 180).

/// A zone of the body or the head by where a face is (along, degrees round from the top).
pub type Zone = fn(f64, f64) -> &'static str;
/// A zone of the tail or the spread wing by how far along or out and which face or band.
pub type Part = fn(f64, &str) -> &'static str;

pub struct Zones {
    pub body: Zone,
    pub head: Zone,
    pub folded: [(&'static str, &'static str); 6],
    pub tail: Option<Part>,
    pub wing: Part,
}

pub struct Species {
    pub name: &'static str,
    pub length: f64,
    pub wingspan: f64,
    pub body: f64,
    pub width: f64,
    pub depth: f64,
    pub body_at: f64,
    pub leg: f64,
    pub pitch: i32,
    pub head: f64,
    pub neck: f64,
    pub neck_up: f64,
    pub bill: f64,
    pub bill_w: f64,
    pub bill_h: f64,
    pub bill_droop: f64,
    pub bill_low: f64,
    pub eye: f64,
    pub eye_at: f64,
    pub ring: bool,
    pub leg_r: f64,
    pub toe: f64,
    pub tail_down: f64,
    pub tip_over: f64,
    pub chord: f64,
    pub planform: &'static [(f64, f64, f64)],
    pub tail: &'static [(f64, f64, f64)],
    pub tail_end: f64,
    pub rings: (usize, usize, usize),
    pub stations: Option<&'static [f64]>,
    pub zones: Zones,
}

// Tails: a tit's and a finch's square-ended and notched, a warbler's short, a blackbird's long and round-ended, a
// dove's long and broad, rounded.
const SQUARE: &[(f64, f64, f64)] = &[
    (0.0, 0.16, 0.22),
    (0.45, 0.27, 0.11),
    (0.8, 0.33, 0.06),
    (1.0, 0.34, 0.04),
];
const LONG: &[(f64, f64, f64)] = &[
    (0.0, 0.15, 0.22),
    (0.4, 0.22, 0.1),
    (0.82, 0.26, 0.05),
    (1.0, 0.23, 0.035),
];
const BROAD: &[(f64, f64, f64)] = &[
    (0.0, 0.17, 0.2),
    (0.4, 0.27, 0.08),
    (0.8, 0.34, 0.045),
    (1.0, 0.33, 0.03),
];

// Spread wings: (along the span, leading edge, trailing edge) in chords forward of the shoulder. The trailing edge
// steps in and out, a feather's end and the notch to the next: the secondaries along the arm, then the primaries
// fanning from the wrist (where the leading edge is furthest forward) to the tip. A songbird's is round-ended, a
// dove's long and pointed.
const ROUNDED: &[(f64, f64, f64)] = &[
    (0.0, 0.2, -0.84),
    (0.13, 0.25, -0.93),
    (0.25, 0.27, -0.85),
    (0.37, 0.28, -0.94),
    (0.47, 0.27, -0.86),
    (0.58, 0.19, -0.9),
    (0.68, 0.11, -0.74),
    (0.78, 0.03, -0.78),
    (0.87, -0.05, -0.6),
    (0.94, -0.12, -0.6),
    (1.0, -0.22, -0.42),
];
const POINTED: &[(f64, f64, f64)] = &[
    (0.0, 0.2, -0.85),
    (0.13, 0.25, -0.92),
    (0.25, 0.27, -0.84),
    (0.37, 0.27, -0.88),
    (0.47, 0.24, -0.78),
    (0.58, 0.15, -0.76),
    (0.68, 0.06, -0.62),
    (0.78, -0.04, -0.6),
    (0.87, -0.16, -0.5),
    (0.94, -0.28, -0.52),
    (1.0, -0.44, -0.5),
];

/// A spread wing's band: the coverts (`lead`), the bar, the flight feathers, or under it.
fn band(
    part: &str,
    lead: &'static str,
    bar: &'static str,
    flight: &'static str,
    under: &'static str,
) -> &'static str {
    match part {
        "lead" => lead,
        "bar" => bar,
        "flight" => flight,
        "under" => under,
        _ => panic!("no band {part:?} on a spread wing"),
    }
}

fn tit_body(t: f64, deg: f64) -> &'static str {
    if t > 0.86 && deg < 130.0 {
        return "nape"; // the neck: the dark blue collar round to the black bib
    }
    if deg < 50.0 {
        return "back";
    }
    if deg < 100.0 {
        return if t > 0.32 { "breast" } else { "back" };
    }
    if deg > 160.0 && 0.46 < t && t < 0.72 {
        return "mask"; // the dark line down the belly
    }
    if t > 0.28 { "breast" } else { "belly" }
}

fn tit_head(s: f64, deg: f64) -> &'static str {
    if deg < 70.0 && s > 0.55 {
        return "brow"; // the white forehead
    }
    if deg < 45.0 {
        return if s > -0.6 { "cap" } else { "brow" }; // the blue crown, ringed white behind
    }
    if deg < 70.0 {
        return if s > -0.6 { "brow" } else { "nape" }; // the white band over the eye, to the nape
    }
    if deg < 100.0 {
        return "mask"; // the black stripe from the bill through the eye to the nape
    }
    if deg < 150.0 {
        return if s > -0.5 { "face" } else { "nape" }; // the white cheek, the dark blue collar behind it
    }
    "throat" // the black bib
}

pub const BLUE_TIT: Species = Species {
    name: "blue tit",
    length: 0.12,
    wingspan: 0.185,
    body: 0.058,
    width: 0.036,
    depth: 0.04,
    body_at: 0.004,
    leg: 0.011,
    pitch: 34,
    head: 0.0145,
    neck: 0.55,
    neck_up: 60.0,
    bill: 0.0045,
    bill_w: 0.0045,
    bill_h: 0.0042,
    bill_droop: 4.0,
    bill_low: 0.1,
    eye: 0.2,
    eye_at: 0.3,
    ring: false,
    leg_r: 0.0012,
    toe: 0.009,
    tail_down: 28.0,
    tip_over: 0.32,
    chord: 0.042,
    planform: ROUNDED,
    tail: SQUARE,
    tail_end: 0.04,
    rings: (10, 10, 5),
    stations: None,
    zones: Zones {
        body: tit_body,
        head: tit_head,
        folded: [
            ("wing", "wing"),
            ("wing", "wing"),
            ("bar", "bar"),
            ("flight", "edge"),
            ("flight", "edge"),
            ("flight", "flight"),
        ],
        tail: None,
        wing: |u, part| {
            band(
                part,
                if u < 0.6 { "wing" } else { "flight" },
                if u < 0.52 { "bar" } else { "flight" },
                "flight",
                "under",
            )
        },
    },
};
pub const BLUE_TIT_COLOURS: &[(&str, &str)] = &[
    ("back", "#93A35C"),
    ("rump", "#8FA864"),
    ("cap", "#3D7ED0"),
    ("brow", "#F4F4F0"),
    ("mask", "#1C2438"),
    ("face", "#F6F6F2"),
    ("throat", "#1C2438"),
    ("nape", "#24427E"),
    ("breast", "#F4D234"),
    ("belly", "#E9DC7E"),
    ("wing", "#4683CC"),
    ("bar", "#F2F2EC"),
    ("flight", "#4E6C98"),
    ("edge", "#6E8EBC"),
    ("tip", "#3E5578"),
    ("under", "#CED2CE"),
    ("tail", "#3D70B4"),
    ("bill", "#2A2A2C"),
    ("eye", "#101010"),
    ("leg", "#6A7A92"),
];

fn finch_body(t: f64, deg: f64) -> &'static str {
    if deg < 50.0 {
        if t > 0.86 {
            return "nape"; // the blue-grey nape
        }
        return if t > 0.22 { "back" } else { "rump" }; // the chestnut mantle, the olive-green rump
    }
    if deg < 100.0 {
        return if t > 0.32 { "breast" } else { "belly" };
    }
    if t > 0.45 { "breast" } else { "belly" }
}

fn finch_head(s: f64, deg: f64) -> &'static str {
    if deg < 50.0 && s > 0.55 {
        return "mask"; // the black forehead over the bill
    }
    if deg < 70.0 {
        return "cap"; // the blue-grey crown and nape
    }
    if deg < 100.0 {
        return if s > -0.4 { "face" } else { "cap" };
    }
    // the rusty-pink cheek and throat
    if deg < 150.0 { "face" } else { "throat" }
}

pub const CHAFFINCH: Species = Species {
    name: "chaffinch",
    length: 0.145,
    wingspan: 0.26,
    body: 0.064,
    width: 0.04,
    depth: 0.042,
    body_at: 0.004,
    leg: 0.013,
    pitch: 27,
    head: 0.0158,
    neck: 0.55,
    neck_up: 55.0,
    bill: 0.0085,
    bill_w: 0.0068,
    bill_h: 0.0068,
    bill_droop: 6.0,
    bill_low: 0.12,
    eye: 0.19,
    eye_at: 0.28,
    ring: false,
    leg_r: 0.0013,
    toe: 0.011,
    tail_down: 22.0,
    tip_over: 0.3,
    chord: 0.054,
    planform: ROUNDED,
    tail: SQUARE,
    tail_end: 0.05,
    rings: (10, 10, 5),
    stations: None,
    zones: Zones {
        body: finch_body,
        head: finch_head,
        // the white shoulder, the black coverts, the white bar, the black flight feathers' yellowish edges
        folded: [
            ("bar", "bar"),
            ("wing", "wing"),
            ("bar", "bar"),
            ("flight", "edge"),
            ("flight", "edge"),
            ("flight", "flight"),
        ],
        tail: Some(|k, part| {
            if part == "outer" && k > 0.45 {
                "outer"
            } else {
                "tail"
            }
        }),
        wing: |u, part| {
            band(
                part,
                if u < 0.3 { "bar" } else { "wing" },
                if 0.13 < u && u < 0.58 { "bar" } else { "wing" },
                if 0.58 < u && u < 0.72 {
                    "edge"
                } else {
                    "flight"
                },
                "under",
            )
        },
    },
};
pub const CHAFFINCH_COLOURS: &[(&str, &str)] = &[
    ("back", "#8C6448"),
    ("rump", "#7E8A50"),
    ("cap", "#6C88B0"),
    ("nape", "#6C88B0"),
    ("mask", "#1E1E20"),
    ("face", "#C98476"),
    ("throat", "#CB8676"),
    ("breast", "#CC8274"),
    ("belly", "#EEDCD2"),
    ("wing", "#33302E"),
    ("bar", "#F2F0EA"),
    ("flight", "#3C3632"),
    ("edge", "#C8C49A"),
    ("under", "#DAD4CC"),
    ("tail", "#3A3634"),
    ("outer", "#E8E6E0"),
    ("bill", "#8C9CAC"),
    ("eye", "#101010"),
    ("leg", "#8C7262"),
];

fn warbler_body(t: f64, deg: f64) -> &'static str {
    if deg < 50.0 {
        return "back";
    }
    if deg < 100.0 {
        return if t > 0.4 { "breast" } else { "back" };
    }
    if t > 0.5 { "breast" } else { "belly" }
}

fn warbler_head(s: f64, deg: f64) -> &'static str {
    if deg < 45.0 {
        return "cap";
    }
    if deg < 70.0 {
        return if s > -0.4 { "brow" } else { "cap" }; // the pale yellow eyebrow
    }
    if deg < 100.0 {
        return if s > -0.55 { "mask" } else { "cap" }; // the dusky line through the eye
    }
    if deg < 150.0 { "face" } else { "throat" }
}

pub const WARBLER: Species = Species {
    name: "willow warbler",
    length: 0.11,
    wingspan: 0.19,
    body: 0.05,
    width: 0.03,
    depth: 0.032,
    body_at: 0.004,
    leg: 0.013,
    pitch: 26,
    head: 0.0124,
    neck: 0.55,
    neck_up: 50.0,
    bill: 0.0068,
    bill_w: 0.003,
    bill_h: 0.0028,
    bill_droop: 3.0,
    bill_low: 0.1,
    eye: 0.2,
    eye_at: 0.3,
    ring: false,
    leg_r: 0.0011,
    toe: 0.009,
    tail_down: 18.0,
    tip_over: 0.3,
    chord: 0.04,
    planform: ROUNDED,
    tail: SQUARE,
    tail_end: 0.03,
    rings: (10, 10, 5),
    stations: None,
    zones: Zones {
        body: warbler_body,
        head: warbler_head,
        folded: [
            ("wing", "wing"),
            ("wing", "wing"),
            ("wing", "wing"),
            ("flight", "edge"),
            ("flight", "edge"),
            ("flight", "flight"),
        ],
        tail: None,
        wing: |u, part| {
            band(
                part,
                "wing",
                if u < 0.55 { "wing" } else { "flight" },
                "flight",
                "under",
            )
        },
    },
};
pub const WARBLER_COLOURS: &[(&str, &str)] = &[
    ("back", "#86985A"),
    ("cap", "#86985A"),
    ("brow", "#ECE6A0"),
    ("mask", "#646A40"),
    ("face", "#DCD994"),
    ("throat", "#F0EAAA"),
    ("breast", "#EEE6A4"),
    ("belly", "#F2F0DC"),
    ("wing", "#7C8A58"),
    ("flight", "#6C764E"),
    ("edge", "#8E9C62"),
    ("under", "#E6E2C0"),
    ("tail", "#6E7852"),
    ("bill", "#8A7A5C"),
    ("eye", "#101010"),
    ("leg", "#B89A7A"),
];

fn blackbird_body(t: f64, deg: f64) -> &'static str {
    if deg < 100.0 {
        return "back";
    }
    if t > 0.4 { "breast" } else { "belly" }
}

fn blackbird_head(_s: f64, deg: f64) -> &'static str {
    if deg < 110.0 { "cap" } else { "throat" }
}

pub const BLACKBIRD: Species = Species {
    name: "blackbird",
    length: 0.25,
    wingspan: 0.36,
    body: 0.112,
    width: 0.066,
    depth: 0.07,
    body_at: 0.008,
    leg: 0.03,
    pitch: 20,
    head: 0.0235,
    neck: 0.62,
    neck_up: 45.0,
    bill: 0.019,
    bill_w: 0.008,
    bill_h: 0.0085,
    bill_droop: 6.0,
    bill_low: 0.1,
    eye: 0.17,
    eye_at: 0.28,
    ring: true,
    leg_r: 0.0022,
    toe: 0.02,
    tail_down: 14.0,
    tip_over: 0.3,
    chord: 0.085,
    planform: ROUNDED,
    tail: LONG,
    tail_end: -0.05,
    rings: (12, 12, 6),
    stations: None,
    zones: Zones {
        body: blackbird_body,
        head: blackbird_head,
        folded: [
            ("wing", "wing"),
            ("wing", "wing"),
            ("wing", "wing"),
            ("flight", "edge"),
            ("flight", "edge"),
            ("flight", "flight"),
        ],
        tail: None,
        wing: |u, part| {
            band(
                part,
                "wing",
                if u < 0.55 { "wing" } else { "flight" },
                "flight",
                "under",
            )
        },
    },
};
pub const BLACKBIRD_COLOURS: &[(&str, &str)] = &[
    ("back", "#1C1C1E"),
    ("cap", "#1C1C1E"),
    ("throat", "#202022"),
    ("breast", "#1F1F21"),
    ("belly", "#232324"),
    ("wing", "#202020"),
    ("flight", "#28251F"),
    ("edge", "#2E2B26"),
    ("under", "#2C2A27"),
    ("tail", "#1C1C1C"),
    ("bill", "#F2A21E"),
    ("eye", "#0E0E0E"),
    ("ring", "#F4BC2A"),
    ("leg", "#3C3228"),
];

fn dove_body(t: f64, deg: f64) -> &'static str {
    if 0.89 < t && t < 0.94 && deg < 100.0 {
        return "collar"; // the black half-collar round the back and sides of the neck
    }
    if 0.94 < t && t < 0.975 && deg < 100.0 {
        return "brow"; // its thin white edge
    }
    if deg < 50.0 {
        return if t < 0.82 { "back" } else { "cap" };
    }
    if deg < 100.0 {
        return if t > 0.4 { "breast" } else { "belly" };
    }
    if t > 0.45 { "breast" } else { "belly" }
}

fn dove_head(_s: f64, deg: f64) -> &'static str {
    if deg < 100.0 { "cap" } else { "throat" }
}

fn dove_tail(k: f64, part: &str) -> &'static str {
    if part == "under" {
        // black at the base underneath, a broad white end
        return if k < 0.55 { "undertail" } else { "tailtip" };
    }
    if k > 0.82 || (part == "outer" && k > 0.6) {
        "tailtip"
    } else {
        "tail"
    }
}

pub const COLLARED_DOVE: Species = Species {
    name: "collared dove",
    length: 0.32,
    wingspan: 0.51,
    body: 0.145,
    width: 0.088,
    depth: 0.088,
    body_at: 0.012,
    leg: 0.018,
    pitch: 16,
    head: 0.024,
    neck: 0.62,
    neck_up: 58.0,
    bill: 0.012,
    bill_w: 0.006,
    bill_h: 0.0055,
    bill_droop: 12.0,
    bill_low: 0.15,
    eye: 0.17,
    eye_at: 0.25,
    ring: false,
    leg_r: 0.0022,
    toe: 0.02,
    tail_down: 10.0,
    tip_over: 0.42,
    chord: 0.115,
    planform: POINTED,
    tail: BROAD,
    tail_end: -0.06,
    rings: (12, 12, 6),
    stations: Some(&[
        0.0, 0.05, 0.14, 0.28, 0.46, 0.64, 0.8, 0.89, 0.94, 0.965, 1.0,
    ]),
    zones: Zones {
        body: dove_body,
        head: dove_head,
        tail: Some(dove_tail),
        folded: [
            ("wing", "wing"),
            ("wing", "wing"),
            ("bar", "bar"),
            ("flight", "flight"),
            ("flight", "tip"),
            ("tip", "tip"),
        ],
        // the buff coverts, a blue-grey band along the arm, the dark primaries
        wing: |u, part| {
            band(
                part,
                if u < 0.5 { "wing" } else { "tip" },
                if u < 0.5 { "bar" } else { "tip" },
                if u < 0.5 { "flight" } else { "tip" },
                if u < 0.75 { "under" } else { "flight" },
            )
        },
    },
};
pub const COLLARED_DOVE_COLOURS: &[(&str, &str)] = &[
    ("back", "#BBA88E"),
    ("cap", "#CFC2B6"),
    ("brow", "#ECE8E2"),
    ("throat", "#D6C8BC"),
    ("collar", "#1C1A1A"),
    ("breast", "#D8C2B8"),
    ("belly", "#DCD4CC"),
    ("wing", "#C2B29C"),
    ("bar", "#A8ACB0"),
    ("flight", "#9C9894"),
    ("tip", "#3E3A38"),
    ("under", "#DADAD8"),
    ("tail", "#AA9E8E"),
    ("tailtip", "#EEEAE4"),
    ("undertail", "#2A2828"),
    ("bill", "#2C2A2A"),
    ("eye", "#6E1E1E"),
    ("leg", "#B65C5C"),
];
