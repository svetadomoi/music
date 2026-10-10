<script lang="ts">
  import { app, pickFolder } from './store.svelte';

  function startManual() {
    app.creating = { kind: 'manual', path: null };
  }

  async function startFolder() {
    const dir = await pickFolder();
    if (dir) app.creating = { kind: 'folder', path: dir };
  }
</script>

<div class="actions">
  <button onclick={startManual}>+ Плейлист</button>
  <button onclick={startFolder}>+ Из папки</button>
</div>

<style>
  .actions {
    display: flex;
    gap: 6px;
  }
  .actions button {
    flex: 1;
    white-space: nowrap;
  }
</style>
