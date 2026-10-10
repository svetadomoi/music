import { ask, open } from '@tauri-apps/plugin-dialog';
import { api, srcOf } from './api';
import type { AppState, CoverPick, Playlist, Track } from './types';

export type Repeat = 'off' | 'all' | 'one';
export type ViewMode = 'compact' | 'detailed';

export const themes = [
  { id: 'dark', label: 'Тёмная' },
  { id: 'light', label: 'Светлая' },
  { id: 'midnight', label: 'Полночь' },
  { id: 'ocean', label: 'Океан' },
  { id: 'forest', label: 'Лес' },
  { id: 'sunset', label: 'Закат' },
] as const;

export type ThemeId = (typeof themes)[number]['id'];

function loadTheme(): ThemeId {
  try {
    const t = localStorage.getItem('theme');
    return themes.some((x) => x.id === t) ? (t as ThemeId) : 'dark';
  } catch {
    return 'dark';
  }
}

export interface Columns {
  cover: boolean;
  number: boolean;
  artist: boolean;
  album: boolean;
  duration: boolean;
}

export const columnList: { key: keyof Columns; label: string }[] = [
  { key: 'cover', label: 'Обложка' },
  { key: 'number', label: 'Номер в альбоме' },
  { key: 'artist', label: 'Исполнитель' },
  { key: 'album', label: 'Альбом' },
  { key: 'duration', label: 'Длительность' },
];

const defaultColumns: Columns = { cover: true, number: false, artist: true, album: true, duration: true };

function loadColumns(): Columns {
  try {
    return { ...defaultColumns, ...JSON.parse(localStorage.getItem('columns') ?? '{}') };
  } catch {
    return { ...defaultColumns };
  }
}

function loadMode(): ViewMode {
  try {
    return localStorage.getItem('viewMode') === 'detailed' ? 'detailed' : 'compact';
  } catch {
    return 'compact';
  }
}

export const app = $state({
  data: { folders: [], tracks: [], playlists: [] } as AppState,
  /** 'library', 'playlists' (сетка плейлистов, только в подробном режиме) или id плейлиста */
  view: 'library',
  mode: loadMode(),
  columns: loadColumns(),
  theme: loadTheme(),
  settingsOpen: false,
  /** id редактируемого плейлиста */
  editing: null as string | null,
  /** открытое окно создания плейлиста */
  creating: null as null | { kind: 'manual' | 'folder'; path: string | null },
  playlistQuery: '',
  /** пути обложек плейлистов (ключ — пути треков), null — обложки нет */
  covers: {} as Record<string, string | null>,
  busy: false,
  toast: '',
  // плеер
  queue: [] as Track[],
  /** плейлист, из которого собрана очередь (null — общий пул) */
  ctx: null as string | null,
  index: -1,
  playing: false,
  pos: 0,
  dur: 0,
  volume: 1,
  repeat: 'off' as Repeat,
});

document.documentElement.dataset.theme = app.theme;

const audio = new Audio();

// Короткие fade in/out громкости убирают щелчки и шипение при паузе, перемотке и смене трека.
let fadeLevel = 1;
let fadeCancel: (() => void) | null = null;

function applyVolume() {
  audio.volume = Math.min(1, Math.max(0, app.volume * fadeLevel));
}

function ramp(to: number, ms: number): Promise<void> {
  fadeCancel?.();
  const from = fadeLevel;
  if (from === to || ms <= 0) {
    fadeLevel = to;
    applyVolume();
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    const start = performance.now();
    let timer: ReturnType<typeof setTimeout>;
    const done = () => {
      clearTimeout(timer);
      fadeCancel = null;
      resolve();
    };
    const tick = () => {
      const k = Math.min(1, (performance.now() - start) / ms);
      fadeLevel = from + (to - from) * k;
      applyVolume();
      if (k >= 1) done();
      else timer = setTimeout(tick, 5);
    };
    fadeCancel = done;
    tick();
  });
}

const FADE_OUT_MS = 60;
const FADE_IN_MS = 90;
let toastTimer: ReturnType<typeof setTimeout> | undefined;

export function notify(msg: string) {
  app.toast = msg;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (app.toast = ''), 4500);
}

