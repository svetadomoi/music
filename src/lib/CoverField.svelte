<script lang="ts">
  import { pickImage } from './store.svelte';
  import { srcOf } from './api';
  import Cropper from './Cropper.svelte';
  import type { CoverPick } from './types';

  const SIZE = 130;

  /** undefined — без изменений, null — сбросить/нет обложки, иначе выбранная область */
  let {
    pick = $bindable(),
    existing = null,
    icon = '♫',
  }: { pick?: CoverPick | null; existing?: string | null; icon?: string } = $props();

  let cropping = $state<string | null>(null);

  const hasCover = $derived(pick ? true : pick === null ? false : !!existing);

  async function choose() {
    const f = await pickImage();
    if (f) cropping = f;
  }

  function bg(p: CoverPick) {
    const k = SIZE / p.crop.w;
    return `background-image:url("${srcOf(p.path)}");background-size:${p.nw * k}px ${p.nh * k}px;background-position:${-p.crop.x * k}px ${-p.crop.y * k}px`;
  }
</script>

<div class="row">
  <div class="art" style="width:{SIZE}px;height:{SIZE}px">
    {#if pick}
      <div class="crop" style={bg(pick)}></div>
    {:else if pick === undefined && existing}
      <img src={srcOf(existing)} alt="" />
    {:else}
      <div class="ph">{icon}</div>
    {/if}
  </div>
  <div class="col">
    <button type="button" onclick={choose}>Выбрать картинку…</button>
    <button type="button" onclick={() => (pick = null)} disabled={!hasCover}>Сбросить</button>
    <small>Без своей обложки берётся картинка из папки или тегов.</small>
  </div>
</div>

{#if cropping}
  <Cropper
    path={cropping}
    oncancel={() => (cropping = null)}
    onconfirm={(p) => {
      pick = p;
      cropping = null;
    }}
  />
{/if}

<style>
  .row {
    display: flex;
    gap: 14px;
  }
  .art {
    flex: none;
    border-radius: 12px;
    overflow: hidden;
    background: linear-gradient(135deg, #3a2f7a, #1f6f8b);
  }
  .art img,
  .crop {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    background-repeat: no-repeat;
  }
  .ph {
    height: 100%;
    display: grid;
    place-items: center;
    font-size: 44px;
    opacity: 0.8;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: flex-start;
  }
  small {
    color: var(--muted);
  }
</style>
