use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use lofty::picture::{MimeType, PictureType};
use lofty::prelude::*;
use walkdir::WalkDir;

use crate::model::{Group, Track};

const EXTS: &[&str] = &["mp3", "flac", "ogg", "oga", "opus", "wav", "m4a", "aac"];

pub fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn track_id(path: &str) -> String {
    format!("{:016x}", fnv1a(path.to_lowercase().as_bytes()))
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn new_id(seed: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    track_id(&format!("{seed}{nanos}"))
}

pub fn read_track(path: &Path, source: &str) -> Track {
    let path_str = path.to_string_lossy().to_string();
    let file_stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.clone());

    let mut t = Track {
        id: track_id(&path_str),
        path: path_str,
        title: file_stem,
        artist: String::new(),
        album: String::new(),
        track_no: None,
        disc_no: None,
        duration: None,
        source: source.to_string(),
    };

    if let Ok(tagged) = lofty::read_from_path(path) {
        let secs = tagged.properties().duration().as_secs_f64();
        if secs > 0.0 {
            t.duration = Some(secs);
        }
        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            if let Some(v) = tag.title().filter(|s| !s.trim().is_empty()) {
                t.title = v.trim().to_string();
            }
            if let Some(v) = tag.artist() {
                t.artist = v.trim().to_string();
            }
            if let Some(v) = tag.album() {
                t.album = v.trim().to_string();
            }
            t.track_no = tag.track();
            t.disc_no = tag.disk();
        }
    }
    t
}

fn audio_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_audio(e.path()))
        .map(|e| e.into_path())
        .collect()
}

pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.to_lowercase(), b.to_lowercase());
    let (mut ai, mut bi) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let mut na = String::new();
                let mut nb = String::new();
                while let Some(c) = ai.peek().copied().filter(|c| c.is_ascii_digit()) {
                    na.push(c);
                    ai.next();
                }
                while let Some(c) = bi.peek().copied().filter(|c| c.is_ascii_digit()) {
                    nb.push(c);
                    bi.next();
                }
                let (na, nb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let ord = na.len().cmp(&nb.len()).then_with(|| na.cmp(nb));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                if x != y {
                    return x.cmp(&y);
                }
                ai.next();
                bi.next();
            }
        }
    }
}

/// Порядок внутри папки/диска
fn sort_in_folder(tracks: &mut [Track]) {
    tracks.sort_by(|a, b| {
        a.disc_no
            .cmp(&b.disc_no)
            .then(a.track_no.cmp(&b.track_no))
            .then_with(|| natural_cmp(&a.path, &b.path))
    });
}

/// Скан папки для общего пула.
pub fn scan_folder(dir: &Path, source: &str) -> Vec<Track> {
    let mut tracks: Vec<Track> = audio_files(dir)
        .iter()
        .map(|p| read_track(p, source))
        .collect();
    tracks.sort_by(|a, b| {
        natural_cmp(&a.artist, &b.artist)
            .then_with(|| natural_cmp(&a.album, &b.album))
            .then(a.disc_no.cmp(&b.disc_no))
            .then(a.track_no.cmp(&b.track_no))
            .then_with(|| natural_cmp(&a.path, &b.path))
    });
    tracks
}

pub fn scan_playlist_folder(root: &Path) -> Vec<Group> {
    let root_str = root.to_string_lossy().to_string();
    let mut root_tracks: Vec<Track> = Vec::new();
    let mut subs: Vec<(String, Vec<Track>)> = Vec::new();

    for p in audio_files(root) {
        let track = read_track(&p, &root_str);
        let first_dir = p
            .strip_prefix(root)
            .ok()
            .filter(|rel| rel.components().count() > 1)
            .and_then(|rel| rel.components().next())
            .map(|c| c.as_os_str().to_string_lossy().to_string());
        match first_dir {
            None => root_tracks.push(track),
            Some(name) => match subs.iter_mut().find(|(n, _)| *n == name) {
                Some((_, v)) => v.push(track),
                None => subs.push((name, vec![track])),
            },
        }
    }

    if subs.is_empty() {
        if root_tracks.is_empty() {
            return Vec::new();
        }
        sort_in_folder(&mut root_tracks);
        return vec![Group {
            name: String::new(),
            folder: root_str,
            tracks: root_tracks,
        }];
    }

    let mut groups = Vec::new();
    if !root_tracks.is_empty() {
        sort_in_folder(&mut root_tracks);
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root_str.clone());
        groups.push(Group {
            name,
            folder: root_str.clone(),
            tracks: root_tracks,
        });
    }
    subs.sort_by(|a, b| natural_cmp(&a.0, &b.0));
    for (name, mut tracks) in subs {
        sort_in_folder(&mut tracks);
        groups.push(Group {
            folder: root.join(&name).to_string_lossy().to_string(),
            name,
            tracks,
        });
    }
    groups
}

const COVER_NAMES: &[&str] = &["cover", "folder", "front", "album", "albumart", "artwork"];

fn find_folder_image(dir: &Path) -> Option<PathBuf> {
    let entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    for name in COVER_NAMES {
        for p in &entries {
            let stem = p
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());
            let ext = p
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());
            if stem.as_deref() == Some(name)
                && matches!(ext.as_deref(), Some("jpg" | "jpeg" | "png"))
            {
                return Some(p.clone());
            }
        }
    }
    None
}

/// Достаёт встроенную обложку.
fn embedded_cover(path: &Path, cache: &Path) -> Option<PathBuf> {
    let tagged = lofty::read_from_path(path).ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag())?;
    let pic = tag
        .pictures()
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| tag.pictures().first())?;
    let ext = match pic.mime_type()? {
        MimeType::Png => "png",
        MimeType::Jpeg => "jpg",
        _ => return None,
    };

    let out = cache.join(format!("emb_{:016x}.{ext}", fnv1a(pic.data())));
    if !out.is_file() {
        std::fs::create_dir_all(cache).ok()?;
        std::fs::write(&out, pic.data()).ok()?;
    }
    Some(out)
}

pub fn find_cover(paths: &[String], cache: &Path) -> Option<String> {
    for p in paths {
        let p = Path::new(p);
        let found = if p.is_dir() {
            find_folder_image(p)
        } else {
            p.parent()
                .and_then(find_folder_image)
                .or_else(|| embedded_cover(p, cache))
        };
        if let Some(f) = found {
            return Some(f.to_string_lossy().to_string());
        }
    }
    None
}
