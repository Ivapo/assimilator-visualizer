//! Raw RGBA frames into an `ffmpeg` child (vis-001 §2.3, Phase 1 "Output"): `libx264`,
//! yuv420p, written to `<out>.partial` and renamed to `<out>` only after ffmpeg exits 0.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

use anyhow::{Context, Result, bail};

/// The `ffmpeg` executable on `PATH`, if there is one.
pub fn find_ffmpeg() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join("ffmpeg"))
        .find(|p| is_executable(p))
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn partial_path(out: &Path) -> PathBuf {
    let mut s = out.as_os_str().to_os_string();
    s.push(".partial");
    PathBuf::from(s)
}

pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    partial: PathBuf,
    out: PathBuf,
}

impl Encoder {
    pub fn start(ffmpeg: &Path, out: &Path, width: u32, height: u32, fps: u32) -> Result<Self> {
        let partial = partial_path(out);
        let mut child = Command::new(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"])
            .args(["-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{width}x{height}"), "-r", &fps.to_string()])
            .args(["-i", "-"])
            .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-f", "mp4"])
            .arg(&partial)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("ffmpeg failed to start: {}", ffmpeg.display()))?;
        let stdin = child.stdin.take();
        Ok(Encoder {
            child,
            stdin,
            partial,
            out: out.to_path_buf(),
        })
    }

    pub fn write_frame(&mut self, rgba: &[u8]) -> Result<()> {
        let stdin = self.stdin.as_mut().expect("open until finish");
        if stdin.write_all(rgba).is_err() {
            let msg = self.fail();
            bail!("ffmpeg failed: {msg}");
        }
        Ok(())
    }

    /// Close ffmpeg's input, wait for it, and move `<out>.partial` to `<out>`.
    pub fn finish(mut self) -> Result<()> {
        drop(self.stdin.take());
        let mut err = String::new();
        if let Some(mut e) = self.child.stderr.take() {
            let _ = e.read_to_string(&mut err);
        }
        let status = self
            .child
            .wait()
            .context("ffmpeg failed: cannot wait for it")?;
        if !status.success() {
            let _ = std::fs::remove_file(&self.partial);
            bail!("ffmpeg failed ({status}): {}", last_line(&err));
        }
        std::fs::rename(&self.partial, &self.out).with_context(|| {
            format!(
                "cannot move {} to {}",
                self.partial.display(),
                self.out.display()
            )
        })
    }

    fn fail(&mut self) -> String {
        drop(self.stdin.take());
        let mut err = String::new();
        if let Some(mut e) = self.child.stderr.take() {
            let _ = e.read_to_string(&mut err);
        }
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.partial);
        last_line(&err)
    }
}

impl Drop for Encoder {
    /// An encoder dropped before `finish` (an error mid-render) leaves no file behind.
    fn drop(&mut self) {
        if self.stdin.is_some() {
            drop(self.stdin.take());
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = std::fs::remove_file(&self.partial);
        }
    }
}

fn last_line(s: &str) -> String {
    s.lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("no message")
        .trim()
        .to_string()
}
