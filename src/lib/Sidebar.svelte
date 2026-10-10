<script lang="ts">
  import { app, addFolder, refresh, deletePlaylist } from './store.svelte';
  import SearchBox from './SearchBox.svelte';
  import CreatePlaylist from './CreatePlaylist.svelte';

  const shown = $derived(
    app.data.playlists.filter((p) => p.name.toLowerCase().includes(app.playlistQuery.trim().toLowerCase())),
  );
</script>

<aside>
  <h1>Nega Player</h1>

  <div class="actions">
    <button onclick={addFolder}>+ Папка</button>
    <button onclick={refresh} disabled={app.busy} title="Перечитать файлы из всех папок">⟳ Обновить</button>
  </div>

  <nav>
    <button class="item" class:active={app.view === 'library'} onclick={() => (app.view = 'library')}>
      <span>♫ Все песни</span>
      <small>{app.data.tracks.length}</small>
    </button>

    <div class="section">Плейлисты</div>
    <SearchBox bind:value={app.playlistQuery} placeholder="Поиск плейлистов…" />
    <div class="playlists">
      {#each shown as p (p.id)}
        <div class="row">
          <button class="item" class:active={app.view === p.id} onclick={() => (app.view = p.id)}>
            <span>{p.kind === 'folder' ? '📁' : '☰'} {p.name}</span>
          </button>
          <button class="x" title="Изменить" onclick={() => (app.editing = p.id)}>✎</button>
          <button class="x" title="Удалить плейлист" onclick={() => deletePlaylist(p)}>✕</button>
        </div>
      {:else}
        <div class="none">{app.playlistQuery ? 'Ничего не найдено' : 'Плейлистов нет'}</div>
      {/each}
    </div>
  </nav>

  <CreatePlaylist />
  <button onclick={() => (app.settingsOpen = true)}>⚙ Настройки</button>
</aside>

<style>
  aside {
    background: var(--panel);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 14px 10px;
    gap: 12px;
    overflow: hidden;
  }
  h1 {
    font-size: 16px;
    margin: 0 6px;
  }
  /* «Все песни» и плейлисты — один контейнер */
  nav {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px;
    border-radius: 10px;
    background: var(--panel-2);
    border: 1px solid var(--border);
  }
  .playlists {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .actions button {
    flex: 1;
  }
  .section {
    margin: 12px 6px 4px;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--muted);
  }
  .row {
    display: flex;
    align-items: center;
  }
  .item {
    flex: 1;
    display: flex;
    justify-content: space-between;
    background: none;
    border: none;
    text-align: left;
    overflow: hidden;
    white-space: nowrap;
  }
  nav > .item {
    flex: none;
  }
  .item span {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .item.active {
    background: var(--accent-bg);
    color: var(--accent);
  }
  .x {
    background: none;
    border: none;
    color: var(--muted);
    padding: 4px 8px;
  }
  .x:hover {
    color: var(--danger);
  }
  .none {
    color: var(--muted);
    font-size: 12px;
    padding: 6px;
  }
  nav :global(.box) {
    width: auto;
    margin: 0 2px;
  }
  small {
    color: var(--muted);
  }
</style>
