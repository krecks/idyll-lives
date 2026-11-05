//! `cargo fetch-llm`: the language model that writes residents' lines, into `web/public/llm/`
//! (not in git; see `web/src/llm/models.ts`). (Was tools/llm/fetch.mjs.)
//!
//! - LFM2.5-350M (Liquid AI, LFM Open License v1.0), ONNX export by onnx-community, its 4-bit
//!   build: https://huggingface.co/onnx-community/LFM2.5-350M-ONNX. Repacked smaller without
//!   changing what it computes (`llmslim.rs`: the tied embedding stored once, the rotary tables cut
//!   to 4,096 positions): 228 MB of weights instead of 294.
//! - The licence goes along (`LICENSE`, which the licence asks for): the export's repository has
//!   none, so it comes from Liquid AI's own: https://huggingface.co/LiquidAI/LFM2.5-350M
//!
//! Everything is pinned to a revision and checked against its SHA-256, the repacked files too. The
//! weights are stored gzipped (`model_q4.onnx_dataz`, about 14 % smaller; the browser unpacks
//! them, `ml/assets.ts`). Files already present with the right hash (unpacked) are kept. The
//! export is downloaded only when the repacked model isn't there, and deleted once it's made.

use crate::download::{self, agent};
use crate::llmslim;
use std::path::PathBuf;

const LFM: &str = "https://huggingface.co/onnx-community/LFM2.5-350M-ONNX/resolve/7dd4999565b0342c381ba90a3d8fc467d6df19c4";
const LIQUID: &str =
    "https://huggingface.co/LiquidAI/LFM2.5-350M/resolve/9e6c6ccf47cd318696e137d381a7ded8fe4df09f";

/// A file of the model: its path in the repository, the name it's saved as, where it comes from,
/// and its SHA-256.
struct File {
    path: &'static str,
    out: &'static str,
    from: &'static str,
    sha256: &'static str,
}

/// Files kept as they are.
const FILES: [File; 3] = [
    File {
        path: "tokenizer.json",
        out: "tokenizer.json",
        from: LFM,
        sha256: "29d43b4be8e8a896fefd7cd836ca6d6b4eedd249f823866ce0453b368e646f49",
    },
    File {
        path: "tokenizer_config.json",
        out: "tokenizer_config.json",
        from: LFM,
        sha256: "e70cd070c2696bf1b0f1ad303c34502753670cca6f844637821cb1c37e6bea8d",
    },
    // A .txt so a static host serves it as text and Credits' link shows it instead of downloading it.
    File {
        path: "LICENSE",
        out: "LICENSE.txt",
        from: LIQUID,
        sha256: "4d28ca14dedc0b3d0fcc2b3339f0e79931faa33874f3d24f522183a8fc70068c",
    },
];

/// The export's 4-bit graph and weights, downloaded under these names to be repacked.
const EXPORT: [File; 2] = [
    File {
        path: "onnx/model_q4.onnx",
        out: "export-model_q4.onnx",
        from: LFM,
        sha256: "d1a705712e93aafaba1346b32245fa59e7857a46e2272003c0e8c524977e0de8",
    },
    File {
        path: "onnx/model_q4.onnx_data",
        out: "export-model_q4.onnx_data",
        from: LFM,
        sha256: "71ec6ad38a4c463dcb3dba671d06a1d9861be3a23e51290d818b95c0b7d2a5db",
    },
];

/// The repacked graph, and its weights (stored gzipped as `<name>z`; the SHA-256 is the
/// unpacked file's). The graph names the weights file by this name.
const GRAPH: (&str, &str) = (
    "model_q4.onnx",
    "e17d03a87346295631d71358fd98abad930172bbf52819a26c34ca4d1dc4d04c",
);
const DATA: (&str, &str) = (
    "model_q4.onnx_data",
    "3b7570c237498b56b6276ceae327a11dd7f3e96dee8a4eaa3dad27548ee6a9de",
);

/// Files earlier versions put into the site that are no longer used.
const OBSOLETE: [&str; 9] = [
    "LICENSE",
    "export-model_q4.onnx",
    "export-model_q4.onnx_data",
    "model_q4.onnx_data",
    "config.json",
    "model_i8.onnx",
    "model_i8.onnx_data",
    "model_q4f16.onnx",
    "model_q4f16.onnx_data",
];

fn out() -> PathBuf {
    crate::characters::repo_root()
        .join("web")
        .join("public")
        .join("llm")
        .join("lfm2.5-350m")
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
    let fetch = |file: &File| -> Result<(), String> {
        println!("llm: downloading {}", file.out);
        download::download_to(
            &agent,
            &format!("{}/{}", file.from, file.path),
            &out.join(file.out),
            file.sha256,
            None,
        )
    };
    for file in &FILES {
        if download::file_sha256(&out.join(file.out), false).as_deref() != Some(file.sha256) {
            fetch(file)?;
        }
    }
    let packed = out.join(format!("{}z", DATA.0));
    if download::file_sha256(&out.join(GRAPH.0), false).as_deref() == Some(GRAPH.1)
        && download::file_sha256(&packed, true).as_deref() == Some(DATA.1)
    {
        return Ok(());
    }
    for file in &EXPORT {
        fetch(file)?;
    }
    let read = |name: &str| {
        let path = out.join(name);
        std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
    };
    let slim = llmslim::slim(
        &read(EXPORT[0].out)?,
        &read(EXPORT[1].out)?,
        llmslim::POSITIONS,
    )?;
    println!(
        "llm: repacked: stored once {} weight(s) of {:.1} MB, rotary tables cut by {:.1} MB",
        slim.shared,
        slim.shared_bytes as f64 / 1e6,
        slim.cut_bytes as f64 / 1e6
    );
    for (name, bytes, want) in [
        (GRAPH.0, &slim.graph, GRAPH.1),
        (DATA.0, &slim.data, DATA.1),
    ] {
        let got = download::sha256_hex(bytes);
        if got != want {
            return Err(format!("repacked {name}: SHA-256 {got}, expected {want}"));
        }
    }
    download::write(&out.join(GRAPH.0), &slim.graph)?;
    download::write(&out.join(DATA.0), &slim.data)?;
    println!("llm: packing {}z", DATA.0);
    download::pack(&out.join(DATA.0), &packed)?;
    for file in &EXPORT {
        download::remove(&out.join(file.out));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every file the fetch writes has a name to look for: the graph names its weights by the
    /// export's name (`model_q4.onnx_data`), which the worker gives the unpacked file.
    #[test]
    fn the_files_are_named_as_the_browser_expects() {
        assert_eq!((GRAPH.0, DATA.0), ("model_q4.onnx", "model_q4.onnx_data"));
        assert!(FILES.iter().chain(&EXPORT).all(|f| !f.out.contains('/')));
        // The export's files are cleaned up if a run stopped before it could.
        assert!(EXPORT.iter().all(|f| OBSOLETE.contains(&f.out)));
    }
}
