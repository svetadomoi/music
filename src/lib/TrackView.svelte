<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import {
    app,
    currentTrack,
    playList,
    fmt,
    addToPlaylist,
    removeTracks,
    removeFolder,
    removeFolderFromPlaylist,
  } from './store.svelte';
  import SearchBox from './SearchBox.svelte';
  import TrackCover from './TrackCover.svelte';
  import type { Track } from './types';

  let query = $state('');
  let target = $state('');
  const selected = new SvelteSet<string>();

  const playlist = $derived(app.data.playlists.find((p) => p.id === app.view));
  const manualPlaylists = $derived(app.data.playlists.filter((p) => p.kind === 'manual'));
  const playing = $derived(currentTrack());

  const sections = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const match = (t: Track) =>
      !q || [t.title, t.artist, t.album].some((s) => s.toLowerCase().includes(q));

    let raw: { name: string; tracks: Track[] }[];
    if (!playlist) {
      raw = [{ name: '', tracks: app.data.tracks }];
    } else if (playlist.kind === 'folder') {
      raw = playlist.groups.map((g) => ({ name: g.name, tracks: g.tracks }));
    } else {
      const byId = new Map(app.data.tracks.map((t) => [t.id, t]));
      const tracks = playlist.track_ids.map((id) => byId.get(id)).filter((t): t is Track => !!t);
      raw = [{ name: '', tracks }];
    }

    let offset = 0;
    return raw
      .map((s) => ({ ...s, tracks: s.tracks.filter(match) }))
      .filter((s) => s.tracks.length > 0 || (!q && playlist?.kind === 'folder'))
      .map((s) => {
        const section = { ...s, offset };
        offset += s.tracks.length;
        return section;
      });
  });

  const flat = $derived(sections.flatMap((s) => s.tracks));

  /** Папки-источники: для общего списка — папки библиотеки, для обычного плейлиста — папки его песен. */
  const folderChips = $derived.by(() => {
    if (!playlist) return app.data.folders;
    if (playlist.kind === 'folder') return [];
    return [...new Set(flat.map((t) => t.source))];
  });

  const title = $derived(playlist ? playlist.name : 'Все песни');
  const canSelect = $derived(!playlist || playlist.kind === 'manual');

  $effect(() => {
    app.view; // сброс при смене вида
    selected.clear();
    query = '';
  });

  const template = $derived(
    [
      canSelect && '24px',
      '28px',
      app.columns.cover && '36px',
      app.columns.number && '32px',
      'minmax(0, 2fr)',
      app.columns.artist && 'minmax(0, 1fr)',
      app.columns.album && 'minmax(0, 1fr)',
      app.columns.duration && '52px',
    ]
      .filter(Boolean)
      .join(' '),
  );

  const label = (path: string) => path.split(/[\\/]/).filter(Boolean).pop() ?? path;

  function toggleAll() {
    if (selected.size === flat.length) selected.clear();
    else flat.forEach((t) => selected.add(t.id));
  }

  async function addSelected() {
    if (!target || !selected.size) return;
    if (await addToPlaylist(target, [...selected])) selected.clear();
  }

  async function removeSelected() {
    if (!selected.size) return;
    if (await removeTracks(playlist?.id ?? null, [...selected])) selected.clear();
  }

  function onChipRemove(path: string) {
    if (playlist) void removeFolderFromPlaylist(playlist.id, path);
    else void removeFolder(path);
  }

  const play = (i: number) => playList(flat, i, playlist?.id ?? null);
</script>

