//! `cargo decoders`: refreshes `web/public/decoders/`, the texture decoder PlayCanvas would
//! otherwise fetch from a CDN, which the cross-origin isolated page can't use. Run after upgrading
//! PlayCanvas; the files are committed, so builds need no network. (Was tools/assets/decoders.mjs.)
//!
//! - `basis/`: PlayCanvas's Basis Universal transcoder (KTX2 textures; `render/pc/loaders.ts`),
//!   from the engine's repository at the `playcanvas` version in `web/package.json` (Apache-2.0),
//!   each file by its SHA-256: what comes down is the bytes committed, or the run stops.
//!
//! (The meshopt geometry decoder isn't here: `render/pc/loaders.ts` imports the meshoptimizer
//! package that also encodes our models.)

use crate::download::{self, agent};

/// PlayCanvas's Basis transcoder (glue and WebAssembly), in its repository's examples, and each one's
/// SHA-256 as committed.
const FROM_PLAYCANVAS: [(&str, &str); 2] = [
    (
        "basis.wasm.js",
        "82143c2193310331a3186530cc1615950f5ddd4d308aa6a25d82ad1d5933c624",
    ),
    (
        "basis.wasm.wasm",
        "70ed1e72dea7132706ee4e8637d9fc1e348ea03bb3a46644372f309067c47eef",
    ),
];

pub fn main_with(_args: &[String]) -> i32 {
    match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("decoders: {e}");
            1
        }
    }
}

fn run() -> Result<(), String> {
    let web = crate::repo_root().join("web");
    let out = web.join("public").join("decoders");
    let pkg: serde_json::Value = serde_json::from_slice(
        &std::fs::read(web.join("package.json")).map_err(|e| format!("web/package.json: {e}"))?,
    )
    .map_err(|e| format!("web/package.json: {e}"))?;
    let agent = agent();
    let pc = pkg["dependencies"]["playcanvas"]
        .as_str()
        .or(pkg["devDependencies"]["playcanvas"].as_str())
        .ok_or("web/package.json: no playcanvas")?;
    let pc_version = pc.trim_start_matches(|c: char| !c.is_ascii_digit());
    for (file, sha256) in FROM_PLAYCANVAS {
        let url = format!(
            "https://raw.githubusercontent.com/playcanvas/engine/v{pc_version}/examples/assets/wasm/basis/{file}"
        );
        let bytes = download::download(&agent, &url, sha256)?;
        download::write(&out.join("basis").join(file), &bytes)?;
        println!(
            "basis/{file} ({:.0} KB, PlayCanvas {pc_version})",
            bytes.len() as f64 / 1024.0
        );
    }
    Ok(())
}
