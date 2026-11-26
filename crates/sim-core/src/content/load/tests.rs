use super::*;

const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.06,"icon":"x"},
                 {"id":"fun","label":"Fun","decayPerHour":0.06},
                 {"id":"social","label":"Social","decayPerHour":0.06}],
        "objects":[{"id":"fridge","name":"Fridge","model":"model.fridge",
          "interactions":[{"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.3},"tags":["food"]}]},
          {"id":"tv","name":"TV","interactions":[{"id":"watch","label":"Watch","minutes":30,"effects":{"fun":0.5},"tags":["screen","fun"]}]}],
        "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
        "emotions":[{"id":"happy","label":"Happy","effects":{"tagPreference":{"friendly":1.5}}}],
        "feelings":[{"id":"goodChat","label":"Good chat","emotion":"happy","mood":0.05,"hours":2}],
        "socials":[{"id":"chat","label":"Chat","minutes":15,"tags":["social","friendly"],
          "needs":{"social":0.3},"success":{"friendship":3,"targetFriendship":3,"feeling":"goodChat"}}],
        "bondPresets":{"roommates":{"friendship":25}},
        "socialRules":{"defaultBond":"roommates","busyTags":["food"]},
        "traits":[
          {"id":"foodie","label":"Foodie","effects":{"tagPreference":{"food":2},"needDecay":{"hunger":1.2}}},
          {"id":"neat","label":"Neat","conflicts":["slob"]},
          {"id":"slob","label":"Slob","conflicts":["neat"]},
          {"id":"lazy","label":"Lazy","effects":{"walkSpeed":0.8,"tagAcceptance":{"friendly":0.1}}}],
        "perks":[{"id":"iron","label":"Iron stomach","cost":3,"effects":{"needDecay":{"hunger":0.5}}},
                 {"id":"fast","label":"Fast","cost":3,"effects":{"walkSpeed":1.5}}],
        "rules":{"minTraits":1,"maxTraits":2,"perkPoints":5}}"#;

#[test]
fn parses_and_ignores_visual_fields() {
    let c = Content::from_json(CONTENT).unwrap();
    assert_eq!(c.objects[0].interactions[0].total_gain[0], 0.3);
    assert!((c.objects[0].interactions[0].gain_per_minute[0] - 0.03).abs() < 1e-6);
    assert_eq!(c.object_index("fridge"), Some(0));
    assert_eq!(
        c.tags,
        [
            "food", "screen", "fun", "social", "friendly", "visit", "chores", "cleaning"
        ]
    );
    assert_eq!(c.objects[1].interactions[0].tags, 0b110);
    assert_eq!(c.socials[0].success.feeling, Some(0));
    assert_eq!(c.feelings[0].emotion, Some(0));
    assert_eq!(c.social_rules.busy_tags, 1);
    assert_eq!(c.social_rules.default_bond.unwrap().friendship, 25.0);
}

#[test]
fn track_pay_rises_to_the_top_and_probation_stops_at_one_level() {
    let c = Content::from_json(
        r#"{
            "needs":[{"id":"fun","label":"Fun","decayPerHour":0.06}],
            "objects":[],
            "skills":[{"id":"writing","label":"Writing"}],
            "grades":[{"id":"A","label":"Entry","payPerHour":10,"skillLevel":0},
                      {"id":"B","label":"Junior","payPerHour":20,"skillLevel":2},
                      {"id":"C","label":"Senior","payPerHour":30,"skillLevel":4}],
            "careerCategories":[{"id":"words","label":"Words","payFactor":1.0,
              "shift":{"start":9,"hours":8,"days":[0]},
              "tracks":[
                {"id":"words.low","label":"Low","skills":{"writing":1},"payFactor":0.5,
                 "titles":["Low A","Low B","Low C"]},
                {"id":"words.high","label":"High","skills":{"writing":1},"payFactor":1.5,
                 "titles":["High A","High B","High C"]},
                {"id":"words.one","label":"One","skills":{"writing":1},"payFactor":2.0,
                 "titles":["One A"]}]}],
            "careerRules":{"probationLevels":1}
        }"#,
    )
    .unwrap();
    let pay = |id: &str| -> Vec<i64> {
        let career = &c.careers[c.career_index(id).unwrap()];
        career.levels.iter().map(|l| l.pay).collect()
    };
    // Entry pay is the field's rate whatever the track, and pay climbs in a straight line to
    // the track's own factor at the top grade (8h × the grade's rate, `words` paying 1).
    assert_eq!(pay("words.low"), [80, 120, 120]);
    assert_eq!(pay("words.high"), [80, 200, 360]);
    // A track with one grade has no top to reach: it pays the field's rate.
    assert_eq!(pay("words.one"), [80]);
    assert_eq!(c.career_rules.probation_levels, 1.0);
}

#[test]
fn content_with_moodlet_keys_still_loads() {
    // Every key as written before the moodlet → feeling rename (serde aliases).
    let old = r#"{
            "needs":[{"id":"fun","label":"Fun","decayPerHour":0.06}],
            "skills":[{"id":"cooking","label":"Cooking"}],
            "objects":[{"id":"tv","name":"TV","interactions":[{"id":"watch","label":"Watch",
              "minutes":30,"effects":{"fun":0.5},"skill":"cooking","moodlet":"b","moodletMinSkill":2}]}],
            "moodlets":[{"id":"a","label":"A","mood":0.1,"hours":2},{"id":"b","label":"B","hours":1}],
            "socials":[{"id":"chat","label":"Chat","minutes":15,
              "success":{"moodlet":"a","targetMoodlet":"b","winnerMoodlet":"a","loserMoodlet":"b"}}],
            "socialRules":{"jealousyMoodlet":"a","heartbreakMoodlet":"b"},
            "careerRules":{"promotionMoodlet":"a","missedMoodlet":"b"},
            "economy":{"rent":{"base":100,"debtMoodlet":"b"}}}"#;
    let c = Content::from_json(old).unwrap();
    assert_eq!(c.feelings.len(), 2);
    let watch = &c.objects[0].interactions[0];
    assert_eq!((watch.feeling, watch.feeling_min_skill), (Some(1), 2.0));
    let s = &c.socials[0].success;
    assert_eq!(
        (
            s.feeling,
            s.target_feeling,
            s.winner_feeling,
            s.loser_feeling
        ),
        (Some(0), Some(1), Some(0), Some(1))
    );
    assert_eq!(c.social_rules.jealousy_feeling, Some(0));
    assert_eq!(c.social_rules.heartbreak_feeling, Some(1));
    assert_eq!(c.career_rules.promotion_feeling, Some(0));
    assert_eq!(c.career_rules.missed_feeling, Some(1));
    assert_eq!(c.rent.as_ref().unwrap().debt_feeling, Some(1));
}

