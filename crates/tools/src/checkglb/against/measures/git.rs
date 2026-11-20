//! Reading a revision out of git.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn git(root: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// Whether git knows `rev` as a commit of the checkout.
pub(crate) fn is_rev(root: &str, rev: &str) -> bool {
    !rev.starts_with('-')
        && git(
            root,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{rev}^{{commit}}"),
            ],
        )
        .is_ok()
}

/// Writes the files `paths` (from the checkout's root) as revision `rev` has them under `into`; one it has not is
/// left out.
pub(crate) fn checkout(root: &str, rev: &str, paths: &[String], into: &str) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot run git: {e}"))?;
    let mut ask = child.stdin.take().ok_or("git: no input")?;
    let mut answers = BufReader::new(child.stdout.take().ok_or("git: no output")?);
    let broken = |e: std::io::Error| format!("git cat-file: {e}");
    for path in paths {
        ask.write_all(format!("{rev}:{path}\n").as_bytes())
            .map_err(broken)?;
        ask.flush().map_err(broken)?;
        let mut head = String::new();
        answers.read_line(&mut head).map_err(broken)?;
        // "<object> blob <size>", or "<name> missing".
        let fields: Vec<&str> = head.split_whitespace().collect();
        let size = match fields.as_slice() {
            [.., "blob", size] => size
                .parse::<usize>()
                .map_err(|_| format!("git cat-file: {head}"))?,
            [.., "missing"] => continue,
            _ => {
                return Err(format!(
                    "git cat-file: {rev}:{path} is no file ({})",
                    head.trim()
                ));
            }
        };
        let mut data = vec![0u8; size + 1];
        answers.read_exact(&mut data).map_err(broken)?;
        data.pop();
        let to = Path::new(into).join(path);
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&to, data).map_err(|e| format!("{}: {e}", to.display()))?;
    }
    drop(ask);
    let _ = child.wait();
    Ok(())
}
