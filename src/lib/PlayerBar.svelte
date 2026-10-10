<script lang="ts">
  import { fly } from 'svelte/transition';
  import { app, currentTrack, toggle, next, prev, seek, setVolume, cycleRepeat, toggleMute, fmt, loadCover, coverFor } from './store.svelte';
  import { srcOf } from './api';

  const track = $derived(currentTrack());
  let expanded = $state(false);

  const waves = $derived(app.volume === 0 ? 0 : app.volume < 0.34 ? 1 : app.volume < 0.67 ? 2 : 3);
  let volShown = $state(false);
  let volTimer: ReturnType<typeof setTimeout> | undefined;
  function showVolume() {
    volShown = true;
    clearTimeout(volTimer);
    volTimer = setTimeout(() => (volShown = false), 1200);
  }

  /** Обложка играющей песни: картинка в папке файла, затем встроенная, затем в папке-источнике. */
  const coverPaths = $derived(track ? [track.path, track.source] : []);
  const cover = $derived(coverPaths.length ? coverFor(coverPaths) : null);
  $effect(() => {
    if (coverPaths.length) void loadCover(coverPaths);
  });
  $effect(() => {
    if (!track) expanded = false;
  });

  function outside(e: PointerEvent) {
    if (expanded && !(e.target as Element).closest('.pop, .thumb')) expanded = false;
  }

  let dragging = $state<number | null>(null);
  const repeatTitle = $derived(
    app.repeat === 'off' ? 'Повтор выключен' : app.repeat === 'all' ? 'Повтор плейлиста' : 'Повтор песни',
  );
</script>

<svelte:window onpointerdown={outside} onkeydown={(e) => e.key === 'Escape' && (expanded = false)} />