#[test]
fn rejects_unknown_references() {
    let bad_need = CONTENT.replace(r#""effects":{"hunger":0.3}"#, r#""effects":{"sleep":0.3}"#);
    assert!(
        Content::from_json(&bad_need)
            .unwrap_err()
            .message
            .contains("unknown need")
    );
    let bad_tag = CONTENT.replace(
        r#""tagPreference":{"food":2}"#,
        r#""tagPreference":{"cooking":2}"#,
    );
    assert!(
        Content::from_json(&bad_tag)
            .unwrap_err()
            .message
            .contains("unknown tag")
    );
    let bad_feeling = CONTENT.replace(r#""feeling":"goodChat""#, r#""feeling":"nope""#);
    assert!(
        Content::from_json(&bad_feeling)
            .unwrap_err()
            .message
            .contains("unknown feeling")
    );
}

#[test]
fn combines_trait_and_perk_modifiers() {
    let c = Content::from_json(CONTENT).unwrap();
    let m = c
        .character_modifiers(&["foodie".into(), "lazy".into()], &["iron".into()])
        .unwrap();
    assert!((m.need_decay[0] - 0.6).abs() < 1e-6);
    assert!((m.walk_speed - 0.8).abs() < 1e-6);
    assert_eq!(m.preference(c.objects[0].interactions[0].tags), 2.0);
    assert_eq!(m.preference(c.objects[1].interactions[0].tags), 1.0);
    assert!((m.acceptance(c.socials[0].tags) - 0.1).abs() < 1e-6);
}

#[test]
fn enforces_character_rules() {
    let c = Content::from_json(CONTENT).unwrap();
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert!(c.check_trait_count(&s(&[])).is_err(), "min traits");
    assert!(
        c.character_modifiers(&s(&["neat", "slob"]), &s(&[]))
            .is_err(),
        "conflict"
    );
    assert!(
        c.check_trait_count(&s(&["neat", "lazy", "foodie"]))
            .is_err(),
        "max traits"
    );
    assert!(
        c.character_modifiers(&s(&["neat"]), &s(&["iron", "fast"]))
            .is_err(),
        "budget"
    );
    assert!(c.character_modifiers(&s(&["neat"]), &s(&["iron"])).is_ok());
    assert!(c.check_trait_count(&s(&["neat"])).is_ok());
}

#[test]
fn validates_gender_and_attraction() {
    let c = Content::from_json(CONTENT).unwrap();
    assert!(c.validate_identity("female", &["male".into()]).is_ok());
    assert!(c.validate_identity("robot", &[]).is_err());
    assert!(c.validate_identity("male", &["robot".into()]).is_err());
}

/// Once-a-day times are kept within the day, like moving's and newcomers': rent, the job
/// market and the planner's review at hour 24 or later would never come round.
#[test]
fn once_a_day_hours_stay_within_the_day() {
    use serde_json::json;
    let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
    c["economy"] = json!({"rent": {"hour": 24, "base": 100}});
    c["careerRules"] = json!({"market": {"hour": 30}});
    c["planner"] = json!({"reviewHour": -1});
    let c = Content::from_json(&c.to_string()).unwrap();
    let hours = [
        c.rent.as_ref().unwrap().hour,
        c.career_rules.market.as_ref().unwrap().hour,
        c.planner.review_hour,
    ];
    assert_eq!(hours, [23.9, 23.9, 0.0]);
    for h in hours {
        let t = crate::clock::tick_at(3, h * 60.0).unwrap();
        assert!(
            crate::clock::at_minute(t, h * 60.0) && crate::clock::day(t) == 3,
            "hour {h} comes on day 3"
        );
    }
}

/// Content that would break the game is refused, saying what's wrong: walking speeds,
/// decays and multipliers, too many genders, recipe steps, resale and upgrade prices, a basic
/// pension below 0, a footprint, repeated ids, bond presets, names nothing has, and times too
/// big to be numbers.
#[test]
fn refuses_content_that_would_break_the_game() {
    use serde_json::json;
    let refused = |edit: &dyn Fn(&mut serde_json::Value), says: &str| {
        let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
        edit(&mut c);
        match Content::from_json(&c.to_string()) {
            Ok(_) => panic!("accepted, expected: {says}"),
            Err(e) => assert!(e.to_string().contains(says), "{e} (expected: {says})"),
        }
    };
    // (1e39 is past what an f32 holds: it reads as infinity.)
    let inf = || json!(1e39);
    refused(
        &|c| c["traits"][3]["effects"]["walkSpeed"] = json!(0),
        "lazy: walkSpeed 0 must be above 0",
    );
    refused(
        &|c| c["traits"][3]["effects"]["walkSpeed"] = json!(-1.0),
        "walkSpeed -1 must be above 0",
    );
    refused(
        &|c| c["perks"][1]["effects"]["walkSpeed"] = json!(11),
        "at most 10",
    );
    refused(
        &|c| c["perks"][1]["effects"]["walkSpeed"] = inf(),
        "fast: walkSpeed inf",
    );
    refused(
        &|c| c["emotions"][0]["effects"]["walkSpeed"] = json!(0),
        "happy: walkSpeed",
    );
    refused(
        &|c| c["feelings"][0]["effects"] = json!({"walkSpeed": 0}),
        "goodChat: walkSpeed",
    );
    refused(
        &|c| c["needs"][0]["decayPerHour"] = json!(-0.1),
        "need 'hunger': decayPerHour -0.1",
    );
    refused(
        &|c| c["traits"][0]["effects"]["needDecay"]["hunger"] = json!(-1),
        "foodie: needDecay -1",
    );
    refused(
        &|c| c["perks"][0]["effects"]["needGain"] = json!({"hunger": -1}),
        "iron: needGain -1",
    );
    refused(
        &|c| c["emotions"][0]["effects"]["tagPreference"]["friendly"] = json!(-1),
        "happy: tagPreference -1",
    );
    refused(
        &|c| c["dayRhythm"] = json!({"nightDecay": {"hunger": -0.5}}),
        "dayRhythm: the hunger multiplier -0.5",
    );
    refused(
        &|c| {
            c["genders"] = (0..33)
                .map(|i| json!({"id": format!("g{i}"), "label": "G"}))
                .collect()
        },
        "at most 32 genders",
    );
    refused(
        &|c| c["objectRules"] = json!({"resale": 1.5}),
        "objectRules.resale 1.5 must be 0..1",
    );
    refused(
        &|c| c["objectRules"] = json!({"resale": -0.5}),
        "objectRules.resale",
    );
    refused(
        &|c| c["objectRules"] = json!({"upgradeCost": -0.3}),
        "objectRules.upgradeCost -0.3",
    );
    refused(
        &|c| c["life"] = json!({"basicPension": -5}),
        "life.basicPension -5 must be ≥ 0",
    );
    refused(
        &|c| c["objects"][1]["footprint"] = json!([7, 1]),
        "object 'tv': a footprint is at most 6 × 6 tiles",
    );
    refused(
        &|c| {
            let it = c["objects"][0]["interactions"][0].clone();
            c["objects"][0]["interactions"]
                .as_array_mut()
                .unwrap()
                .push(it);
        },
        "object 'fridge': interaction 1 repeats the id 'snack'",
    );
    refused(
        &|c| {
            c["needs"]
                .as_array_mut()
                .unwrap()
                .push(json!({"id": "fun", "label": "Fun", "decayPerHour": 0.1}))
        },
        "duplicate need id",
    );
    refused(
        &|c| {
            c["emotions"]
                .as_array_mut()
                .unwrap()
                .push(json!({"id": "happy", "label": "Happy"}))
        },
        "duplicate emotion id",
    );
    refused(
        &|c| {
            c["feelings"]
                .as_array_mut()
                .unwrap()
                .push(json!({"id": "goodChat", "label": "Good", "hours": 1}))
        },
        "duplicate feeling id",
    );
    refused(
        &|c| c["bondPresets"]["roommates"]["friendship"] = json!(150),
        "bond preset 'roommates': friendship and romance are -100..100",
    );
    refused(
        &|c| c["bondPresets"]["roommates"]["romance"] = json!(-101),
        "bond preset 'roommates'",
    );
    refused(
        &|c| c["socialRules"]["busyTags"] = json!(["food", "nonexistent"]),
        "socialRules.busyTags: unknown tag 'nonexistent'",
    );
    refused(
        &|c| c["socialRules"]["guestTags"] = json!(["bathrom"]),
        "socialRules.guestTags: unknown tag 'bathrom'",
    );
    refused(
        &|c| c["dayRhythm"] = json!({"sleepTags": ["slep"]}),
        "dayRhythm.sleepTags: unknown tag 'slep'",
    );
    refused(
        &|c| c["dayRhythm"] = json!({"napTags": ["nap"]}),
        "dayRhythm.napTags: unknown tag 'nap'",
    );
    refused(
        &|c| {
            c["dayRhythm"] =
                json!({"nightQuiet": {"noisy": [{"tag": "chores", "unless": ["kare"]}]}})
        },
        "dayRhythm.nightQuiet: unknown tag 'kare'",
    );
    refused(
        &|c| c["roomKinds"] = json!([{"id": "den", "tags": ["screen"], "essentials": [["scren"]]}]),
        "room kind 'den': unknown tag 'scren'",
    );
    refused(
        &|c| c["roomRules"] = json!({"dirt": {"fod": 0.1}}),
        "roomRules.dirt: unknown tag 'fod'",
    );
    refused(
        &|c| c["objectRules"] = json!({"repair": {"skill": "handiness"}}),
        "objectRules.repair: unknown skill 'handiness'",
    );
    refused(
        &|c| c["objectRules"] = json!({"repair": {"tags": ["chore"]}}),
        "objectRules.repair: unknown tag 'chore'",
    );
    refused(
        &|c| c["accidents"] = json!([{"id": "doze", "need": "fun", "rest": {"label": "Doze", "minutes": 10, "anim": "snooze"}}]),
        "accident 'doze': unknown anim 'snooze'",
    );
    refused(
        &|c| c["accidents"] = json!([{"id": "doze", "need": "fun", "rest": {"label": "Doze", "minutes": 10, "tags": ["sleep"]}}]),
        "accident 'doze': unknown tag 'sleep'",
    );
    refused(
        &|c| c["activities"] = json!([{"id": "tv", "label": "TV", "tags": ["scren"]}]),
        "activity 'tv': unknown tag 'scren'",
    );
    refused(
        &|c| c["cooking"] = json!({"eatTags": ["fod"]}),
        "cooking.eatTags: unknown tag 'fod'",
    );
    refused(
        &|c| {
            c["items"] = json!([{"id": "pot", "label": "Pot"}]);
            let step = json!({"at": ["counter"], "minutes": 1});
            c["recipes"] =
                json!([{"id": "stew", "label": "Stew", "item": "pot", "steps": vec![step; 256]}]);
        },
        "recipe 'stew': 1 to 255 steps",
    );
    refused(
        &|c| c["feelings"][0]["hours"] = inf(),
        "goodChat: hours must be > 0",
    );
    refused(
        &|c| c["objects"][0]["interactions"][0]["minutes"] = inf(),
        "fridge.snack: minutes must be > 0",
    );
    refused(
        &|c| c["socials"][0]["minutes"] = inf(),
        "chat: minutes must be > 0",
    );
    refused(
        &|c| c["careerRules"] = json!({"commuteMinutes": 1e39}),
        "careerRules.commuteMinutes: times must be numbers",
    );
    // Careers: pay and probation stay within what the game can pay out.
    let career = |c: &mut serde_json::Value| {
        c["skills"] = json!([{"id": "writing", "label": "Writing"}]);
        c["grades"] = json!([{"id": "A", "label": "Entry", "payPerHour": 10, "skillLevel": 0}]);
        c["careerRules"] = json!({});
        c["carBodies"] = json!([{"id": "van", "label": "van", "len": 5.0, "wid": 1.98},
                                    {"id": "saloon", "label": "saloon", "len": 4.4, "wid": 1.78}]);
        c["careerCategories"] = json!([{
                "id": "words", "label": "Words", "payFactor": 1.0,
                "shift": {"start": 9, "hours": 8, "days": [0]}, "car": {"A": "saloon"},
                "tracks": [{"id": "words.low", "label": "Low", "skills": {"writing": 1},
                            "payFactor": 1.0, "titles": ["Low A"], "car": {"A": "van"}}]}]);
    };
    // Each row: where the one field is set, the value put there, and what it is refused for.
    for (path, value, says) in [
        (
            "/grades/0/payPerHour",
            json!(-1),
            "grade 'A': payPerHour -1 must be 0..10000",
        ),
        (
            "/grades/0/payPerHour",
            json!(1e39),
            "grade 'A': payPerHour inf must be 0..10000",
        ),
        (
            "/careerCategories/0/payFactor",
            json!(0.1),
            "career category 'words': payFactor 0.1 must be 0.25..3",
        ),
        (
            "/careerCategories/0/tracks/0/payFactor",
            json!(4),
            "track 'words.low': payFactor 4 must be 0.25..3",
        ),
        (
            "/careerRules",
            json!({"probationLevels": 2}),
            "careerRules.probationLevels 2 must be 0..1",
        ),
        // Cars: a body parks and drives by its size, and a field's car names a body and a grade.
        (
            "/carBodies/0/len",
            json!(9),
            "car body 'van': len 9 must be 2.5..5.4",
        ),
        (
            "/carBodies/0/wid",
            json!(0.5),
            "car body 'van': wid 0.5 must be 1.4..2.2",
        ),
        ("/carBodies/1/id", json!("van"), "duplicate car body 'van'"),
        (
            "/careerCategories/0/car",
            json!({"Z": "van"}),
            "'words': car for unknown grade 'Z'",
        ),
        (
            "/careerCategories/0/tracks/0/car",
            json!({"A": "bus"}),
            "'words.low': unknown car body 'bus'",
        ),
    ] {
        refused(
            &|c| {
                career(c);
                *c.pointer_mut(path).unwrap() = value.clone()
            },
            says,
        );
    }
    refused(
        &|c| c["accidents"] = json!([{"id": "doze", "need": "fun", "cooldownHours": 1e39}]),
        "accident 'doze': times must be numbers",
    );
    refused(
        &|c| c["dayRhythm"] = json!({"babyWakes": {"cooldownMinutes": 1e39}}),
        "dayRhythm.babyWakes.cooldownMinutes",
    );
    refused(
        &|c| c["cooking"] = json!({"inviteMinutes": 1e39}),
        "cooking.inviteMinutes must be a number",
    );
    refused(
        &|c| c["objects"][0]["interactions"][0]["hours"] = json!([20, 20]),
        "fridge.snack: `hours` is [from, to]",
    );
    refused(
        &|c| c["objects"][0]["interactions"][0]["hours"] = json!([20, 25]),
        "fridge.snack: `hours`",
    );
    refused(
        &|c| c["objects"][0]["interactions"][0]["workOn"] = json!("laundry"),
        "fridge.snack: `workOn` laundry needs content `laundry`",
    );
    refused(
        &|c| c["laundry"] = json!({"perDay": 0, "most": 4}),
        "laundry: `perDay` above 0",
    );
    // Ironing makes pressed clothes, which content `laundry.pressed` says what they do.
    refused(
        &|c| {
            c["laundry"] = json!({"perDay": 0.4, "most": 4});
            c["objects"][0]["interactions"][0]["makes"] = json!("ironed");
        },
        "fridge.snack: `makes` ironed needs content `laundry.pressed`",
    );
    refused(
        &|c| {
            c["laundry"] =
                json!({"perDay": 0.4, "most": 4, "pressed": {"feeling": "nope", "outfits": 4}})
        },
        "laundry.pressed: unknown feeling 'nope'",
    );
    // Cleaning the floor: standing at it, the tool in the hands.
    refused(
        &|c| {
            c["objects"][0]["interactions"][0]["cleans"] = json!({"amount": 0.5});
            c["objects"][0]["interactions"][0]["pose"] = json!("sit");
        },
        "fridge.snack: a use that `cleans`",
    );
    refused(
        &|c| c["objects"][0]["interactions"][0]["cleans"] = json!({"amount": 1.5}),
        "fridge.snack: a use that `cleans`",
    );
    // Sold by someone: at an object someone serves at, standing near it (`staff.rs`).
    refused(
        &|c| c["objects"][0]["interactions"][0]["selfServe"] = json!({"label": "Take one"}),
        "`selfServe` is for something sold, at an object with `staff`",
    );
    refused(
        &|c| c["objects"][0]["staff"] = json!({"at": [0, -3], "hours": [8, 18]}),
        "object 'fridge': `staff.at` is more than 1.5 m off its footprint",
    );
    refused(
        &|c| c["objects"][0]["staff"] = json!({"at": [0, -0.8], "hours": [8, 8]}),
        "object 'fridge': `staff.hours`",
    );
    refused(
        &|c| c["service"] = json!({"waitMinutes": 0}),
        "service: `share` at least 0 and `waitMinutes` above 0",
    );
    // A game for two: two places, and its use for one another of the object's, alike.
    refused(
        &|c| c["objects"][1]["interactions"][0]["partner"] = json!(true),
        "tv.watch: a game for two (`partner`) is played at an object with two places",
    );
    refused(
        &|c| c["objects"][1]["interactions"][0]["alone"] = json!("watch"),
        "tv.watch: `alone` is what's done with nobody to play: it needs `partner`",
    );
    let game = |alone: &str, pose: &str| {
        let alone = alone.to_owned();
        let pose = pose.to_owned();
        move |c: &mut serde_json::Value| {
            c["objects"][1]["slots"] = json!(2);
            c["objects"][1]["interactions"][0]["partner"] = json!(true);
            c["objects"][1]["interactions"][0]["alone"] = json!(alone);
            c["objects"][1]["interactions"].as_array_mut().unwrap().push(json!({"id":"solo","label":"Solo","minutes":10,"pose":pose,"effects":{"fun":0.1}}));
        }
    };
    refused(
        &game("nothing", "stand"),
        "tv.watch: `alone` names no use of 'tv': 'nothing'",
    );
    refused(
        &game("watch", "stand"),
        "tv.watch: `alone` is a use for one (no `partner`), done in the same pose as the game",
    );
    refused(&game("solo", "sit"), "tv.watch: `alone` is a use for one");
    let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
    game("solo", "stand")(&mut c);
    let content = Content::from_json(&c.to_string()).unwrap();
    let tv = content.objects.iter().find(|o| o.id == "tv").unwrap();
    assert!(
        tv.interactions[0].partner
            && tv.interactions[0].alone == Some(1)
            && !tv.interactions[1].partner
    );
    refused(
        &|c| {
            c["objects"][1]["layer"] = json!("surface");
            c["objects"][1]["footprint"] = json!([1, 1]);
            c["objects"][1]["interactions"][0]["pose"] = json!("lie");
        },
        "a thing on a surface is used standing or sitting at its top",
    );
    // (Each of these on its own is fine.) A careers block left to its defaults, one grade and
    // all, is accepted with them.
    let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
    career(&mut c);
    c["objectRules"] = json!({"resale": 1.0, "upgradeCost": 0.0});
    c["bondPresets"]["roommates"]["friendship"] = json!(-100);
    c["perks"][1]["effects"]["walkSpeed"] = json!(10);
    assert!(Content::from_json(&c.to_string()).is_ok());
}
