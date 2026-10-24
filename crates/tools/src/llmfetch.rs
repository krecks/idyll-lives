//! `cargo fetch-llm`: the language model that writes residents' lines, into `web/public/llm/`
//! (not in git; see `web/src/llm/models.ts`). (Was tools/llm/fetch.mjs.)
//!
//! - LFM2.5-350M (Liquid AI, LFM Open License v1.0), ONNX export by onnx-community, its 4-bit
//!   build as it is: https://huggingface.co/onnx-community/LFM2.5-350M-ONNX
//! - The licence goes along (`LICENSE`, which the licence asks for): the export's repository has
//!   none, so it comes from Liquid AI's own: https://huggingface.co/LiquidAI/LFM2.5-350M
//!
//! Everything is pinned to a revision and checked against its SHA-256. The weights are stored
//! gzipped (`model_q4.onnx_dataz`, about 13 % smaller; the browser unpacks them, `ml/assets.ts`).
//! Files already present with the right hash (unpacked) are kept. Big files are streamed.

use crate::download::{self, agent};
use std::path::PathBuf;

const LFM: &str = "https://huggingface.co/onnx-community/LFM2.5-350M-ONNX/resolve/7dd4999565b0342c381ba90a3d8fc467d6df19c4";
const LIQUID: &str = "https://huggingface.co/LiquidAI/LFM2.5-350M/resolve/9e6c6ccf47cd318696e137d381a7ded8fe4df09f";

/// A file of the model: its path in the repository, the name it's saved as, where it comes from,
/// and its SHA-256.
struct File {
    path: &'static str,
    out: &'static str,
    from: &'static str,
    sha256: &'static str,
    /// Stored gzipped (`<out>z`): the weights, which compress well and unpack in the browser.
    pack: bool,
}

const FILES: [File; 5] = [
    File { path: "tokenizer.json", out: "tokenizer.json", from: LFM, sha256: "29d43b4be8e8a896fefd7cd836ca6d6b4eedd249f823866ce0453b368e646f49", pack: false },
    File { path: "tokenizer_config.json", out: "tokenizer_config.json", from: LFM, sha256: "e70cd070c2696bf1b0f1ad303c34502753670cca6f844637821cb1c37e6bea8d", pack: false },
    File { path: "onnx/model_q4.onnx", out: "model_q4.onnx", from: LFM, sha256: "d1a705712e93aafaba1346b32245fa59e7857a46e2272003c0e8c524977e0de8", pack: false },
    File { path: "onnx/model_q4.onnx_data", out: "model_q4.onnx_data", from: LFM, sha256: "71ec6ad38a4c463dcb3dba671d06a1d9861be3a23e51290d818b95c0b7d2a5db", pack: true },
    // A .txt so a static host serves it as text and Credits' link shows it instead of downloading it.
    File { path: "LICENSE", out: "LICENSE.txt", from: LIQUID, sha256: "4d28ca14dedc0b3d0fcc2b3339f0e79931faa33874f3d24f522183a8fc70068c", pack: false },
];

/// Files earlier versions put into the site that are no longer used.
const OBSOLETE: [&str; 7] = ["LICENSE", "model_q4.onnx_data", "config.json", "model_i8.onnx", "model_i8.onnx_data", "model_q4f16.onnx", "model_q4f16.onnx_data"];

fn out() -> PathBuf {
    crate::characters::repo_root().join("web").join("public").join("llm").join("lfm2.5-350m")
}

pub fn main_with(_args: &[String]) -> i32 {
    match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("llm: {e}");
            1
        }
    }
}

fn run() -> Result<(), String> {
    let out = out();
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    for name in OBSOLETE {
        download::remove(&out.join(name));
    }
    let agent = agent();
    for file in &FILES {
        let name = file.out;
        let path = if file.pack { out.join(format!("{name}z")) } else { out.join(name) };
        if download::file_sha256(&path, file.pack).as_deref() == Some(file.sha256) {
            continue;
        }
        println!("llm: downloading {name}");
        let target = out.join(name);
        download::download_to(&agent, &format!("{}/{}", file.from, file.path), &target, file.sha256, None)?;
        if file.pack {
            println!("llm: packing {name}z");
            download::pack(&target, &path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every file the fetch writes has a name to look for, and the weights are the one packed.
    #[test]
    fn the_files_are_named_as_the_browser_expects() {
        assert_eq!(FILES.len(), 5);
        let packed: Vec<&str> = FILES.iter().filter(|f| f.pack).map(|f| f.out).collect();
        assert_eq!(packed, ["model_q4.onnx_data"]);
        assert!(FILES.iter().all(|f| !f.out.contains('/')));
    }
}
