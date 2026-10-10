<script lang="ts">
  import { app, deletePlaylist, loadCover, coverFor } from './store.svelte';
  import { srcOf } from './api';
  import SearchBox from './SearchBox.svelte';
  import type { Playlist } from './types';

  const byId = $derived(new Map(app.data.tracks.map((t) => [t.id, t])));

  /** Пути, по которым ищется обложка: корень папки + первые треки. */
  function coverPaths(p: Playlist): string[] {
    if (p.kind === 'folder') {
      const first = p.groups.flatMap((g) => g.tracks.slice(0, 3)).slice(0, 3);
      return [...(p.root ? [p.root] : []), ...first.map((t) => t.path)];
    }
    return p.track_ids
      .slice(0, 5)
      .map((id) => byId.get(id)?.path)
      .filter((x): x is string => !!x);
  }

  const items = $derived(
    app.data.playlists
      .filter((p) => p.name.toLowerCase().includes(app.playlistQuery.trim().toLowerCase()))
      .map((p) => {
      const paths = coverPaths(p);
      const count =
        p.kind === 'folder' ? p.groups.reduce((n, g) => n + g.tracks.length, 0) : p.track_ids.length;
      return { p, paths, count };
    }),
  );

  $effect(() => {
    for (const it of items) if (!it.p.cover && it.paths.length) void loadCover(it.paths);
  });
</script>

<header>
  <div>
    <h2>Плейлисты</h2>
    <div class="meta">{items.length} из {app.data.playlists.length}</div>
  </div>
  <SearchBox bind:value={app.playlistQuery} placeholder="Поиск плейлистов…" />
</header>

<div class="grid">
  {#each items as { p, paths, count } (p.id)}
    {@const cover = p.cover ?? coverFor(paths)}
    <div class="card" role="button" tabindex="0" onclick={() => (app.view = p.id)} onkeydown={(e) => e.key === 'Enter' && (app.view = p.id)}>
      <div class="art">
        {#if cover}
          <img src={srcOf(cover)} alt="" draggable="false" />
        {:else}
          <div class="ph">{p.kind === 'folder' ? '📁' : '♫'}</div>
        {/if}
        <button
          class="del edit"
          title="Изменить"
          onclick={(e) => {
            e.stopPropagation();
            app.editing = p.id;
          }}>✎</button
        >
        <button
          class="del"
          title="Удалить плейлист"
          onclick={(e) => {
            e.stopPropagation();
            void deletePlaylist(p);
          }}>✕</button
        >
      </div>
      <div class="name">{p.name}</div>
      <div class="sub">{p.kind === 'folder' ? 'Из папки' : 'Плейлист'} · {count} песен</div>
    </div>
  {:else}
    <p class="empty">Плейлистов пока нет. Создайте их кнопками «+ Плейлист» и «+ Из папки».</p>
  {/each}
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    padding: 18px 20px 10px;
  }
  h2 {
    margin: 0;
  }
  .meta {
    color: var(--muted);
    font-size: 12px;
  }
  .grid {
    flex: 1;
    overflow-y: auto;
    padding: 8px 20px 20px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 20px;
    align-content: start;
  }
  .card {
    padding: 12px;
    border-radius: 12px;
    background: var(--panel);
    cursor: pointer;
    transition: background 0.15s;
  }
  .card:hover {
    background: var(--panel-2);
  }
  .art {
    position: relative;
    aspect-ratio: 1;
    border-radius: 12px;
    overflow: hidden;
    background: linear-gradient(135deg, #3a2f7a, #1f6f8b);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.4);
  }
  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .ph {
    height: 100%;
    display: grid;
    place-items: center;
    font-size: 48px;
    opacity: 0.8;
  }
  .del {
    position: absolute;
    top: 6px;
    right: 6px;
    padding: 2px 7px;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.6);
    opacity: 0;
  }
  .del.edit {
    right: 38px;
  }
  .card:hover .del {
    opacity: 1;
  }
  .del:hover {
    color: var(--danger);
  }
  .name {
    margin-top: 10px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    color: var(--muted);
    font-size: 12px;
  }
  .empty {
    grid-column: 1 / -1;
    color: var(--muted);
  }
</style>
