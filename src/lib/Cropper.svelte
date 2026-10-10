<script lang="ts">
  import { srcOf } from './api';
  import type { CoverPick } from './types';

  let {
    path,
    onconfirm,
    oncancel,
  }: { path: string; onconfirm: (pick: CoverPick) => void; oncancel: () => void } = $props();

  const MIN = 24;

  let nw = $state(0);
  let nh = $state(0);
  let dw = $state(0);
  let dh = $state(0);
  let sel = $state({ x: 0, y: 0, size: 0 });
  let drag: { mode: 'move' | 'resize'; px: number; py: number; start: typeof sel } | null = null;

  function onload(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    nw = img.naturalWidth;
    nh = img.naturalHeight;
    dw = img.clientWidth;
    dh = img.clientHeight;
    const size = Math.min(dw, dh);
    sel = { x: (dw - size) / 2, y: (dh - size) / 2, size };
  }

  function down(e: PointerEvent, mode: 'move' | 'resize') {
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag = { mode, px: e.clientX, py: e.clientY, start: { ...sel } };
  }

  function move(e: PointerEvent) {
    if (!drag) return;
    const dx = e.clientX - drag.px;
    const dy = e.clientY - drag.py;
    const s = drag.start;
    if (drag.mode === 'move') {
      sel = {
        size: s.size,
        x: Math.min(Math.max(0, s.x + dx), dw - s.size),
        y: Math.min(Math.max(0, s.y + dy), dh - s.size),
      };
    } else {
      const max = Math.min(dw - s.x, dh - s.y);
      const size = Math.min(Math.max(MIN, s.size + Math.max(dx, dy)), max);
      sel = { ...s, size };
    }
  }

  const up = () => (drag = null);

  function confirm() {
    const k = nw / dw;
    onconfirm({
      path,
      nw,
      nh,
      crop: { x: sel.x * k, y: sel.y * k, w: sel.size * k, h: sel.size * k },
    });
  }
</script>

<svelte:window
  onkeydowncapture={(e) => {
    if (e.key === 'Escape') {
      e.stopImmediatePropagation();
      oncancel();
    }
  }}
/>

<div class="overlay" role="presentation">
  <div class="dialog" role="dialog" aria-modal="true">
    <h3>Выберите область обложки</h3>
    <div class="stage">
      <div class="wrap">
        <img src={srcOf(path)} alt="" draggable="false" onload={onload} />
        {#if dw}
          <div
            class="sel"
            role="presentation"
            style="left:{sel.x}px;top:{sel.y}px;width:{sel.size}px;height:{sel.size}px"
            onpointerdown={(e) => down(e, 'move')}
            onpointermove={move}
            onpointerup={up}
            onpointercancel={up}
          >
            <div
              class="handle"
              role="presentation"
              onpointerdown={(e) => down(e, 'resize')}
              onpointermove={move}
              onpointerup={up}
              onpointercancel={up}
            ></div>
          </div>
        {/if}
      </div>
    </div>
    <small>Перетаскивайте рамку, размер меняется за правый нижний угол.</small>
    <div class="end">
      <button type="button" onclick={oncancel}>Отмена</button>
      <button type="button" onclick={confirm} disabled={!dw}>Готово</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: grid;
    place-items: center;
    z-index: 40;
  }
  .dialog {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: 94vw;
  }
  h3 {
    margin: 0;
  }
  .stage {
    display: flex;
    justify-content: center;
  }
  .wrap {
    position: relative;
    line-height: 0;
    overflow: hidden;
    touch-action: none;
  }
  img {
    display: block;
    max-width: min(640px, 86vw);
    max-height: 60vh;
  }
  .sel {
    position: absolute;
    box-sizing: border-box;
    border: 2px solid #fff;
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.6);
    cursor: move;
    touch-action: none;
  }
  .handle {
    position: absolute;
    right: -7px;
    bottom: -7px;
    width: 14px;
    height: 14px;
    background: var(--accent);
    border: 2px solid #fff;
    border-radius: 3px;
    cursor: nwse-resize;
  }
  small {
    color: var(--muted);
  }
  .end {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
