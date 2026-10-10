<script lang="ts">
  import { app, addFolder, refresh, pickFolder } from './store.svelte';

  const inPlaylists = $derived(
    app.view === 'playlists' || app.data.playlists.some((p) => p.id === app.view),
  );

  let menu = $state(false);

  function add() {
    if (inPlaylists) menu = !menu;
    else void addFolder();
  }

  function createManual() {
    menu = false;
    app.creating = { kind: 'manual', path: null };
  }

  async function createFromFolder() {
    menu = false;
    const dir = await pickFolder();
    if (dir) app.creating = { kind: 'folder', path: dir };
  }

  function outside(e: PointerEvent) {
    if (menu && !(e.target as Element).closest('.add-wrap')) menu = false;
  }
</script>

<svelte:window onpointerdown={outside} onkeydown={(e) => e.key === 'Escape' && (menu = false)} />

<header>
  <div class="tabs">
    <button class="tab" class:active={app.view === 'library'} onclick={() => (app.view = 'library')}>
      ♫ Все песни <small>{app.data.tracks.length}</small>
    </button>
    <button class="tab" class:active={inPlaylists} onclick={() => (app.view = 'playlists')}>
      ☰ Плейлисты <small>{app.data.playlists.length}</small>
    </button>
  </div>

  <div class="tools">
    <div class="add-wrap">
      <button class="add" onclick={add} title={inPlaylists ? 'Создать плейлист' : 'Добавить папку с музыкой'}>+</button>
      {#if menu}
        <div class="menu">
          <button onclick={createManual}>Создать плейлист</button>
          <button onclick={createFromFolder}>Создать из папки</button>
        </div>
      {/if}
    </div>
    <button onclick={refresh} disabled={app.busy} title="Перечитать файлы из всех папок">⟳ Обновить</button>
    <button onclick={() => (app.settingsOpen = true)} title="Настройки">⚙</button>
  </div>
</header>

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    padding: 8px 16px 0;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tab {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 6px 6px 0 0;
    padding: 10px 16px;
  }
  .tab.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }
  small {
    color: var(--muted);
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-bottom: 6px;
  }
  .add-wrap {
    position: relative;
  }
  .add {
    width: 34px;
    font-size: 18px;
    line-height: 1;
    padding: 5px 0;
  }
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    z-index: 15;
    min-width: 190px;
    display: flex;
    flex-direction: column;
    padding: 4px;
    border-radius: 10px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  }
  .menu button {
    border: none;
    background: none;
    text-align: left;
  }
  .menu button:hover {
    background: var(--accent-bg);
  }
</style>
