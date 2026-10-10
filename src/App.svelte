<script lang="ts">
  import { onMount } from 'svelte';
  import { app, init } from './lib/store.svelte';
  import Sidebar from './lib/Sidebar.svelte';
  import TrackView from './lib/TrackView.svelte';
  import TopBar from './lib/TopBar.svelte';
  import PlaylistGrid from './lib/PlaylistGrid.svelte';
  import EditPlaylist from './lib/EditPlaylist.svelte';
  import CreateDialog from './lib/CreateDialog.svelte';
  import Settings from './lib/Settings.svelte';
  import PlayerBar from './lib/PlayerBar.svelte';

  onMount(init);
</script>

<div class="layout" class:detailed={app.mode === 'detailed'}>
  {#if app.mode === 'compact'}<Sidebar />{/if}
  <main>
    {#if app.mode === 'detailed'}<TopBar />{/if}
    {#if app.view === 'playlists'}<PlaylistGrid />{:else}<TrackView />{/if}
  </main>
  <PlayerBar />
</div>

{#if app.creating}
  <CreateDialog />
{/if}
{#if app.editing}
  {#key app.editing}<EditPlaylist />{/key}
{/if}
{#if app.settingsOpen}
  <Settings />
{/if}
{#if app.toast}
  <div class="toast">{app.toast}</div>
{/if}
{#if app.busy}
  <div class="busy">Обработка…</div>
{/if}

<style>
  .layout {
    display: grid;
    grid-template-columns: 260px 1fr;
    grid-template-rows: minmax(0, 1fr) auto;
    height: 100vh;
  }
  .layout.detailed {
    grid-template-columns: 1fr;
  }
  main {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .toast,
  .busy {
    position: fixed;
    right: 16px;
    padding: 8px 14px;
    border-radius: 8px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    font-size: 13px;
    z-index: 10;
  }
  .toast {
    bottom: 96px;
    max-width: 480px;
  }
  .busy {
    top: 16px;
    color: var(--accent);
  }
</style>
