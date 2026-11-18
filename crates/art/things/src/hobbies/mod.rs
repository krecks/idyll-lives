//! The hobbies: music (H1: `model.piano`, `model.ukulele`, `model.guitarElectric`, `model.keyboardDigital`,
//! `model.drumKit`, `model.guitarAcoustic`), games (H2: `model.chessTable`, `model.poolTable`, `model.foosball`,
//! `model.dartboard`, `model.arcadeCabinet`, `model.puzzleTable`) and crafts and science (H3: `model.easel`,
//! `model.workbench`, `model.telescope`, `model.sewingTable`, `model.chemistrySet`, `model.potteryWheel`,
//! `model.craftTable`). Notes: NOTES.md.
//!
//! Each is built where the game places it, in the game's axes (`F`), and stood on the floor (`settle`, not
//! centred), so a seat stays where the game seats residents. The moving parts and their clips:
//!   piano          the fallboard lifts off the keys and the music desk swings out (`fallboard`)
//!   drumKit        the crash and ride cymbals wobble while drummed (`cymbals`, a loop)
//!   chessTable     a black pawn slides one square towards white (`move`)
//!   poolTable      the triangle lifts off the racked balls (`rack_off`)
//!   foosball       the two outer rods kick back and forth (`rods`, a loop)
//!   dartboard      three darts grow in the board (`darts`)
//!   arcadeCabinet  the joystick waggles (`joystick`, a loop)
//!   puzzleTable    a patch of finished puzzle spreads (`pieces`)
//!   easel          the painting spreads across the canvas (`painting`)
//!   workbench      the vice's jaw slides shut on a plank (`vice`)
//!   telescope      the tube tilts up 35° (`aim`)
//!   sewingTable    the presser foot lowers (`presser`); the handwheel turns (`handwheel`, a loop)
//!   chemistrySet   the liquid rises in the flask (`bubble`)
//!   potteryWheel   the pot rises from a lump (`clay`); the wheel head turns (`wheel`, a loop)
//!   craftTable     the scrapbook's cover swings open (`book_open`)
//!   ukulele, guitarElectric, guitarAcoustic
//!                  the instrument is hidden while played (`off_stand`): the game draws it in the player's hands
//!                  (content `held`), from this part, upright as built (its lean on the stand is the node's turn)
//! The digital piano is still.

use artkit::family::{Family, Model};
use artkit::json;
use artkit::json::Json;
use artkit::palette;

mod acoustic;
mod arcade;
mod chemistry;
mod chess;
mod common;
mod craft;
mod darts;
mod drum_kit;
mod drums;
mod easel;
mod electric;
mod foosball;
mod guitars;
mod keyboard;
mod piano;
mod pool;
mod pottery;
mod puzzle;
mod sewing;
mod telescope;
mod workbench;
mod workbench_board;