<footer>
  <div class="info">
    <button class="thumb" onclick={() => track && (expanded = !expanded)} disabled={!track} title="Показать обложку" aria-label="Показать обложку">
      {#if cover}<img src={srcOf(cover)} alt="" draggable="false" />{:else}<span>♫</span>{/if}
    </button>
    <div class="text">
      {#if track}
        <div class="title">{track.title}</div>
        <div class="sub">{[track.artist, track.album].filter(Boolean).join(' — ')}</div>
      {:else}
        <div class="sub">Ничего не играет</div>
      {/if}
    </div>
  </div>

  {#if expanded && track}
    <div class="pop" transition:fly={{ y: 60, duration: 220 }}>
      <div class="big">
        {#if cover}<img src={srcOf(cover)} alt="" draggable="false" />{:else}<span>♫</span>{/if}
      </div>
      <div class="ptitle">{track.title}</div>
      <div class="psub">{[track.artist, track.album].filter(Boolean).join(' — ')}</div>
    </div>
  {/if}

  <div class="center">
    <div class="buttons">
      <button onclick={prev} disabled={!track} title="Назад">⏮</button>
      <button class="play" onclick={toggle} disabled={!track} title="Пауза / воспроизведение">
        {app.playing ? '⏸' : '▶'}
      </button>
      <button onclick={() => next()} disabled={!track} title="Вперёд">⏭</button>
      <button class="repeat" class:on={app.repeat !== 'off'} onclick={cycleRepeat} title={repeatTitle} aria-label={repeatTitle}>
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="17 1 21 5 17 9" />
          <path d="M3 11V9a4 4 0 0 1 4-4h14" />
          <polyline points="7 23 3 19 7 15" />
          <path d="M21 13v2a4 4 0 0 1-4 4H3" />
          {#if app.repeat === 'one'}
            <text x="12" y="15" text-anchor="middle" font-size="9" font-weight="700" fill="currentColor" stroke="none">1</text>
          {/if}
        </svg>
        <i class="dot"></i>
      </button>
    </div>
    <div class="seek">
      <span>{fmt(dragging ?? app.pos)}</span>
      <input
        type="range"
        min="0"
        max={app.dur || 0}
        step="0.1"
        value={dragging ?? app.pos}
        disabled={!track}
        oninput={(e) => (dragging = +e.currentTarget.value)}
        onchange={async (e) => {
          // держим ползунок на выбранной позиции, пока перемотка не завершится
          await seek(+e.currentTarget.value);
          dragging = null;
        }}
      />
      <span>{fmt(app.dur)}</span>
    </div>
  </div>

  <div class="vol">
    {#if volShown}
      <div class="badge" style="left: {30 + app.volume * 110}px">{Math.round(app.volume * 100)}%</div>
    {/if}
    <button
      class="vicon"
      onclick={() => {
        toggleMute();
        showVolume();
      }}
      title={app.volume === 0 ? 'Включить звук' : 'Выключить звук'}
      aria-label="Звук"
    >
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="10 5 5.5 9 2 9 2 15 5.5 15 10 19" />
        {#if waves === 0}
          <line x1="22" y1="9" x2="16" y2="15" />
          <line x1="16" y1="9" x2="22" y2="15" />
        {/if}
        {#if waves >= 1}<path d="M13.5 9.5a3.5 3.5 0 0 1 0 5" />{/if}
        {#if waves >= 2}<path d="M16 7a7.5 7.5 0 0 1 0 10" />{/if}
        {#if waves >= 3}<path d="M18.5 4.5a11 11 0 0 1 0 15" />{/if}
      </svg>
    </button>
    <input
      type="range"
      min="0"
      max="1"
      step="0.01"
      value={app.volume}
      oninput={(e) => {
        setVolume(+e.currentTarget.value);
        showVolume();
      }}
    />
  </div>
</footer>

<style>
  footer {
    position: relative;
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: 1fr 2fr 1fr;
    align-items: center;
    gap: 16px;
    padding: 10px 16px;
    background: var(--panel);
    border-top: 1px solid var(--border);
  }
  .info {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .text {
    min-width: 0;
    overflow: hidden;
  }
  .thumb {
    flex: none;
    width: 56px;
    height: 56px;
    padding: 0;
    overflow: hidden;
    border-radius: 8px;
    background: linear-gradient(135deg, #3a2f7a, #1f6f8b);
    display: grid;
    place-items: center;
    font-size: 22px;
  }
  .thumb img,
  .big img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .pop {
    position: absolute;
    left: 16px;
    bottom: calc(100% + 12px);
    width: min(360px, calc(100vw - 32px));
    padding: 14px;
    border-radius: 16px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
    z-index: 15;
  }
  .big {
    aspect-ratio: 1;
    border-radius: 12px;
    overflow: hidden;
    background: linear-gradient(135deg, #3a2f7a, #1f6f8b);
    display: grid;
    place-items: center;
    font-size: 96px;
    opacity: 1;
  }
  .ptitle {
    margin-top: 12px;
    font-weight: 700;
    font-size: 16px;
  }
  .psub {
    color: var(--muted);
    font-size: 13px;
  }
  .title {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    color: var(--muted);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .center {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .buttons {
    display: flex;
    justify-content: center;
    gap: 8px;
  }
  .repeat {
    position: relative;
    display: grid;
    place-items: center;
    background: none;
    border: none;
    color: var(--muted);
  }
  .repeat .dot {
    position: absolute;
    bottom: 1px;
    left: 50%;
    width: 4px;
    height: 4px;
    margin-left: -2px;
    border-radius: 50%;
    background: currentColor;
    opacity: 0;
  }
  .repeat.on .dot {
    opacity: 1;
  }
  .buttons button.on {
    color: var(--accent);
  }
  .play {
    min-width: 44px;
  }
  .seek {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--muted);
  }
  .seek input {
    flex: 1;
  }
  .vicon {
    background: none;
    border: none;
    padding: 2px;
    color: var(--muted);
    display: grid;
  }
  .vicon:hover {
    color: var(--text);
  }
  .badge {
    position: absolute;
    bottom: calc(100% + 6px);
    transform: translateX(-50%);
    padding: 2px 8px;
    border-radius: 6px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    pointer-events: none;
  }
  .vol {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    justify-content: flex-end;
  }
  .vol input {
    width: 110px;
  }
</style>