export const currentTrack = () => app.queue[app.index] as Track | undefined;
export const currentPlaylist = (): Playlist | undefined =>
  app.data.playlists.find((p) => p.id === app.view);

export function setMode(m: ViewMode) {
  app.mode = m;
  if (m === 'compact' && app.view === 'playlists') app.view = 'library';
  try {
    localStorage.setItem('viewMode', m);
  } catch {
    /* не критично */
  }
}

export function setTheme(t: ThemeId) {
  app.theme = t;
  document.documentElement.dataset.theme = t;
  try {
    localStorage.setItem('theme', t);
  } catch {
    /* не критично */
  }
}

export function setColumn(key: keyof Columns, on: boolean) {
  app.columns[key] = on;
  try {
    localStorage.setItem('columns', JSON.stringify(app.columns));
  } catch {
    /* не критично */
  }
}

const coverKey = (paths: string[]) => paths.join('|');
export const coverFor = (paths: string[]) => app.covers[coverKey(paths)];

export async function loadCover(paths: string[]) {
  const key = coverKey(paths);
  if (key in app.covers) return;
  app.covers[key] = null;
  try {
    app.covers[key] = await api.getCover(paths);
  } catch {
    /* оставляем без обложки */
  }
}

function setData(d: AppState) {
  app.data = d;
  if (app.view !== 'library' && app.view !== 'playlists' && !d.playlists.some((p) => p.id === app.view)) app.view = 'library';
}

async function run(fn: () => Promise<AppState>): Promise<boolean> {
  app.busy = true;
  try {
    setData(await fn());
    return true;
  } catch (e) {
    notify(String(e));
    return false;
  } finally {
    app.busy = false;
  }
}

// ---------- библиотека и плейлисты ----------

export async function init() {
  await run(() => api.getState());
}

export async function addFolder() {
  const dir = await open({ directory: true, title: 'Добавить папку с музыкой' });
  if (typeof dir === 'string') await run(() => api.addFolder(dir));
}

export async function removeFolder(path: string) {
  const ok = await ask(`Убрать папку из общего списка?\n${path}\nФайлы на диске останутся.`, {
    title: 'Удаление папки',
    kind: 'warning',
  });
  if (ok) await run(() => api.removeFolder(path));
}

export const createPlaylist = (name: string, cover: CoverPick | null) =>
  run(() => api.createPlaylist(name, cover));
export const createFolderPlaylist = (name: string, path: string, cover: CoverPick | null) =>
  run(() => api.createFolderPlaylist(name, path, cover));

export async function pickFolder(): Promise<string | null> {
  const dir = await open({ directory: true, title: 'Папка для плейлиста' });
  return typeof dir === 'string' ? dir : null;
}

export async function deletePlaylist(p: Playlist) {
  const ok = await ask(`Удалить плейлист «${p.name}»? Файлы на диске останутся.`, {
    title: 'Удаление плейлиста',
    kind: 'warning',
  });
  if (ok) await run(() => api.deletePlaylist(p.id));
}

