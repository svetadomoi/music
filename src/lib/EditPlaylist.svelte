<script lang="ts">
  import { app, savePlaylistEdit } from './store.svelte';
  import CoverField from './CoverField.svelte';
  import type { CoverPick } from './types';

  const playlist = $derived(app.data.playlists.find((p) => p.id === app.editing));

  let name = $state(app.data.playlists.find((p) => p.id === app.editing)?.name ?? '');
  /** undefined — без изменений, null — сбросить, иначе выбранная область */
  let pick = $state<CoverPick | null | undefined>(undefined);

  const close = () => (app.editing = null);

  async function save() {
    if (playlist && (await savePlaylistEdit(playlist, name, pick))) close();
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

{#if playlist}
  <div class="overlay" role="presentation" onclick={close}>
    <form
      class="dialog"
      onclick={(e) => e.stopPropagation()}
      onsubmit={(e) => {
        e.preventDefault();
        void save();
      }}
    >
      <h3>Редактирование плейлиста</h3>

      <label>
        <span class="label">Название</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input bind:value={name} autofocus />
      </label>

      <div class="label">Обложка</div>
      <CoverField bind:pick existing={playlist.cover} icon={playlist.kind === 'folder' ? '📁' : '♫'} />

      <div class="end">
        <button type="button" onclick={close}>Отмена</button>
        <button type="submit" disabled={app.busy || !name.trim()}>Сохранить</button>
      </div>
    </form>
  </div>
{/if}

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
  .end {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