<header>
  <div>
    {#if playlist && app.mode === 'detailed'}
      <button class="back" onclick={() => (app.view = 'playlists')}>← Плейлисты</button>
    {/if}
    <h2>{title}</h2>
    <div class="meta">
      {flat.length} песен
      {#if playlist?.kind === 'folder'}· плейлист из папки: {playlist.root}{/if}
    </div>
  </div>
  <div class="head-tools">
    {#if playlist}<button onclick={() => (app.editing = playlist.id)}>✎ Изменить</button>{/if}
    <SearchBox bind:value={query} />
  </div>
</header>

{#if folderChips.length || canSelect}
  <div class="bar">
    {#each folderChips as f (f)}
      <span class="chip" title={f}>
        📁 {label(f)}
        <button onclick={() => onChipRemove(f)} title={playlist ? 'Убрать папку из плейлиста' : 'Убрать папку из общего списка'}>✕</button>
      </span>
    {/each}
    {#if canSelect && flat.length}
      <span class="spacer"></span>
      <button onclick={toggleAll}>{selected.size === flat.length ? 'Снять выделение' : 'Выделить все'}</button>
      {#if !playlist}
        <select bind:value={target}>
          <option value="">В плейлист…</option>
          {#each manualPlaylists as p (p.id)}<option value={p.id}>{p.name}</option>{/each}
        </select>
        <button onclick={addSelected} disabled={!target || !selected.size}>Добавить ({selected.size})</button>
      {/if}
      <button onclick={removeSelected} disabled={!selected.size}>
        {playlist ? 'Убрать из плейлиста' : 'Удалить запись'} ({selected.size})
      </button>
    {/if}
  </div>
{/if}

{#if flat.length}
  <div class="cols head" style:grid-template-columns={template}>
    {#if canSelect}<span></span>{/if}
    <span></span>
    {#if app.columns.cover}<span></span>{/if}
    {#if app.columns.number}<span class="r">№</span>{/if}
    <span>Название</span>
    {#if app.columns.artist}<span>Исполнитель</span>{/if}
    {#if app.columns.album}<span>Альбом</span>{/if}
    {#if app.columns.duration}<span class="r">Время</span>{/if}
  </div>
{/if}

<div class="list">
  {#if flat.length === 0 && !sections.some((s) => s.name)}
    <p class="empty">
      {#if !playlist}Добавьте папку с музыкой.
      {:else if playlist.kind === 'manual'}Плейлист пуст. Выделите песни в «Все песни» и добавьте их сюда.
      {:else}В папке не найдено аудиофайлов. Попробуйте «⟳ Обновить».{/if}
    </p>
  {/if}

  {#each sections as s (s.name + s.offset)}
    {#if s.name}
      <div class="group">💿 {s.name} <small>{s.tracks.length}</small></div>
    {/if}
    {#each s.tracks as t, i (t.id)}
      {@const idx = s.offset + i}
      <div
        class="track"
        style:grid-template-columns={template}
        class:current={playing?.id === t.id}
        role="button"
        tabindex="0"
        ondblclick={() => play(idx)}
        onkeydown={(e) => e.key === 'Enter' && play(idx)}
      >
        {#if canSelect}
          <input
            type="checkbox"
            checked={selected.has(t.id)}
            onchange={(e) => (e.currentTarget.checked ? selected.add(t.id) : selected.delete(t.id))}
          />
        {/if}
        <button class="play" onclick={() => play(idx)} title="Играть">
          {playing?.id === t.id && app.playing ? '♪' : '▶'}
        </button>
        {#if app.columns.cover}<TrackCover track={t} />{/if}
        {#if app.columns.number}<span class="no">{t.track_no ?? ''}</span>{/if}
        <span class="t">{t.title}</span>
        {#if app.columns.artist}<span class="a">{t.artist}</span>{/if}
        {#if app.columns.album}<span class="a">{t.album}</span>{/if}
        {#if app.columns.duration}<span class="dur">{fmt(t.duration)}</span>{/if}
      </div>
    {/each}
  {/each}
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    padding: 18px 20px 10px;
    gap: 16px;
  }
  h2 {
    margin: 0;
  }
  .meta {
    color: var(--muted);
    font-size: 12px;
    max-width: 520px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .back {
    margin-bottom: 6px;
    padding: 3px 8px;
    font-size: 12px;
  }
  .head-tools {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 0 20px 10px;
  }
  .spacer {
    flex: 1;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 4px 2px 10px;
    border-radius: 999px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    font-size: 12px;
  }
  .chip button {
    background: none;
    border: none;
    padding: 2px 6px;
    color: var(--muted);
  }
  .chip button:hover {
    color: var(--danger);
  }
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 0 12px 12px;
  }
  .empty {
    color: var(--muted);
    padding: 24px 8px;
  }
  .group {
    margin: 16px 8px 4px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border);
    font-weight: 600;
    color: var(--accent);
  }
  .group small {
    color: var(--muted);
    font-weight: 400;
  }
  .cols {
    display: grid;
    align-items: center;
    column-gap: 10px;
  }
  .head {
    padding: 4px 20px;
    margin-right: 10px;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--muted);
    border-bottom: 1px solid var(--border);
  }
  .r {
    text-align: right;
  }
  .track {
    display: grid;
    align-items: center;
    column-gap: 10px;
    padding: 4px 8px;
    border-radius: 6px;
  }
  .track:hover {
    background: var(--panel-2);
  }
  .track.current {
    background: var(--accent-bg);
  }
  .track.current .t {
    color: var(--accent);
  }
  .play {
    background: none;
    border: none;
    padding: 2px 6px;
    color: var(--muted);
    width: 28px;
  }
  .track:hover .play {
    color: var(--text);
  }
  .no {
    text-align: right;
    color: var(--muted);
    font-size: 12px;
  }
  .t,
  .a {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .a {
    font-size: 13px;
    color: var(--muted);
  }
  .dur {
    text-align: right;
    color: var(--muted);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }
</style>
