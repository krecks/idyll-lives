//! `cargo fetch-voice`: the resident voice model and the pronunciation dictionary, into
//! `web/public/voice/` (not in git; see docs/design/voices.md). (Was tools/voice/fetch.mjs.)
//!
//! - Supertone's Supertonic 3 (BigScience OpenRAIL-M: its licence, with its use restrictions, is
//!   fetched with it and goes along), its config, text indexer and ten voice styles as published,
//!   and its four ONNX graphs with their weights stored as 16-bit floats (`fp16`: half the size,
//!   the same arithmetic; a `NOTICE` says so) and gzipped (`.onnxz`, unpacked by the browser):
//!   398 MB as published, 185 MB here. https://huggingface.co/Supertone/supertonic-3
//! - Misaki's US English dictionaries (Apache-2.0, its licence goes along as `misaki-LICENSE.txt`),
//!   for mouth shapes: https://github.com/hexgrad/misaki
//!
//! Everything is pinned to a revision and checked against its SHA-256 (the converted graphs too:
//! the conversion is deterministic), so every build speaks the same. Files already present with
//! the right hash (unpacked) are kept. `bun run voice` builds the voice WASM, then runs this.

use crate::download::{self, agent, gzip9, sha256_hex};
use crate::fp16::weights_to_fp16;
use std::path::{Path, PathBuf};

const SUPERTONIC: &str = "https://huggingface.co/Supertone/supertonic-3/resolve/3cadd1ee6394adea1bd021217a0e650ede09a323";

/// The model's folder under `voice/`: a new version gets a new folder, so browsers never mix
/// versions.
pub const MODEL_DIR: &str = "supertonic-3/";

/// Supertonic 3's graphs: as published, and with their weights in 16-bit floats, saved gzipped as
/// `<name>.onnxz`.
const GRAPHS: [(&str, &str, &str); 4] = [
    ("duration_predictor", "c3eb91414d5ff8a7a239b7fe9e34e7e2bf8a8140d8375ffb14718b1c639325db", "622de4d364dcb42259986c32d5ca4a0f521fddfaed09ffb155aa8cf3d7e4d864"),
    ("text_encoder", "c7befd5ea8c3119769e8a6c1486c4edc6a3bc8365c67621c881bbb774b9902ff", "7533e6a775956007f8dc848d008751e7dbbcf8399cefb5ef180737541c3e0e0c"),
    ("vector_estimator", "883ac868ea0275ef0e991524dc64f16b3c0376efd7c320af6b53f5b780d7c61c", "27a86f93335fd6edec7210c24dddb4f0de700688f9f0d18ab6418bd852742402"),
    ("vocoder", "085de76dd8e8d5836d6ca66826601f615939218f90e519f70ee8a36ed2a4c4ba", "373aca3e481c5478af6e66773a58dcd1bdd6e3f874d676dc3b90ffb7e268452a"),
];

/// Supertonic 3's other files, as published (`path` in the repository, saved under its last part).
const FILES: [(&str, &str); 13] = [
    ("onnx/tts.json", "42078d3aef1cd43ab43021f3c54f47d2d75ceb4e75f627f118890128b06a0d09"),
    ("onnx/unicode_indexer.json", "9bf7346e43883a81f8645c81224f786d43c5b57f3641f6e7671a7d6c493cb24f"),
    ("voice_styles/F1.json", "bbdec6ee00231c2c742ad05483df5334cab3b52fda3ba38e6a07059c4563dbc2"),
    ("voice_styles/F2.json", "7c722c6a72707b1a77f035d67f0d1351ba187738e06f7683e8c72b1df3477fc6"),
    ("voice_styles/F3.json", "12f6ef2573baa2defa1128069cb59f203e3ab67c92af77b42df8a0e3a2f7c6ab"),
    ("voice_styles/F4.json", "c2fa764c1225a76dfc3e2c73e8aa4f70d9ee48793860eb34c295fff01c2e032b"),
    ("voice_styles/F5.json", "45966e73316415626cf41a7d1c6f3b4c70dbc1ba2bee5c1978ef0ce33244fc8d"),
    ("voice_styles/M1.json", "e35604687f5d23694b8e91593a93eec0e4eca6c0b02bb8ed69139ab2ea6b0a5b"),
    ("voice_styles/M2.json", "b76cbf62bac707c710cf0ae5aba5e31eea1a6339a9734bfae33ab98499534a50"),
    ("voice_styles/M3.json", "ea1ac35ccb91b0d7ecad533a2fbd0eec10c91513d8951e3b25fbba99954e159b"),
    ("voice_styles/M4.json", "ca8eefad4fcd989c9379032ff3e50738adc547eeb5e221b82593a6d7b3bac303"),
    ("voice_styles/M5.json", "dd22b92740314321f8ae11c5e87f8dd60d060f15dd3a632b5adf77f471f77af2"),
    ("LICENSE", "0d944a9110fed9a9602d60e0423a272903e7bd21ab060490774efc77c2275e9f"),
];

