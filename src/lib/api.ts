import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import type { AppState, CoverPick } from './types';

const coverArg = (c: CoverPick | null) => (c ? { path: c.path, crop: c.crop } : null);

export const api = {
  getState: () => invoke<AppState>('get_state'),
  fileExists: (path: string) => invoke<boolean>('file_exists', { path }),
  getCover: (paths: string[]) => invoke<string | null>('get_cover', { paths }),
  addFolder: (path: string) => invoke<AppState>('add_folder', { path }),
  removeFolder: (path: string) => invoke<AppState>('remove_folder', { path }),
  createPlaylist: (name: string, cover: CoverPick | null) =>
    invoke<AppState>('create_playlist', { name, cover: coverArg(cover) }),
  createFolderPlaylist: (name: string, path: string, cover: CoverPick | null) =>
    invoke<AppState>('create_folder_playlist', { name, path, cover: coverArg(cover) }),
  renamePlaylist: (id: string, name: string) => invoke<AppState>('rename_playlist', { id, name }),
  setPlaylistCover: (id: string, cover: CoverPick | null) =>
    invoke<AppState>('set_playlist_cover', { id, cover: coverArg(cover) }),
  deletePlaylist: (id: string) => invoke<AppState>('delete_playlist', { id }),
  addTracksToPlaylist: (playlistId: string, trackIds: string[]) =>
    invoke<AppState>('add_tracks_to_playlist', { playlistId, trackIds }),
  removeTracks: (playlistId: string | null, trackIds: string[]) =>
    invoke<AppState>('remove_tracks', { playlistId, trackIds }),
  removeFolderFromPlaylist: (playlistId: string, folder: string) =>
    invoke<AppState>('remove_folder_from_playlist', { playlistId, folder }),
  refresh: () => invoke<AppState>('refresh'),
};

export const srcOf = (path: string) => convertFileSrc(path);
