//! What the tools that fetch pinned files share (`cargo fetch-voice`, `cargo fetch-llm`,
//! `cargo decoders`, `cargo envconv`): SHA-256 of what came down, gzip, and writes that only land whole.

use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;

pub const UA: &str = "idyll-lives-tool-fetch/1.0";

/// An agent that waits as long as a big model takes (no overall timeout: a 200 MB download is slow
/// on a slow line, and it is checked by its hash, not by its clock).
pub fn agent() -> ureq::Agent {
    let connect = Some(std::time::Duration::from_secs(60));
    ureq::Agent::config_builder()
        .timeout_connect(connect)
        .build()
        .into()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The SHA-256 of a file's contents (of its unpacked contents if `packed`): `None` if it isn't
/// there or can't be read or unpacked.
pub fn file_sha256(path: &Path, packed: bool) -> Option<String> {
    let mut hash = Sha256::new();
    let file = std::fs::File::open(path).ok()?;
    if packed {
        let mut reader = flate2::read::GzDecoder::new(std::io::BufReader::new(file));
        std::io::copy(&mut reader, &mut hash).ok()?;
    } else {
        std::io::copy(&mut std::io::BufReader::new(file), &mut hash).ok()?;
    }
    Some(format!("{:x}", hash.finalize()))
}

pub fn gunzip(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(bytes)
        .read_to_end(&mut out)
        .map_err(|e| format!("gunzip: {e}"))?;
    Ok(out)
}

pub fn gzip9(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(9));
    enc.write_all(bytes).map_err(|e| format!("gzip: {e}"))?;
    enc.finish().map_err(|e| format!("gzip: {e}"))
}

/// Writes `bytes` to `path`, creating its folders; through a `.part` file, so a run that stops
/// halfway leaves no half file behind.
pub fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let part = with_suffix(path, ".part");
    std::fs::write(&part, bytes).map_err(|e| format!("{}: {e}", part.display()))?;
    std::fs::rename(&part, path).map_err(|e| format!("{}: {e}", path.display()))
}

fn with_suffix(path: &Path, suffix: &str) -> std::path::PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// The whole file at `url`, with no hash to check it against (a CDN's, pinned by version).
pub fn download_unhashed(agent: &ureq::Agent, url: &str) -> Result<Vec<u8>, String> {
    let mut resp = agent
        .get(url)
        .header("User-Agent", UA)
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    resp.body_mut()
        .with_config()
        .limit(u64::MAX)
        .read_to_vec()
        .map_err(|e| format!("{url}: {e}"))
}

/// The whole file at `url`, checked against its SHA-256.
pub fn download(agent: &ureq::Agent, url: &str, expected: &str) -> Result<Vec<u8>, String> {
    let mut resp = agent
        .get(url)
        .header("User-Agent", UA)
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let bytes = resp
        .body_mut()
        .with_config()
        .limit(u64::MAX)
        .read_to_vec()
        .map_err(|e| format!("{url}: {e}"))?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!("{url}: SHA-256 {actual}, expected {expected}"));
    }
    Ok(bytes)
}

/// Streams `url` to `path` (through a `.part` file), hashing on the way: a big model never sits in
/// memory, and a wrong file never lands. With `limit`, an answer longer than that many bytes fails
/// as soon as it passes it, rather than filling the disk before the hash can refuse it.
pub fn download_to(
    agent: &ureq::Agent,
    url: &str,
    path: &Path,
    expected: &str,
    limit: Option<u64>,
) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let resp = agent
        .get(url)
        .header("User-Agent", UA)
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let mut body = resp.into_body().into_reader();
    let part = with_suffix(path, ".part");
    let mut out = std::io::BufWriter::new(
        std::fs::File::create(&part).map_err(|e| format!("{}: {e}", part.display()))?,
    );
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    let mut total = 0u64;
    let copied = (|| {
        loop {
            let n = body.read(&mut buf).map_err(|e| format!("{url}: {e}"))?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if let Some(limit) = limit.filter(|&l| total > l) {
                return Err(format!("{url}: more than {limit} bytes"));
            }
            hash.update(&buf[..n]);
            out.write_all(&buf[..n])
                .map_err(|e| format!("{}: {e}", part.display()))?;
        }
        out.flush().map_err(|e| format!("{}: {e}", part.display()))
    })();
    drop(out);
    if let Err(e) = copied {
        let _ = std::fs::remove_file(&part);
        return Err(e);
    }
    let actual = format!("{:x}", hash.finalize());
    if actual != expected {
        let _ = std::fs::remove_file(&part);
        return Err(format!("{url}: SHA-256 {actual}, expected {expected}"));
    }
    std::fs::rename(&part, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Gzips `from` into `to` (through a `.part` file), then deletes `from`.
pub fn pack(from: &Path, to: &Path) -> Result<(), String> {
    let mut input = std::io::BufReader::new(
        std::fs::File::open(from).map_err(|e| format!("{}: {e}", from.display()))?,
    );
    let part = with_suffix(to, ".part");
    let mut enc = flate2::write::GzEncoder::new(
        std::io::BufWriter::new(
            std::fs::File::create(&part).map_err(|e| format!("{}: {e}", part.display()))?,
        ),
        flate2::Compression::new(9),
    );
    std::io::copy(&mut input, &mut enc).map_err(|e| format!("{}: {e}", from.display()))?;
    enc.finish()
        .map_err(|e| format!("{}: {e}", part.display()))?;
    drop(input);
    std::fs::rename(&part, to).map_err(|e| format!("{}: {e}", to.display()))?;
    std::fs::remove_file(from).map_err(|e| format!("{}: {e}", from.display()))
}

/// Removes `path` if it's there (a folder and what's in it too).
pub fn remove(path: &Path) {
    let _ = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    /// A server on localhost that answers one request with `body`; its URL.
    fn serve(body: Vec<u8>) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/file", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 2 {
                line.clear();
            }
            let mut stream = stream;
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        });
        url
    }

    #[test]
    fn a_download_lands_whole_and_a_long_one_not_at_all() {
        let dir = std::env::temp_dir().join(format!("ll-download-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("file.bin");
        let body = vec![7u8; 200_000];
        let sha = sha256_hex(&body);

        download_to(&agent(), &serve(body.clone()), &path, &sha, Some(200_000)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), body);
        std::fs::remove_file(&path).unwrap();

        let err = download_to(&agent(), &serve(body), &path, &sha, Some(100_000)).unwrap_err();
        assert!(err.ends_with("more than 100000 bytes"), "{err}");
        assert!(
            !path.exists() && !with_suffix(&path, ".part").exists(),
            "nothing left behind"
        );
        remove(&dir);
    }
}