palette! { M:
    // the piano and the digital piano
    piano = ("#6E4630", gloss), // the lacquered case; the main part the player colours
    piano_trim = ("#9C6B44", gloss), // lid, key slip, back panels, music desk
    ivory = ("#F6F3EA", satin), // the white keys
    key_gap = ("#B9B2A2", satin), // the lines between them
    ebony = ("#141414", gloss), // the black keys
    brass = ("#B8925A", metal), // pedals, candle sconces
    candle = ("#F3EAD8", satin),
    sheet = ("#F4EFE2", matte), // sheet music, paper
    ink = ("#3A3A3A", matte), // the staves
    cushion = ("#7A2E2E", satin), // the piano bench's top
    kb_body = ("#2A2C30", satin), // the digital piano's case
    kb_trim = ("#4A4D52", satin), // its control strip and speakers
    kb_seat = ("#3A3D41", fabric), // its bench
    lcd = ("#7FB8A8", gloss),
    // the guitars and the ukulele
    stand = ("#2B2C2E", metal), // the instrument stands
    rubber = ("#1A1B1D", satin), // their pads and feet
    uke = ("#C08A52", wood), // the ukulele's body and neck
    uke_top = ("#D9A86C", wood), // its soundboard
    fretboard = ("#3B2A20", satin),
    hole = ("#1E1712", matte), // sound holes
    fret = ("#C9CBCC", metal), // frets, tuners
    string = ("#D8D8D2", metal),
    bone = ("#EFE8D8", satin), // nuts and saddles
    electric = ("#D9483B", gloss), // the electric guitar's body
    pickguard = ("#F2F2F0", gloss),
    maple = ("#D9B98A", satin), // its neck
    chrome = ("#D0D3D6", chrome), // hardware: bridges, drum stands, rods
    black = ("#2A2C30", gloss), // headstock, pickups, knobs
    amp = ("#2A2C30", matte), // the amp's cabinet
    grille = ("#4A4D52", fabric), // its speaker cloth
    amp_trim = ("#C9A27A", satin), // its control panel and piping
    acoustic = ("#C08A52", gloss), // the acoustic guitar's back and sides
    spruce = ("#E2C48E", gloss), // its top
    neck = ("#3B2A20", wood), // its neck and head
    tortoise = ("#5A3420", gloss), // its pickguard
    // the drums
    shell = ("#B03A2E", gloss), // the shells and hoops
    head = ("#F2F0EA", satin), // the drum heads
    cymbal = ("#C9A64A", metal),
    seat = ("#26272A", satin), // the throne
    // the games
    table_wood = ("#7A5234", wood), // the chess and jigsaw tables
    chair_wood = ("#A8754A", wood), // their chair and stool
    seat_pad = ("#B5653F", satin), // the chess chair's pad
    square_light = ("#E9D7B4", satin),
    square_dark = ("#4A2E1C", satin),
    piece_light = ("#F2EDE2", gloss),
    piece_dark = ("#1E1C1B", gloss),
    pool_frame = ("#6B4630", gloss), // the pool table's lacquered frame
    cloth = ("#2F7A57", matte), // its baize (plain, not `fabric`: the balls' two parts take its draw call)
    pocket = ("#141414", matte),
    pearl = ("#EDE6D6", gloss), // the rail sights
    rack = ("#A8754A", gloss), // the triangle
    cue = ("#D9B98A", satin),
    cue_butt = ("#3B2A20", satin),
    ball_yellow = ("#F2C14E", gloss),
    ball_blue = ("#3E8EDE", gloss),
    ball_red = ("#D9483B", gloss),
    ball_purple = ("#6E3FA8", gloss),
    ball_orange = ("#E9853B", gloss),
    ball_green = ("#2F8F4E", gloss),
    ball_maroon = ("#8E2A2A", gloss),
    ball_black = ("#141414", gloss),
    ball_white = ("#F4F2EC", gloss),
    foos = ("#CDB08A", satin), // the foosball cabinet
    foos_trim = ("#8E6B45", satin), // its legs and corners
    field = ("#2F8F4E", matte),
    line = ("#F2F2F0", matte),
    grip = ("#2A2C30", satin), // handles, goal mouths
    cabinet = ("#6E4A30", satin), // the dartboard's cabinet
    board = ("#1E1F22", matte), // the board's dark segments and number ring
    board_light = ("#EADBB8", matte),
    ring_red = ("#C23B2E", matte),
    ring_green = ("#2F7A4A", matte),
    steel = ("#C9CBCC", metal), // dart barrels
    arcade = ("#1E1F22", satin), // the arcade cabinet
    art = ("#2D3E8C", gloss), // its side art
    art_stripe = ("#F2C14E", gloss),
    bezel = ("#16181B", gloss),
    screen = ("#3E8EDE", gloss, glow("#7FC4FF")),
    marquee = ("#F2C14E", gloss, glow("#FFE08A")),
    puzzle_board = ("#5E7A9A", matte),
    sky = ("#9CC3DB", matte), // the jigsaw's and the painting's sky
    hill = ("#7FA35B", matte),
    sun = ("#F2C14E", matte),
    blush = ("#E4B8AE", matte),
    card = ("#C9A979", matte), // the jigsaw's box
    // crafts and science
    easel_wood = ("#B98B5E", wood),
    easel_dark = ("#9A6E47", wood),
    easel_metal = ("#8E9499", metal), // its clamp and wing nuts
    canvas = ("#F3EFE6", matte),
    tree = ("#5E8445", matte),
    tree_light = ("#6E9450", matte),
    river = ("#5E8FB0", matte),
    palette = ("#D2B48A", satin),
    jar = ("#7E9AA8", satin),
    bench_top = ("#C9A579", wood), // the workbench's butcher-block top
    bench_steel = ("#8A9298", metal), // its tool board and the vice's pads
    bench_frame = ("#A87B50", wood), // its posts, rails and shelves
    pegboard = ("#C9B08A", matte), // its tool board (hardboard)
    label = ("#EFEDE6", matte),
    foot = ("#2F3236", satin), // feet, grips, dark plastic
    toolbox = ("#4E7A5A", satin),
    toolbox_lid = ("#5E8C68", satin),
    drill = ("#F2B84B", satin),
    plank = ("#D9C7A7", wood),
    can = ("#B9BDC1", metal),
    vice = ("#3E6FB0", satin),
    tube = ("#2E4E7E", gloss), // the telescope's tube
    tripod = ("#F2F2F0", gloss),
    optic = ("#1E2A3A", gloss), // lenses
    sew_table = ("#CDB08A", satin),
    sew_legs = ("#B08E66", satin),
    machine = ("#F2F1EE", gloss), // the sewing machine
    machine_trim = ("#9DC6CF", satin),
    fabric_pink = ("#E4B8AE", satin), // the cloth being sewn
    sew_pad = ("#9DAF8C", satin), // the sewing chair's pad
    thread_red = ("#B5653F", satin),
    thread_teal = ("#4E7A7F", satin),
    basket = ("#C9B79C", matte),
    chem_bench = ("#E4E3DF", satin), // the lab bench
    chem_frame = ("#2B2C2E", metal),
    chem_board = ("#F2F1EE", satin),
    glass = ("#DDEFF2", glass), // flasks and test tubes (see-through: its own draw call)
    liquid_green = ("#7FD1AE", gloss),
    liquid_orange = ("#E9853B", gloss),
    liquid_blue = ("#6E8BE6", gloss),
    liquid_yellow = ("#F2C14E", gloss),
    liquid_red = ("#D9483B", gloss),
    scope = ("#2A2C30", satin), // the microscope
    wheel_body = ("#7F8C8D", satin), // the pottery wheel
    pan = ("#9AA3A5", satin), // its splash pan and wheel head
    clay = ("#B5653F", matte),
    clay_slip = ("#9E5434", satin), // wet clay on the wheel head
    stool = ("#8A5A36", satin),
    craft_top = ("#E3CFA8", wood), // the craft table's top
    craft_metal = ("#2B2C2E", metal), // its hairpin legs
    paper = ("#E7D6B8", matte),
    scrapbook = ("#C0473A", satin), // the scrapbook's cover
    pages = ("#F4EFE2", matte),
    photo = ("#9CC3DB", gloss),
    photo_warm = ("#E9B98A", gloss),
    tape_blue = ("#9DC6CF", matte),
    tape_gold = ("#D6A64A", matte),
    cup = ("#4E7A7F", gloss),
}

