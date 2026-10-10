<script lang="ts">
  import { loadCover, coverFor } from './store.svelte';
  import { srcOf } from './api';
  import type { Track } from './types';

  let { track }: { track: Track } = $props();

  let visible = $state(false);
  const paths = $derived([track.path, track.source]);
  const cover = $derived(visible ? coverFor(paths) : null);

  // обложки грузятся лениво — только для строк, попавших на экран
  function watch(node: HTMLElement) {
    const io = new IntersectionObserver((entries) => {
      if (entries.some((e) => e.isIntersecting)) {
        visible = true;
        io.disconnect();
      }
    });
    io.observe(node);
    return { destroy: () => io.disconnect() };
  }

  $effect(() => {
    if (visible) void loadCover(paths);
  });
</script>

<span class="cover" use:watch>
  {#if cover}<img src={srcOf(cover)} alt="" loading="lazy" draggable="false" />{:else}<i>♫</i>{/if}
</span>

<style>
  .cover {
    width: 36px;
    height: 36px;
    border-radius: 6px;
    overflow: hidden;
    background: linear-gradient(135deg, #3a2f7a, #1f6f8b);
    display: grid;
    place-items: center;
    font-size: 14px;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  i {
    font-style: normal;
    opacity: 0.7;
  }
</style>