export async function pickImage(): Promise<string | null> {
  const f = await open({
    title: 'Обложка плейлиста',
    filters: [{ name: 'Изображения', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp'] }],
  });
  return typeof f === 'string' ? f : null;
}

/** cover: undefined — не менять, null — сбросить, иначе новая картинка с областью */
export async function savePlaylistEdit(p: Playlist, name: string, cover: CoverPick | null | undefined) {
  const n = name.trim();
  if (!n) return false;
  if (n !== p.name && !(await run(() => api.renamePlaylist(p.id, n)))) return false;
  if (cover !== undefined && !(await run(() => api.setPlaylistCover(p.id, cover)))) return false;
  return true;
}

export const addToPlaylist = (playlistId: string, ids: string[]) =>
  run(() => api.addTracksToPlaylist(playlistId, ids));
export const removeTracks = (playlistId: string | null, ids: string[]) =>
  run(() => api.removeTracks(playlistId, ids));

export async function removeFolderFromPlaylist(playlistId: string, folder: string) {
  const ok = await ask(`Убрать из плейлиста все песни из папки?\n${folder}`, {
    title: 'Удаление папки из плейлиста',
    kind: 'warning',
  });
  if (ok) await run(() => api.removeFolderFromPlaylist(playlistId, folder));
}

export async function refresh() {
  if (await run(() => api.refresh())) {
    app.covers = {};
    notify('Файлы перечитаны');
  }
}

// ---------- плеер ----------

audio.addEventListener('timeupdate', () => (app.pos = audio.currentTime));
audio.addEventListener('durationchange', () => (app.dur = isFinite(audio.duration) ? audio.duration : 0));
audio.addEventListener('play', () => (app.playing = true));
audio.addEventListener('pause', () => (app.playing = false));
audio.addEventListener('ended', () => {
  if (app.repeat === 'one') void playIndex(app.index);
  else void next(true);
});
audio.addEventListener('error', async () => {
  const t = currentTrack();
  if (!t) return;
  if (await api.fileExists(t.path)) notify(`Не удалось воспроизвести «${t.title}» (формат не поддерживается?)`);
  else await handleMissing(t);
});

async function handleMissing(t: Track) {
  audio.pause();
  const remove = await ask(
    `Файл не найден:\n${t.path}\n\nУдалить запись из ${app.ctx ? 'плейлиста' : 'общего списка'}?`,
    { title: 'Файл отсутствует', kind: 'warning', okLabel: 'Удалить запись', cancelLabel: 'Оставить' },
  );
  if (!remove) return;
  const i = app.queue.findIndex((q) => q.id === t.id);
  if (!(await removeTracks(app.ctx, [t.id]))) return;
  if (i < 0) return;
  app.queue.splice(i, 1);
  if (app.queue[i]) await playIndex(i);
  else {
    app.index = -1;
    app.pos = 0;
  }
}

async function playIndex(i: number) {
  const t = app.queue[i];
  if (!t) return;
  app.index = i;
  if (!(await api.fileExists(t.path))) {
    await handleMissing(t);
    return;
  }
  if (!audio.paused) await ramp(0, FADE_OUT_MS);
  fadeLevel = 0;
  applyVolume();
  audio.src = srcOf(t.path);
  try {
    await audio.play();
    void ramp(1, FADE_IN_MS);
  } catch {
    fadeLevel = 1;
    applyVolume();
    /* ошибку покажет обработчик 'error' */
  }
}

export function playList(tracks: Track[], i: number, ctx: string | null) {
  app.queue = [...tracks];
  app.ctx = ctx;
  return playIndex(i);
}

export async function toggle() {
  if (!currentTrack()) return;
  if (audio.paused) {
    fadeLevel = 0;
    applyVolume();
    try {
      await audio.play();
    } finally {
      void ramp(1, FADE_IN_MS);
    }
  } else {
    await ramp(0, FADE_OUT_MS);
    audio.pause();
  }
}

export async function next(auto = false) {
  if (!app.queue.length) return;
  let i = app.index + 1;
  if (i >= app.queue.length) {
    if (app.repeat === 'off' && auto) {
      audio.pause();
      audio.currentTime = 0;
      return;
    }
    i = 0;
  }
  await playIndex(i);
}

export async function prev() {
  if (!app.queue.length) return;
  if (audio.currentTime > 3) await seek(0);
  else await playIndex(app.index > 0 ? app.index - 1 : app.queue.length - 1);
}

export async function seek(sec: number) {
  app.pos = sec;
  if (audio.paused) {
    audio.currentTime = sec;
    return;
  }
  await ramp(0, 25);
  audio.currentTime = sec;
  await new Promise<void>((r) => {
    const t = setTimeout(r, 120);
    audio.addEventListener('seeked', () => { clearTimeout(t); r(); }, { once: true });
  });
  void ramp(1, FADE_IN_MS);
}

let volumeBeforeMute = 1;

export function setVolume(v: number) {
  app.volume = v;
  applyVolume();
}

export function toggleMute() {
  if (app.volume > 0) {
    volumeBeforeMute = app.volume;
    setVolume(0);
  } else setVolume(volumeBeforeMute || 1);
}

export function cycleRepeat() {
  app.repeat = app.repeat === 'off' ? 'all' : app.repeat === 'all' ? 'one' : 'off';
}

export function fmt(sec: number | null | undefined) {
  if (sec == null || !isFinite(sec)) return '';
  const s = Math.floor(sec);
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, '0')}`;
}
