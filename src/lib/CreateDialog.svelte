<script lang="ts">
  import { app, createPlaylist, createFolderPlaylist, pickFolder } from './store.svelte';
  import CoverField from './CoverField.svelte';
  import type { CoverPick } from './types';

  const start = app.creating;
  const folderName = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? '';

  let folder = $state<string | null>(start?.path ?? null);
  let name = $state(start?.path ? folderName(start.path) : '');
  let pick = $state<CoverPick | null | undefined>(undefined);

  const isFolder = start?.kind === 'folder';
  const close = () => (app.creating = null);

  async function changeFolder() {
    const dir = await pickFolder();
    if (!dir) return;
    if (!name.trim() || name === folderName(folder ?? '')) name = folderName(dir);
    folder = dir;
  }

  async function save() {
    const n = name.trim();
    if (!n || (isFolder && !folder)) return;
    const cover = pick ?? null;
    const ok = isFolder && folder ? await createFolderPlaylist(n, folder, cover) : await createPlaylist(n, cover);
    if (ok) {
      const last = app.data.playlists[app.data.playlists.length - 1];
      if (last) app.view = last.id;
      close();
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

<div class="overlay" role="presentation" onclick={close}>
  <form
    class="dialog"
    onclick={(e) => e.stopPropagation()}
    onsubmit={(e) => {
      e.preventDefault();
      void save();
    }}
  >
    <h3>{isFolder ? 'Новый плейлист из папки' : 'Новый плейлист'}</h3>

    {#if isFolder}
      <div class="label">Папка</div>
      <div class="folder">
        <span class="path" title={folder ?? ''}>{folder ?? 'не выбрана'}</span>
        <button type="button" onclick={changeFolder}>Изменить…</button>
      </div>
    {/if}

    <label>
      <span class="label">Название</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input bind:value={name} placeholder="Название плейлиста" autofocus />
    </label>

    <div class="label">Обложка</div>
    <CoverField bind:pick icon={isFolder ? '📁' : '♫'} />

    <div class="end">
      <button type="button" onclick={close}>Отмена</button>
      <button type="submit" disabled={app.busy || !name.trim() || (isFolder && !folder)}>Создать</button>
    </div>
  </form>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: grid;
    place-items: center;
    z-index: 20;
  }
  .dialog {
    width: 440px;
    max-width: 90vw;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  h3 {
    margin: 0;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .label {
    color: var(--muted);
    font-size: 12px;
    text-transform: uppercase;
  }
  .folder {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
  }
  .end {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
