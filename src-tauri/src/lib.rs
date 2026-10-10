mod model;
mod scan;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Deserialize;
use tauri::{Manager, State};

use model::{AppState, Group, Kind, Playlist, Track};

struct Store {
    state: Mutex<AppState>,
    file: PathBuf,
}

impl Store {
    fn commit(
        &self,
        f: impl FnOnce(&mut AppState) -> Result<(), String>,
    ) -> Result<AppState, String> {
        let mut s = self.state.lock().map_err(|e| e.to_string())?;
        f(&mut s)?;
        let json = serde_json::to_string(&*s).map_err(|e| e.to_string())?;
        fs::write(&self.file, json).map_err(|e| e.to_string())?;
        Ok(s.clone())
    }

    fn snapshot(&self) -> Result<AppState, String> {
        Ok(self.state.lock().map_err(|e| e.to_string())?.clone())
    }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())
}

fn prune_manual_playlists(s: &mut AppState) {
    let ids: HashSet<&str> = s.tracks.iter().map(|t| t.id.as_str()).collect();
    for p in s.playlists.iter_mut().filter(|p| p.kind == Kind::Manual) {
        p.track_ids.retain(|id| ids.contains(id.as_str()));
    }
}

fn merge_into_pool(pool: &mut Vec<Track>, incoming: Vec<Track>) {
    let known: HashSet<String> = pool.iter().map(|t| t.id.clone()).collect();
    pool.extend(incoming.into_iter().filter(|t| !known.contains(&t.id)));
}

fn playlist_mut<'a>(s: &'a mut AppState, id: &str) -> Result<&'a mut Playlist, String> {
    s.playlists
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| "Плейлист не найден".to_string())
}

#[derive(Deserialize)]
struct Crop {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Deserialize)]
struct CoverInput {
    path: String,
    crop: Option<Crop>,
}

fn make_cover(app: &tauri::AppHandle, input: &CoverInput) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("covers");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut img =
        image::open(&input.path).map_err(|e| format!("Не удалось открыть картинку: {e}"))?;
    if let Some(c) = &input.crop {
        let (iw, ih) = (img.width(), img.height());
        let x = (c.x.max(0.0).round() as u32).min(iw.saturating_sub(1));
        let y = (c.y.max(0.0).round() as u32).min(ih.saturating_sub(1));
        let w = (c.w.round() as u32).clamp(1, iw - x);
        let h = (c.h.round() as u32).clamp(1, ih - y);
        img = img.crop_imm(x, y, w, h);
    }
    if img.width().max(img.height()) > 800 {
        img = img.resize(800, 800, image::imageops::FilterType::Lanczos3);
    }
    let dest = dir.join(format!("custom_{}.png", scan::new_id(&input.path)));
    img.save(&dest).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
fn get_state(store: State<'_, Store>) -> Result<AppState, String> {
    store.snapshot()
}

#[tauri::command]
async fn get_cover(app: tauri::AppHandle, paths: Vec<String>) -> Result<Option<String>, String> {
    let cache = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("covers");
    blocking(move || scan::find_cover(&paths, &cache)).await
}

#[tauri::command]
fn file_exists(path: String) -> bool {
    Path::new(&path).is_file()
}

#[tauri::command]
async fn add_folder(store: State<'_, Store>, path: String) -> Result<AppState, String> {
    if !Path::new(&path).is_dir() {
        return Err("Папка не найдена".into());
    }
    let p = path.clone();
    let tracks = blocking(move || scan::scan_folder(Path::new(&p), &p)).await?;
    store.commit(|s| {
        if !s.folders.contains(&path) {
            s.folders.push(path.clone());
        }
        // пересчёт папки целиком
        s.tracks.retain(|t| t.source != path);
        merge_into_pool(&mut s.tracks, tracks);
        Ok(())
    })
}

#[tauri::command]
fn remove_folder(store: State<'_, Store>, path: String) -> Result<AppState, String> {
    store.commit(|s| {
        s.folders.retain(|f| *f != path);
        s.tracks.retain(|t| t.source != path);
        prune_manual_playlists(s);
        Ok(())
    })
}

#[tauri::command(async)]
fn create_playlist(
    app: tauri::AppHandle,
    store: State<'_, Store>,
    name: String,
    cover: Option<CoverInput>,
) -> Result<AppState, String> {
    let cover = cover.map(|c| make_cover(&app, &c)).transpose()?;
    store.commit(|s| {
        s.playlists.push(Playlist {
            id: scan::new_id(&name),
            name,
            kind: Kind::Manual,
            track_ids: Vec::new(),
            root: None,
            groups: Vec::new(),
            cover,
        });
        Ok(())
    })
}

#[tauri::command]
async fn create_folder_playlist(
    app: tauri::AppHandle,
    store: State<'_, Store>,
    name: String,
    path: String,
    cover: Option<CoverInput>,
) -> Result<AppState, String> {
    if !Path::new(&path).is_dir() {
        return Err("Папка не найдена".into());
    }
    let p = path.clone();
    let groups = blocking(move || scan::scan_playlist_folder(Path::new(&p))).await?;
    let cover = blocking(move || cover.map(|c| make_cover(&app, &c)).transpose()).await??;
    store.commit(|s| {
        s.playlists.push(Playlist {
            id: scan::new_id(&name),
            name,
            kind: Kind::Folder,
            track_ids: Vec::new(),
            root: Some(path),
            groups,
            cover,
        });
        Ok(())
    })
}

