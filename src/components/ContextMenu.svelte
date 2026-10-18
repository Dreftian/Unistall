<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { InstalledApp } from '../types/app';
  import { openFolder, openRegedit } from '../lib/tauriBridge';

  export let x = 0;
  export let y = 0;
  export let app: InstalledApp;

  const dispatch = createEventDispatcher();

  function close() {
    dispatch('close');
  }

  function handleAction(action: string) {
    close();
    switch (action) {
      case 'uninstall':
        dispatch('action', { type: 'uninstall', app });
        break;
      case 'quietUninstall':
        dispatch('action', { type: 'quietUninstall', app });
        break;
      case 'forceDelete':
        dispatch('action', { type: 'forceDelete', app });
        break;
      case 'openExplorer':
        if (app.installLocation) {
          openFolder(app.installLocation);
        }
        break;
      case 'openRegedit':
        openRegedit();
        break;
      case 'searchGoogle':
        window.open(`https://www.google.com/search?q=${encodeURIComponent(app.displayName)}`, '_blank');
        break;
    }
  }
</script>

<svelte:window on:click={close} on:keydown={(e) => e.key === 'Escape' && close()} />

<div
  style="top: {y}px; left: {x}px;"
  class="fixed z-50 w-56 rounded-xl py-1 text-xs select-none shadow-2xl animate-in fade-in zoom-in-95 duration-100 fluent-glass"
>
  <div class="px-3 py-1 text-[10px] text-[var(--text-sub)] font-semibold uppercase tracking-wider truncate border-b border-[var(--border-color)] mb-1">
    {app.displayName}
  </div>

  <button
    on:click={() => handleAction('uninstall')}
    class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-[var(--bg-hover)] text-[var(--text-main)] text-left transition-colors"
  >
    <svg class="w-3.5 h-3.5 text-rose-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
    </svg>
    <span>Desinstalación profunda</span>
  </button>

  {#if app.quietUninstallString}
    <button
      on:click={() => handleAction('quietUninstall')}
      class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-[var(--bg-hover)] text-[var(--accent-color)] text-left transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
      </svg>
      <span>Desinstalación silenciosa</span>
    </button>
  {/if}

  <button
    on:click={() => handleAction('forceDelete')}
    class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-rose-500/20 text-rose-500 text-left transition-colors"
  >
    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M18 6 6 18M6 6l12 12" />
    </svg>
    <span>Forzar erradicación total</span>
  </button>

  <div class="my-1 border-t border-[var(--border-color)]"></div>

  {#if app.installLocation}
    <button
      on:click={() => handleAction('openExplorer')}
      class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-[var(--bg-hover)] text-[var(--text-main)] text-left transition-colors"
    >
      <svg class="w-3.5 h-3.5 text-[var(--text-sub)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" />
      </svg>
      <span>Abrir en Explorador</span>
    </button>
  {/if}

  <button
    on:click={() => handleAction('openRegedit')}
    class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-[var(--bg-hover)] text-[var(--text-main)] text-left transition-colors"
  >
    <svg class="w-3.5 h-3.5 text-[var(--text-sub)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect width="18" height="18" x="3" y="3" rx="2" /><path d="M7 7h10M7 12h10M7 17h10" />
    </svg>
    <span>Abrir en Regedit</span>
  </button>

  <button
    on:click={() => handleAction('searchGoogle')}
    class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-[var(--bg-hover)] text-[var(--text-main)] text-left transition-colors"
  >
    <svg class="w-3.5 h-3.5 text-[var(--text-sub)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10" /><path d="m4.93 4.93 4.24 4.24M14.83 9.17l4.24-4.24M14.83 14.83l4.24 4.24M9.17 14.83l-4.24 4.24" />
    </svg>
    <span>Buscar en Google</span>
  </button>
</div>