/// Files earlier versions wrote that are no longer used (they would be copied into the build).
const OBSOLETE: [&str; 14] = ["misaki-LICENSE", "pocket-en-2026-04", "presets-2.pcm", "presets-2.json", "presets-2.stamp", "presets-1.pcm", "presets-1.json", "presets-1.stamp", "paradee-8m.onnx", "paradee-8m-edit1.onnx", "paradee-8m.json", "kitten-nano-0.8-edit1.onnx", "kitten-nano-0.8-edit2.onnx", "kitten-nano-0.8-voices.f32"];

/// Misaki's licence (Apache-2.0: recipients get a copy), next to the dictionary made from it.
const LEXICON_LICENSE: (&str, &str, &str) = ("https://raw.githubusercontent.com/hexgrad/misaki/fba1236595f2d2bf21d414ba6e57d25256afada3/LICENSE", "misaki-LICENSE.txt", "c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4");

const LEXICON: [(&str, &str); 2] = [
    ("https://raw.githubusercontent.com/hexgrad/misaki/fba1236595f2d2bf21d414ba6e57d25256afada3/misaki/data/us_gold.json", "dc414872a49a28ae6c141463d502fd945f3b2fde040484fdc47d00cc4612686f"),
    ("https://raw.githubusercontent.com/hexgrad/misaki/fba1236595f2d2bf21d414ba6e57d25256afada3/misaki/data/us_silver.json", "de8f67be911bb6c659187b4a65fd966b6a30e56350e0f790d763210b053ac475"),
];

/// What `NOTICE` says about the changed graphs (the licence asks for changes to be marked).
const NOTICE: &str = "Supertonic 3 (Supertone Inc.), https://huggingface.co/Supertone/supertonic-3, revision 3cadd1ee.
Licensed under the BigScience OpenRAIL-M licence (LICENSE.txt), including its use restrictions.

Modified by Idyll Lives (cargo fetch-voice, crates/tools/src/fp16.rs): in duration_predictor,
text_encoder, vector_estimator and vocoder, every float32 weight of 1,024 values or more is stored
as float16 and cast back to float32 when the model loads; the files are gzipped (.onnxz). Nothing
was retrained. tts.json, unicode_indexer.json and the voice styles (F1-F5, M1-M5) are unchanged.
";

/// The model's folder (`web/public/voice/supertonic-3/`).
pub fn model_dir() -> PathBuf {
    out().join(MODEL_DIR)
}

fn out() -> PathBuf {
    crate::characters::repo_root().join("web").join("public").join("voice")
}

/// Where a published file is saved: its last path part, but the licence as a `.txt` so a static
/// host serves it as text and Credits' link shows it instead of downloading it.
fn out_name(path: &str) -> &str {
    if path == "LICENSE" {
        return "LICENSE.txt";
    }
    path.rsplit('/').next().unwrap_or(path)
}

pub fn main_with(_args: &[String]) -> i32 {
    match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("voice: {e}");
            1
        }
    }
}

