<script lang="ts">
  let { value = $bindable(''), placeholder = 'Поиск…' }: { value?: string; placeholder?: string } = $props();
</script>

<span class="box">
  <input bind:value {placeholder} />
  {#if value}
    <button class="clear" onclick={() => (value = '')} title="Очистить" aria-label="Очистить">✕</button>
  {/if}
</span>

<style>
  .box {
    position: relative;
    display: inline-block;
    width: 220px;
    max-width: 100%;
  }
  input {
    width: 100%;
    padding-right: 28px;
  }
  .clear {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    border: none;
    background: none;
    padding: 2px 6px;
    color: var(--muted);
  }
  .clear:hover {
    color: var(--text);
  }
</style>
