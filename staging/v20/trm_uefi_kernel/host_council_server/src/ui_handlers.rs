//! Static serving for the built dashboard (`dashboard/dist`).
//!
//! The UI and the API are served from ONE origin, which is the whole point: the app's fetches are
//! root-relative (`API_BASE = ''`), so when this kernel serves both, every call resolves to this
//! process with no proxy and no Node dev server. That is what retires the four Node processes the
//! platform ran for a single 811 KB bundle.
//!
//! Path traversal is refused by construction: only Normal and CurDir path components survive, so
//! `/ui/../../secrets` cannot escape the dist root.

use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use tiny_http::{Header, Response};

pub type Resp = Response<Cursor<Vec<u8>>>;

fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

/// Find the built dashboard by walking up from the executable. The exe lives at
/// <repo>/trm_uefi_kernel/host_council_server/target/release/, so the dist root is a few levels up.
/// TRM_UI_DIST overrides. Nothing is assumed: a candidate only counts if it holds an index.html.
pub fn dist_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("TRM_UI_DIST") {
        let p = PathBuf::from(p);
        if p.join("index.html").is_file() {
            return Some(p);
        }
    }
    let mut dir = std::env::current_exe().ok()?;
    for _ in 0..6 {
        dir = dir.parent()?.to_path_buf();
        let cand = dir.join("dashboard").join("dist");
        if cand.join("index.html").is_file() {
            return Some(cand);
        }
    }
    None
}

/// Map a URL path under the dist root, dropping anything that could escape it.
fn resolve(root: &Path, url: &str) -> Option<PathBuf> {
    let rel = url.trim_start_matches('/');
    let mut out = root.to_path_buf();
    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

fn file_body(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

/// Serves everything under /ui. Returns None for anything else so other handlers still run.
pub fn handle(url: &str) -> Option<Resp> {
    // /ui WITHOUT a trailing slash must REDIRECT, not serve. index.html references relative assets
    // (./assets/..., ./vite.svg), so at /ui the browser's base URL is "/" and those resolve to
    // /assets/... and /vite.svg -- both of which this handler does not claim, so they 404. The page
    // then loads as an empty shell with no JavaScript: the SPA never mounts and no page ever updates.
    // That was the exact "all pages are not updating" symptom.
    if url == "/ui" {
        return Some(
            Response::from_data(Vec::new())
                .with_status_code(301)
                .with_header(Header::from_bytes(&b"Location"[..], &b"/ui/"[..]).unwrap()),
        );
    }
    // Root-level asset paths are accepted as well, so the app still works if anything reaches it
    // through a base URL without the /ui/ prefix.
    let root_asset = url.starts_with("/assets/") || url == "/vite.svg";
    if !root_asset && !url.starts_with("/ui/") {
        return None;
    }
    let root = match dist_root() {
        Some(r) => r,
        None => {
            let msg = "{\"error\":\"ui_unavailable\",\"detail\":\"dashboard/dist not found; set TRM_UI_DIST\"}";
            return Some(
                Response::from_string(msg)
                    .with_status_code(503)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                            .unwrap(),
                    ),
            );
        }
    };

    // /ui/ is the app root; /ui/<path> and the root-level asset paths are either real files or
    // client-side routes.
    let as_file = if root_asset {
        resolve(&root, url).unwrap_or_else(|| root.join("index.html"))
    } else if url == "/ui/" {
        root.join("index.html")
    } else {
        resolve(&root, &url[3..]).unwrap_or_else(|| root.join("index.html"))
    };

    let (path, body) = match file_body(&as_file) {
        Some(b) => (as_file, b),
        // SPA fallback: an unknown path under /ui is a client-side route, not a 404.
        None => {
            let idx = root.join("index.html");
            match file_body(&idx) {
                Some(b) => (idx, b),
                None => return None,
            }
        }
    };

    let ct = content_type(&path);
    let len = body.len();
    Some(
        Response::from_data(body)
            .with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], ct.as_bytes()).unwrap(),
            )
            .with_header(
                tiny_http::Header::from_bytes(&b"Content-Length"[..], len.to_string().as_bytes())
                    .unwrap(),
            ),
    )
}
