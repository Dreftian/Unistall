<script lang="ts">
  import { onMount } from 'svelte';

  export let items: any[] = [];
  export let itemHeight = 52;
  export let buffer = 8;

  let viewport: HTMLDivElement;
  let scrollTop = 0;
  let viewportHeight = 600;

  $: totalHeight = items.length * itemHeight;

  $: startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - buffer);
  $: endIndex = Math.min(
    items.length,
    Math.ceil((scrollTop + viewportHeight) / itemHeight) + buffer
  );

  $: visibleItems = items.slice(startIndex, endIndex).map((item, index) => ({
    item,
    virtualIndex: startIndex + index,
  }));

  $: topPadding = startIndex * itemHeight;
  $: bottomPadding = Math.max(0, (items.length - endIndex) * itemHeight);

  function handleScroll() {
    if (viewport) {
      scrollTop = viewport.scrollTop;
    }
  }

  function handleResize() {
    if (viewport) {
      viewportHeight = viewport.clientHeight;
    }
  }

  onMount(() => {
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  });
</script>

<div
  bind:this={viewport}
  on:scroll={handleScroll}
  class="w-full h-full overflow-y-auto overflow-x-hidden relative"
>
  <div style="height: {totalHeight}px; position: relative;">
    <div style="transform: translateY({topPadding}px); will-change: transform;">
      {#each visibleItems as { item, virtualIndex } (item.id || virtualIndex)}
        <div style="height: {itemHeight}px;">
          <slot {item} index={virtualIndex} />
        </div>
      {/each}
    </div>
  </div>
</div>