// The seated hobbies' parts wait while the resident sits down (characters.ts: the `sitDown` clip, about 1 s), so
// the board, the keys, the presser and the wheel move once the hands are there, as the weight bench's bar does
// (fitness `BAR_DELAY`). A part that goes round while used does so from then too: `every` a hair under its
// round (its clip, 1.0 s), so the rounds run on without a pause.
const SIT: f64 = 0.9;

fn after_sitting(clip: &str, every: Option<f64>) -> Json {
    let mut out = json!({"clip": (clip), "delay": (SIT)});
    if let Some(every) = every {
        out.set("every", Json::Num(every));
    }
    out
}

/// The family.
pub fn family() -> Option<Family> {
    let off_stand = || json!({"uses": {"music": "off_stand"}});
    let pool = || {
        Json::Arr(vec![
            Json::from("rack_off"),
            json!({"clip": "racked_off", "delay": (pool::BREAK)}),
            json!({"clip": "spread", "delay": (pool::BREAK)}),
        ])
    };
    let sew = || {
        Json::Arr(vec![
            after_sitting("presser", None),
            after_sitting("handwheel", Some(0.9)),
        ])
    };
    Some(Family::new(
        "hobbies",
        M.palette(),
        vec![
            Model::new("piano", piano::piano, (1.54, 1.48, 0.827))
                .key("model.piano")
                .centre(0.0, -0.0435 + piano::PIANO_FORWARD)
                .motions(json!({"uses": {"play": (after_sitting("fallboard", None))}}))
                .paint(&[("piano", "Body"), ("piano_trim", "Trim")]),
            Model::new("ukulele", guitars::ukulele, (0.348, 0.841, 0.258))
                .key("model.ukulele")
                .centre(0.0, 0.065)
                .motions(off_stand())
                .paint(&[("uke", "Body & neck"), ("uke_top", "Top"), ("stand", "Stand")]),
            Model::new("guitarElectric", electric::guitar_electric, (0.46, 1.001, 0.584))
                .key("model.guitarElectric")
                .centre(0.0, -0.128)
                .motions(off_stand())
                .paint(&[("electric", "Guitar"), ("amp", "Amp")]),
            Model::new("keyboardDigital", keyboard::keyboard_digital, (1.392, 1.047, 0.77))
                .key("model.keyboardDigital")
                .centre(0.0, -0.025 + keyboard::KB_FORWARD)
                .paint(&[("kb_body", "Body"), ("kb_trim", "Trim")]),
            Model::new("drumKit", drum_kit::drum_kit, (1.732, 1.238, 1.165))
                .key("model.drumKit")
                .centre(0.05, -0.0175)
                .motions(json!({"uses": {"drum": (after_sitting("cymbals", Some(0.9))),
                                         "practise": (after_sitting("cymbals", Some(0.9)))}}))
                .paint(&[("shell", "Shells"), ("chrome", "Hardware")]),
            Model::new("guitarAcoustic", acoustic::guitar_acoustic, (0.4, 1.161, 0.405))
                .key("model.guitarAcoustic")
                .centre(0.0, -0.0217)
                .motions(off_stand())
                .paint(&[("acoustic", "Body"), ("spruce", "Top"), ("stand", "Stand")]),
            Model::new("chessTable", chess::chess_table, (1.7, 0.92, 0.48))
                .key("model.chessTable")
                .motions(json!({"uses": {"chess": (after_sitting("move", None)), "play": (after_sitting("move", None))}}))
                .paint(&[("table_wood", "Table"), ("chair_wood", "Chair")]),
            Model::new("poolTable", pool::pool_table, (2.5, 0.807, 1.4))
                .key("model.poolTable")
                .motions(json!({"uses": {"pool": (pool()), "trickShots": (pool())}}))
                .paint(&[("pool_frame", "Frame"), ("cloth", "Cloth")]),
            Model::new("foosball", foosball::foosball, (1.42, 0.947, 0.935))
                .key("model.foosball")
                .centre(0.0, 0.0375)
                .motions(json!({"loops": {"foosball": "rods", "practise": "rods"}}))
                .paint(&[("foos", "Table"), ("foos_trim", "Trim")]),
            Model::new("dartboard", darts::dartboard, (0.62, 0.62, 0.07))
                .key("model.dartboard")
                .centre(0.0, -0.393)
                .on_wall()
                .motions(json!({"uses": {"darts": "darts"}}))
                .paint(&[("board", "Board"), ("cabinet", "Cabinet")]),
            Model::new("arcadeCabinet", arcade::arcade_cabinet, (0.672, 1.82, 0.872))
                .key("model.arcadeCabinet")
                .centre(0.0, 0.006)
                .motions(json!({"loops": {"arcade": "joystick"}}))
                .paint(&[("arcade", "Cabinet"), ("art", "Art")]),
            Model::new("puzzleTable", puzzle::puzzle_table, (0.865, 0.794, 0.891))
                .key("model.puzzleTable")
                .centre(0.0025, 0.0155)
                .motions(json!({"uses": {"jigsaw": (after_sitting("pieces", None))}}))
                .paint(&[("table_wood", "Table"), ("chair_wood", "Stool")]),
            Model::new("easel", easel::easel, (0.704, 1.747, 0.762))
                .key("model.easel")
                .centre(0.0318, -0.0347)
                .motions(json!({"uses": {"paint": "painting"}}))
                .paint(&[("easel_wood", "Wood"), ("easel_metal", "Metal")]),
            Model::new("workbench", workbench::workbench, (1.62, 1.69, 0.665))
                .key("model.workbench")
                .centre(0.0, -0.0975)
                .motions(json!({"uses": {"tinker": "vice"}}))
                .paint(&[("bench_top", "Top"), ("bench_frame", "Frame"), ("pegboard", "Tool board")]),
            Model::new("telescope", telescope::telescope, (0.664, 1.522, 0.912))
                .key("model.telescope")
                .centre(0.0, 0.026)
                .motions(json!({"uses": {"stargaze": "aim"}}))
                .paint(&[("tube", "Tube"), ("tripod", "Tripod")]),
            Model::new("sewingTable", sewing::sewing_table, (0.9, 1.08, 0.89))
                .key("model.sewingTable")
                .centre(0.0, 0.025)
                .motions(json!({"uses": {"sew": (sew()), "mend": (sew())}}))
                .paint(&[("sew_table", "Table"), ("sew_legs", "Legs"), ("machine", "Machine")]),
            Model::new("chemistrySet", chemistry::chemistry_set, (0.92, 1.4, 0.5))
                .key("model.chemistrySet")
                .centre(0.0, -0.18)
                .motions(json!({"uses": {"experiment": "bubble"}}))
                .paint(&[("chem_bench", "Bench"), ("glass", "Glassware")]),
            Model::new("potteryWheel", pottery::pottery_wheel, (0.6, 0.544, 0.88))
                .key("model.potteryWheel")
                .centre(0.06, 0.01)
                .motions(json!({"uses": {"throw": (Json::Arr(vec![after_sitting("clay", None), after_sitting("wheel", Some(0.9))]))}}))
                .paint(&[("wheel_body", "Wheel"), ("stool", "Stool")]),
            Model::new("craftTable", craft::craft_table, (1.7, 0.904, 0.89))
                .key("model.craftTable")
                .centre(0.0, 0.005)
                .motions(json!({"uses": {"scrapbook": (after_sitting("book_open", None))}}))
                .paint(&[("craft_top", "Wood"), ("craft_metal", "Metal")]),
        ],
        "The hobby models (crates/art/things/src/hobbies, built by cargo art): music, games, crafts and \
         science. Listed in the base manifest's packs. The materials map onto the game's finishes so \
         a model costs one draw call per finish group. The piano lifts its fallboard, the drums' \
         cymbals and the foosball rods move while played, the darts, the painting, the jigsaw patch, \
         the pot and the liquid grow, and the other hobbies move a part while a resident uses them.",
    ).switched())
}
