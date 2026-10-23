//! `cargo decoders`: refreshes `web/public/decoders/`, the texture and geometry decoders the
//! engines would otherwise fetch from a CDN, which the cross-origin isolated page can't use. Run
//! after upgrading PlayCanvas, Babylon or meshoptimizer; the files are committed, so builds need no
//! network. (Was tools/assets/decoders.mjs.)
//!
//! - `basis/`: PlayCanvas's Basis Universal transcoder (KTX2 textures; `render/pc/loaders.ts`),
//!   from the engine's repository at the `playcanvas` version in `web/package.json` (Apache-2.0).
//! - KTX2 (Basis Universal) texture decoder and transcoders for Babylon (the previews and dev
//!   pages until they move to PlayCanvas): Babylon's CDN, pinned to the `@babylonjs/core` version
//!   in `web/package.json`, in the CDN's layout (`render/babylon/loaders.ts` points
//!   `Tools.ScriptBaseUrl` there) (Apache-2.0; zstddec BSD).
//! - meshopt geometry decoder for Babylon: the meshoptimizer package that also encodes our models
//!   (MIT). (PlayCanvas's side imports the package itself.)

use crate::download::{self, agent};

/// PlayCanvas's Basis transcoder (glue and WebAssembly), in its repository's examples.
const FROM_PLAYCANVAS: [&str; 2] = ["basis.wasm.js", "basis.wasm.wasm"];

/// Babylon's CDN files, under the version in `web/package.json`.
const FROM_CDN: [&str; 10] = [
    "babylon.ktx2Decoder.js",
    "zstddec.wasm",
    "ktx2Transcoders/1/msc_basis_transcoder.js",
    "ktx2Transcoders/1/msc_basis_transcoder.wasm",
    "ktx2Transcoders/1/uastc_astc.wasm",
    "ktx2Transcoders/1/uastc_bc7.wasm",
    "ktx2Transcoders/1/uastc_r8_unorm.wasm",
    "ktx2Transcoders/1/uastc_rg8_unorm.wasm",
    "ktx2Transcoders/1/uastc_rgba8_srgb_v2.wasm",
    "ktx2Transcoders/1/uastc_rgba8_unorm_v2.wasm",
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
    let web = crate::characters::repo_root().join("web");
    let out = web.join("public").join("decoders");
    let pkg: serde_json::Value = serde_json::from_slice(&std::fs::read(web.join("package.json")).map_err(|e| format!("web/package.json: {e}"))?).map_err(|e| format!("web/package.json: {e}"))?;
    let pinned = pkg["dependencies"]["@babylonjs/core"].as_str().ok_or("web/package.json: no @babylonjs/core")?;
    let version = pinned.trim_start_matches(|c: char| !c.is_ascii_digit());
    let cdn = format!("https://cdn.babylonjs.com/v{version}");

    let agent = agent();
    let pc = pkg["dependencies"]["playcanvas"].as_str().or(pkg["devDependencies"]["playcanvas"].as_str()).ok_or("web/package.json: no playcanvas")?;
    let pc_version = pc.trim_start_matches(|c: char| !c.is_ascii_digit());
    for file in FROM_PLAYCANVAS {
        let url = format!("https://raw.githubusercontent.com/playcanvas/engine/v{pc_version}/examples/assets/wasm/basis/{file}");
        let bytes = download::download_unhashed(&agent, &url)?;
        download::write(&out.join("basis").join(file), &bytes)?;
        println!("basis/{file} ({:.0} KB, PlayCanvas {pc_version})", bytes.len() as f64 / 1024.0);
    }
    for file in FROM_CDN {
        let url = format!("{cdn}/{file}");
        let bytes = download::download_unhashed(&agent, &url)?;
        download::write(&out.join(file), &bytes)?;
        println!("{file} ({:.0} KB, Babylon {version})", bytes.len() as f64 / 1024.0);
    }

    // The UMD build: Babylon loads it as a classic script and uses the global MeshoptDecoder.
    let meshopt = web.join("node_modules").join("meshoptimizer");
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(meshopt.join("package.json")).map_err(|e| format!("meshoptimizer: {e} (bun install first)"))?).map_err(|e| format!("meshoptimizer: {e}"))?;
    let source = meshopt.join("meshopt_decoder.cjs");
    let bytes = std::fs::read(&source).map_err(|e| format!("{}: {e}", source.display()))?;
    download::write(&out.join("meshopt_decoder.js"), &bytes)?;
    println!("meshopt_decoder.js (meshoptimizer {})", meta["version"].as_str().unwrap_or("?"));
    Ok(())
}
