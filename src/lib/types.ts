export interface Track {
  id: string;
  path: string;
  title: string;
  artist: string;
  album: string;
  track_no: number | null;
  disc_no: number | null;
  duration: number | null;
  source: string;
}

export interface Group {
  name: string;
  folder: string;
  tracks: Track[];
}

export interface Playlist {
  id: string;
  name: string;
  kind: 'manual' | 'folder';
  track_ids: string[];
  root: string | null;
  groups: Group[];
  cover: string | null;
}

export interface AppState {
  folders: string[];
  tracks: Track[];
  playlists: Playlist[];
}

/** Область картинки в её собственных пикселях. */
export interface Crop {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Выбранная картинка обложки и вырезанная область (nw/nh — размер оригинала). */
export interface CoverPick {
  path: string;
  crop: Crop;
  nw: number;
  nh: number;
}