#[tauri::command]
fn rename_playlist(store: State<'_, Store>, id: String, name: String) -> Result<AppState, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Название не может быть пустым".into());
    }
    store.commit(|s| {
        playlist_mut(s, &id)?.name = name;
        Ok(())
    })
}

#[tauri::command(async)]
fn set_playlist_cover(
    app: tauri::AppHandle,
    store: State<'_, Store>,
    id: String,
    cover: Option<CoverInput>,
) -> Result<AppState, String> {
    let new_cover = cover.map(|c| make_cover(&app, &c)).transpose()?;
    store.commit(|s| {
        let p = playlist_mut(s, &id)?;
        if let Some(old) = p.cover.take() {
            let _ = fs::remove_file(old);
        }
        p.cover = new_cover;
        Ok(())
    })
}

#[tauri::command]
fn delete_playlist(store: State<'_, Store>, id: String) -> Result<AppState, String> {
    store.commit(|s| {
        if let Some(p) = s.playlists.iter().find(|p| p.id == id) {
            if let Some(c) = &p.cover {
                let _ = fs::remove_file(c);
            }
        }
        s.playlists.retain(|p| p.id != id);
        Ok(())
    })
}

#[tauri::command]
fn add_tracks_to_playlist(
    store: State<'_, Store>,
    playlist_id: String,
    track_ids: Vec<String>,
) -> Result<AppState, String> {
    store.commit(|s| {
        let pool: HashSet<String> = s.tracks.iter().map(|t| t.id.clone()).collect();
        let p = playlist_mut(s, &playlist_id)?;
        if p.kind != Kind::Manual {
            return Err("В плейлист на основе папки нельзя добавлять песни".into());
        }
        for id in track_ids {
            if pool.contains(&id) && !p.track_ids.contains(&id) {
                p.track_ids.push(id);
            }
        }
        Ok(())
    })
}

#[tauri::command]
fn remove_tracks(
    store: State<'_, Store>,
    playlist_id: Option<String>,
    track_ids: Vec<String>,
) -> Result<AppState, String> {
    store.commit(|s| {
        let ids: HashSet<String> = track_ids.into_iter().collect();
        match playlist_id {
            None => {
                s.tracks.retain(|t| !ids.contains(&t.id));
                prune_manual_playlists(s);
            }
            Some(pid) => {
                let p = playlist_mut(s, &pid)?;
                p.track_ids.retain(|id| !ids.contains(id));
                for g in &mut p.groups {
                    g.tracks.retain(|t| !ids.contains(&t.id));
                }
                p.groups.retain(|g| !g.tracks.is_empty());
            }
        }
        Ok(())
    })
}

#[tauri::command]
fn remove_folder_from_playlist(
    store: State<'_, Store>,
    playlist_id: String,
    folder: String,
) -> Result<AppState, String> {
    store.commit(|s| {
        let in_folder: HashSet<String> = s
            .tracks
            .iter()
            .filter(|t| t.source == folder)
            .map(|t| t.id.clone())
            .collect();
        let p = playlist_mut(s, &playlist_id)?;
        if p.kind != Kind::Manual {
            return Err("Из плейлиста на основе папки можно удалить только сам плейлист".into());
        }
        p.track_ids.retain(|id| !in_folder.contains(id));
        Ok(())
    })
}

#[tauri::command]
async fn refresh(store: State<'_, Store>) -> Result<AppState, String> {
    let snap = store.snapshot()?;
    let folders = snap.folders.clone();
    let roots: Vec<(String, String)> = snap
        .playlists
        .iter()
        .filter(|p| p.kind == Kind::Folder)
        .filter_map(|p| p.root.clone().map(|r| (p.id.clone(), r)))
        .collect();

    // Недоступные папки
    let (scanned_folders, scanned_roots) = blocking(move || {
        let f: Vec<(String, Vec<Track>)> = folders
            .into_iter()
            .filter(|d| Path::new(d).is_dir())
            .map(|d| {
                let t = scan::scan_folder(Path::new(&d), &d);
                (d, t)
            })
            .collect();
        let r: Vec<(String, Vec<Group>)> = roots
            .into_iter()
            .filter(|(_, r)| Path::new(r).is_dir())
            .map(|(id, r)| (id, scan::scan_playlist_folder(Path::new(&r))))
            .collect();
        (f, r)
    })
    .await?;

    store.commit(|s| {
        for (dir, tracks) in scanned_folders {
            s.tracks.retain(|t| t.source != dir);
            merge_into_pool(&mut s.tracks, tracks);
        }
        for (id, groups) in scanned_roots {
            if let Ok(p) = playlist_mut(s, &id) {
                p.groups = groups;
            }
        }
        prune_manual_playlists(s);
        Ok(())
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            fs::create_dir_all(&dir)?;
            let file = dir.join("state.json");
            let state = fs::read_to_string(&file)
                .ok()
                .and_then(|s| serde_json::from_str::<AppState>(&s).ok())
                .unwrap_or_default();
            app.manage(Store {
                state: Mutex::new(state),
                file,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            file_exists,
            get_cover,
            add_folder,
            remove_folder,
            create_playlist,
            create_folder_playlist,
            rename_playlist,
            set_playlist_cover,
            delete_playlist,
            add_tracks_to_playlist,
            remove_tracks,
            remove_folder_from_playlist,
            refresh,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
