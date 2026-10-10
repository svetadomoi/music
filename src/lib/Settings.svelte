<script lang="ts">
  import { app, setMode, setTheme, themes, setColumn, columnList } from './store.svelte';
  import type { ViewMode } from './store.svelte';

  const modes: { id: ViewMode; title: string; text: string }[] = [
    { id: 'compact', title: 'Компактный', text: 'Плейлисты списком в левом меню.' },
    { id: 'detailed', title: 'Подробный', text: 'Вкладки «Все песни» и «Плейлисты» сверху; плейлисты — карточки с обложкой.' },
  ];

  const close = () => (app.settingsOpen = false);
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

<div class="overlay" role="presentation" onclick={close}>
  <div class="dialog" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
    <h3>Настройки</h3>
    <div class="label">Отображение</div>
    {#each modes as m (m.id)}
      <label class="mode" class:active={app.mode === m.id}>
        <input type="radio" name="mode" checked={app.mode === m.id} onchange={() => setMode(m.id)} />
        <span>
          <b>{m.title}</b>
          <small>{m.text}</small>
        </span>
      </label>
    {/each}
    <div class="label cols-label">Цветовая схема</div>
    <div class="themes">
      {#each themes as t (t.id)}
        <button
          type="button"
          class="theme"
          class:active={app.theme === t.id}
          data-theme={t.id}
          onclick={() => setTheme(t.id)}
        >
          <span class="prev">
            <i class="p1"></i>
            <i class="p2"></i>
            <i class="p3"></i>
          </span>
          {t.label}
        </button>
      {/each}
    </div>
    <div class="label cols-label">Столбцы в списке песен</div>
    <div class="cols">
      <label class="col"><input type="checkbox" checked disabled /> Название</label>
      {#each columnList as c (c.key)}
        <label class="col">
          <input type="checkbox" checked={app.columns[c.key]} onchange={(e) => setColumn(c.key, e.currentTarget.checked)} />
          {c.label}
        </label>
      {/each}
    </div>
    <div class="end"><button onclick={close}>Закрыть</button></div>
  </div>
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
    width: 420px;
    max-width: 90vw;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  h3 {
    margin: 0 0 4px;
  }
  .label {
    color: var(--muted);
    font-size: 12px;
    text-transform: uppercase;
  }
  .mode {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
  }
  .mode.active {
    border-color: var(--accent);
    background: var(--accent-bg);
  }
  .mode span {
    display: flex;
    flex-direction: column;
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .theme {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: stretch;
    padding: 6px;
    background: var(--bg);
    color: var(--text);
    border: 2px solid var(--border);
    font-size: 12px;
  }
  .theme.active {
    border-color: var(--accent);
  }
  .prev {
    display: flex;
    gap: 3px;
    height: 26px;
    padding: 3px;
    border-radius: 5px;
    background: var(--panel);
  }
  .prev i {
    flex: 1;
    border-radius: 3px;
  }
  .p1 {
    background: var(--panel-2);
  }
  .p2 {
    background: var(--accent);
  }
  .p3 {
    background: var(--muted);
  }
  .cols-label {
    margin-top: 8px;
  }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 12px;
  }
  .col {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  small {
    color: var(--muted);
  }
  .end {
    display: flex;
    justify-content: flex-end;
    margin-top: 6px;
  }
</style>
