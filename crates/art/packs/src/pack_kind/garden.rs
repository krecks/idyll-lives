//! The songbird feeding station (1x1): a feeder house on a post, a bird bath, songbirds.

use super::common::{M, WALL, bush, name, songbird, tube, turned};
use artkit::forms::rounded_rect;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::motion::{Grow, grow};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, soft, swing, torus};

const POST: (f64, f64) = (-0.2, -0.22); // x, depth
const HOUSE_Y: f64 = 1.48; // the feeder house's middle

pub fn songbird_feeder() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let none = (0.0, 0.0, 0.0);
    let (px, pz) = POST;
    let still = new_parts(|| {
        fbox!(
            f,
            "post_foot",
            0.3,
            0.04,
            0.3,
            (px, 0.02, pz),
            M.stone,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "post_plinth",
            0.12,
            0.06,
            0.12,
            (px, 0.07, pz),
            M.post,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "post",
            0.07,
            1.31,
            0.07,
            (px, 0.71, pz),
            M.post,
            bevel = 0.008,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            // brackets under the tray, out from the post's four faces
            let arm = [
                (0.0, 0.0),
                (0.0, 0.06),
                (s * 0.12, 0.06),
                (s * 0.11, 0.045),
                (s * 0.03, 0.01),
            ];
            fextrude!(
                f,
                name("tray_bracket"),
                &arm,
                0.016,
                (px + s * 0.035, 1.298, pz),
                M.post,
                plane = Facing::Front
            );
            fextrude!(
                f,
                name("tray_bracket"),
                &arm,
                0.016,
                (px, 1.298, pz + s * 0.035),
                M.post,
                plane = Facing::Side
            );
        }
        fbox!(
            f,
            "seed_tray",
            0.38,
            0.02,
            0.32,
            (px, 1.368, pz),
            M.tray,
            bevel = 0.004,
            bevel_segments = 1
        );
        let lp: Vec<(f64, f64, f64)> = rounded_rect(0.37, 0.31, 0.012, 2)
            .iter()
            .map(|&(u, v)| (u + px, 1.385, v + pz))
            .collect();
        fsweep!(
            f,
            "tray_lip",
            &lp,
            0,
            M.tray,
            profile = rounded_rect(0.024, 0.012, 0.004, 1),
            up = (0, 0, 1),
            closed = true
        );
        for (x, z) in [
            (-0.31, -0.12),
            (-0.12, -0.3),
            (-0.27, -0.31),
            (-0.13, -0.13),
            (-0.3, -0.33),
        ] {
            fsphere!(
                f,
                name("seeds"),
                0.025,
                (x, 1.383, z),
                M.seed,
                scale = (1, 1, 0.35),
                segments = 8,
                rings = 4
            );
        }
        // the feeder house: walls with a clear hopper of seed, a round door with a perch, corner trims
        fbox!(
            f,
            "house",
            0.22,
            0.2,
            0.18,
            (px, HOUSE_Y, pz),
            M.feeder,
            bevel = 0.006,
            bevel_segments = 1
        );
        for sx in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                fbox!(
                    f,
                    name("house_trim"),
                    0.018,
                    0.2,
                    0.018,
                    (px + sx * 0.105, HOUSE_Y, pz + sz * 0.085),
                    M.post,
                    bevel = 0.003,
                    bevel_segments = 1
                );
            }
        }
        fbox!(
            f,
            "house_hopper",
            0.16,
            0.07,
            0.004,
            (px, 1.42, pz + 0.091),
            M.water,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "hopper_seed",
            0.15,
            0.045,
            0.003,
            (px, 1.41, pz + 0.0905),
            M.seed
        );
        fcyl!(
            f,
            "house_hole",
            0.033,
            0.006,
            (px, 1.51, pz + 0.091),
            M.dark,
            rot = (90, 0, 0),
            verts = 16
        );
        torus!(
            "house_hole_ring",
            0.035,
            0.005,
            at = f.at(px, 1.51, pz + 0.093),
            material = M.post,
            rot = (90, 0, 0),
            major_segments = 16,
            minor_segments = 3
        );
        tube(
            f,
            "perch",
            &[(px, 1.465, pz + 0.09), (px, 1.465, pz + 0.16)],
            0.006,
            M.tray,
            Some(6),
            1,
        );
        songbird(
            f,
            "robin",
            px,
            1.471,
            pz + 0.15,
            M.bird_brown,
            M.bird_orange,
            M.mustard,
            Some(M.dark),
            0.0,
            0.9,
            0.0,
            false,
        );
        // the bird bath: a fluted stem, a dished bowl with a rolled rim, its water
        turned(
            f,
            "bath_stand",
            0.2,
            0.0,
            0.16,
            &[
                (0.0, 0.0),
                (0.12, 0.0),
                (0.12, 0.02),
                (0.1, 0.03),
                (0.06, 0.06),
                (0.045, 0.1),
                (0.042, 0.31),
                (0.05, 0.5),
                (0.06, 0.55),
                (0.1, 0.6),
                (0.0, 0.6),
            ],
            M.stone,
            20,
            none,
        );
        turned(
            f,
            "bath_bowl",
            0.2,
            0.0,
            0.16,
            &[
                (0.0, 0.6),
                (0.15, 0.6),
                (0.24, 0.66),
                (0.252, 0.69),
                (0.245, 0.708),
                (0.228, 0.71),
                (0.215, 0.695),
                (0.18, 0.67),
                (0.0, 0.665),
            ],
            M.stone,
            32,
            none,
        );
        fcyl!(
            f,
            "bath_water",
            0.212,
            0.008,
            (0.2, 0.688, 0.16),
            M.water,
            verts = 28
        );
        // flowering shrubs, a seed sack with its top rolled down, a scoop tin with its scoop
        for (k, (x, z, m)) in [
            (-0.34, 0.02, M.rose_flower),
            (-0.05, -0.36, M.mustard),
            (0.34, -0.3, M.lilac),
            (-0.34, 0.33, M.rose_flower),
        ]
        .into_iter()
        .enumerate()
        {
            bush(f, x, 0.0, z, 0.08, m, k as i32 * 3);
        }
        soft!(
            f,
            "sack",
            0.16,
            0.2,
            0.1,
            (0.08, 0.1, WALL + 0.06),
            M.sack,
            round = 0.03,
            puff = 0.012,
            flat = 2
        );
        torus!(
            "sack_roll",
            0.075,
            0.016,
            at = f.at(0.08, 0.205, WALL + 0.06),
            material = M.sack,
            rot = (0, 0, 0),
            major_segments = 16,
            minor_segments = 5
        )
        .set_scale((1.05, 0.62, 1.0));
        fsphere!(
            f,
            "sack_seed",
            0.06,
            (0.08, 0.2, WALL + 0.06),
            M.seed,
            scale = (1.0, 0.55, 0.25),
            segments = 10,
            rings = 5
        );
        turned(
            f,
            "scoop_tin",
            -0.02,
            0.0,
            0.38,
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.06, 0.1),
                (0.064, 0.102),
                (0.06, 0.106),
                (0.056, 0.1),
                (0.046, 0.006),
                (0.0, 0.006),
            ],
            M.tin,
            16,
            none,
        );
        fsphere!(
            f,
            "tin_seed",
            0.05,
            (-0.02, 0.09, 0.38),
            M.seed,
            scale = (1.0, 1.0, 0.25),
            segments = 10,
            rings = 5
        );
        tube(
            f,
            "scoop",
            &[(-0.02, 0.085, 0.38), (0.0, 0.11, 0.37), (0.02, 0.15, 0.36)],
            0.006,
            M.tin,
            Some(6),
            1,
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The roof, its ridge left to right, hinged on its back eave: `lid_open` lifts it for a refill. Shingled slopes,
    // a ridge cap.
    let (eave_y, eave_z) = (HOUSE_Y + 0.1, pz - 0.115);
    let roof = new_parts(|| {
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                name("roof_slab"),
                0.28,
                0.02,
                0.15,
                (px, HOUSE_Y + 0.15, pz + s * 0.06),
                M.roof,
                rot = (s * 36.0, 0.0, 0.0),
                bevel = 0.004,
                bevel_segments = 1
            );
            for k in 0..3 {
                // rows of shingles, each a strip with its lower edge proud
                let t = 0.02 + 0.045 * k as f64;
                let yy = HOUSE_Y + 0.2 - (0.13 - t) * sind(36.0) - 0.006;
                let zz = pz + s * ((0.13 - t) * cosd(36.0) - 0.01);
                fbox!(
                    f,
                    name("shingle"),
                    0.284,
                    0.012,
                    0.05,
                    (px, yy + 0.012, zz),
                    M.roof,
                    rot = (s * 36.0, 0.0, 0.0),
                    bevel = 0.003,
                    bevel_segments = 1
                );
            }
        }
        fcyl!(
            f,
            "ridge",
            0.016,
            0.29,
            (px, HOUSE_Y + 0.2, pz),
            M.roof,
            rot = (0, 90, 0),
            verts = 10
        );
    });
    let roof = hinged(join(&roof, "roof", None), f.at(px, eave_y, eave_z));
    // The songbirds that come while residents watch: built where they perch, joined with their origin in the
    // feeder house and shrunk into it; `bird` grows them out to the tray and the bath.
    let birds = new_parts(|| {
        songbird(
            f,
            "tit",
            -0.34,
            1.383,
            -0.17,
            M.bird_blue,
            M.bird_cream,
            M.mustard,
            Some(M.dark),
            -60.0,
            0.85,
            20.0,
            false,
        );
        songbird(
            f,
            "finch",
            0.36,
            0.694,
            0.2,
            M.bird_gold,
            M.bird_yellow,
            M.mustard,
            Some(M.dark),
            70.0,
            0.9,
            0.0,
            false,
        );
        songbird(
            f,
            "sparrow",
            0.02,
            0.694,
            0.1,
            M.bird_grey,
            M.bird_pink,
            M.mustard,
            Some(M.dark),
            -80.0,
            0.9,
            35.0,
            false,
        );
    });
    let birds = join(&birds, "birds", Some(f.at(px, HOUSE_Y, pz)));
    birds.set_scale((0.001, 0.001, 0.001));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    swing!(roof, "lid_open", -70.0, seconds = 0.7, axis = Axis::X);
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow(birds, "bird", Grow { seconds: 1.0, axis });
    }
    objs
}