fn run() -> Result<(), String> {
    let agent = agent();
    let out = out();
    let model = out.join(MODEL_DIR);
    std::fs::create_dir_all(&model).map_err(|e| format!("{}: {e}", model.display()))?;

    for (path, expected) in FILES {
        let file = model.join(out_name(path));
        if download::file_sha256(&file, false).as_deref() == Some(expected) {
            continue;
        }
        println!("voice: downloading {}", out_name(path));
        let bytes = download::download(&agent, &format!("{SUPERTONIC}/{path}"), expected)?;
        download::write(&file, &bytes)?;
    }
    for (name, sha, fp16) in GRAPHS {
        let file = model.join(format!("{name}.onnxz"));
        if download::file_sha256(&file, true).as_deref() == Some(fp16) {
            continue;
        }
        println!("voice: downloading {name}.onnx, storing its weights as float16");
        let published = download::download(&agent, &format!("{SUPERTONIC}/onnx/{name}.onnx"), sha)?;
        let (converted, weights, saved) = weights_to_fp16(&published)?;
        let actual = sha256_hex(&converted);
        if actual != fp16 {
            return Err(format!("{name}: float16 weights SHA-256 {actual}, expected {fp16}"));
        }
        download::write(&file, &gzip9(&converted)?)?;
        println!("voice: {name}: {weights} weights, {:.1} MB saved", saved as f64 / 1e6);
    }
    download::write(&model.join("NOTICE"), NOTICE.as_bytes())?;
    for name in OBSOLETE {
        download::remove(&out.join(name));
    }
    for (name, ..) in GRAPHS {
        download::remove(&model.join(format!("{name}.onnx")));
    }
    download::remove(&model.join("LICENSE"));

    let (license_url, license_out, license_sha) = LEXICON_LICENSE;
    if download::file_sha256(&out.join(license_out), false).as_deref() != Some(license_sha) {
        println!("voice: downloading {license_out}");
        let bytes = download::download(&agent, license_url, license_sha)?;
        download::write(&out.join(license_out), &bytes)?;
    }

    lexicon(&agent, &out)
}

/// The dictionary as `word<TAB>phonemes` lines, gzipped. Gold entries win over silver ones; for
/// words whose pronunciation depends on the part of speech only the default is kept (lines are
/// short and the engine has no tagger). Named `.lexz`, not `.gz`: servers send `.gz` files with
/// `Content-Encoding: gzip`, and the browser would unpack them on its own.
fn lexicon(agent: &ureq::Agent, out: &Path) -> Result<(), String> {
    let stamp: Vec<String> = LEXICON.iter().map(|(_, sha)| sha[..12].to_string()).collect();
    let stamp = stamp.join(" ");
    let stamp_path = out.join("en-us.lex.stamp");
    if std::fs::read_to_string(&stamp_path).is_ok_and(|s| s == stamp) {
        return Ok(());
    }
    println!("voice: building en-us.lexz");
    let mut words: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    // Silver first, then gold over it.
    for (url, sha) in [LEXICON[1], LEXICON[0]] {
        let text = download::download(agent, url, sha)?;
        let dict: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&text).map_err(|e| format!("{url}: {e}"))?;
        for (word, value) in dict {
            let phonemes = match &value {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Object(o) => o.get("DEFAULT").and_then(|v| v.as_str()).map(str::to_string),
                _ => None,
            };
            if let Some(p) = phonemes.filter(|p| !p.is_empty()) {
                words.insert(word, p);
            }
        }
    }
    let lines: Vec<String> = words.iter().map(|(w, p)| format!("{w}\t{p}")).collect();
    let text = format!("{}\n", lines.join("\n"));
    download::write(&out.join("en-us.lexz"), &gzip9(text.as_bytes())?)?;
    download::write(&stamp_path, stamp.as_bytes())?;
    println!("voice: {} words", lines.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_licence_is_saved_as_text() {
        assert_eq!(out_name("LICENSE"), "LICENSE.txt");
        assert_eq!(out_name("voice_styles/M5.json"), "M5.json");
        assert_eq!(out_name("onnx/tts.json"), "tts.json");
    }
}
