<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { appStore } from '../stores/appStore';
  import { initiateSoftwareScan, elevateUac, loadStartupList, scanOrphanedResiduals } from '../lib/tauriBridge';
  import type { FilterCategory } from '../types/app';

  const dispatch = createEventDispatcher();
  const { apps, startupEntries, orphanedItems, selectedCategory, isScanning, isScanningOrphans, settings } = appStore;

  function selectCategory(cat: FilterCategory) {
    appStore.selectedCategory.set(cat);
    if (cat === 'startup') {
      loadStartupList();
    } else if (cat === 'residuals' || cat === 'leftovers') {
      if ($orphanedItems.length === 0 && !$isScanningOrphans) {
        scanOrphanedResiduals();
      }
    }
  }

  function toggleTheme() {
    const next = $settings.theme === 'dark' ? 'light' : 'dark';
    appStore.updateSettings({ theme: next });
  }

  $: totalCount = $apps.length;
  $: win32Count = $apps.filter((a) => a.source === 'win32Registry').length;
  $: uwpCount = $apps.filter((a) => a.source === 'windowsStoreUwp').length;
  $: cliCount = $apps.filter((a) => ['scoop', 'chocolatey', 'winget'].includes(a.source)).length;
  $: startupCount = $startupEntries.length;
  $: residualsCount = $orphanedItems.length;
</script>

<aside class="w-60 h-full bg-[var(--bg-secondary)] border-r border-[var(--border-color)] flex flex-col justify-between p-2.5 select-none transition-colors">
  <!-- Cabecera / Marca -->
  <div>
    <div class="flex items-center justify-between px-2 py-2 mb-3">
      <div class="flex items-center gap-2">
        <div class="w-6 h-6 rounded-lg bg-gradient-to-tr from-[#0078d4] to-[#60cdff] flex items-center justify-center shadow-md shadow-cyan-500/20">
          <svg class="w-3.5 h-3.5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" />
          </svg>
        </div>
        <div>
          <h1 class="text-xs font-bold tracking-wide text-[var(--text-main)]">Unistall</h1>
          <p class="text-[9px] text-[var(--text-sub)] leading-none">v1.1.0 Fluent Pro</p>
        </div>
      </div>

      <!-- Botón de Tema Rápido -->
      <button
        on:click={toggleTheme}
        aria-label="Cambiar tema"
        class="p-1 rounded-md text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
        title="Alternar Modo Oscuro / Claro"
      >
        {#if $settings.theme === 'dark'}
          <svg class="w-3.5 h-3.5 text-yellow-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
          </svg>
        {:else}
          <svg class="w-3.5 h-3.5 text-blue-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
          </svg>
        {/if}
      </button>
    </div>

    <!-- Navegación por categorías -->
    <nav class="space-y-0.5 text-xs">
      <button
        on:click={() => selectCategory('all')}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md font-medium transition-colors {$selectedCategory === 'all' ? 'bg-[var(--bg-hover)] text-[var(--accent-color)] font-semibold' : 'text-[var(--text-main)] hover:bg-[var(--bg-hover)]/60'}"
      >
        <div class="flex items-center gap-2">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect width="7" height="7" x="3" y="3" rx="1" /><rect width="7" height="7" x="14" y="3" rx="1" /><rect width="7" height="7" x="14" y="14" rx="1" /><rect width="7" height="7" x="3" y="14" rx="1" />
          </svg>
          <span>Todas las apps</span>
        </div>
        <span class="text-[10px] px-1 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] border border-[var(--border-color)]">{totalCount}</span>
      </button>

      <button
        on:click={() => selectCategory('win32')}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md font-medium transition-colors {$selectedCategory === 'win32' ? 'bg-[var(--bg-hover)] text-[var(--accent-color)] font-semibold' : 'text-[var(--text-main)] hover:bg-[var(--bg-hover)]/60'}"
      >
        <div class="flex items-center gap-2">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect width="20" height="14" x="2" y="3" rx="2" /><line x1="8" x2="16" y1="21" y2="21" /><line x1="12" x2="12" y1="17" y2="21" />
          </svg>
          <span>Escritorio (Win32)</span>
        </div>
        <span class="text-[10px] px-1 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] border border-[var(--border-color)]">{win32Count}</span>
      </button>

      <button
        on:click={() => selectCategory('uwp')}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md font-medium transition-colors {$selectedCategory === 'uwp' ? 'bg-[var(--bg-hover)] text-[var(--accent-color)] font-semibold' : 'text-[var(--text-main)] hover:bg-[var(--bg-hover)]/60'}"
      >
        <div class="flex items-center gap-2">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4Z" /><line x1="3" x2="21" y1="6" y2="6" /><path d="M16 10a4 4 0 0 1-8 0" />
          </svg>
          <span>Windows Store</span>
        </div>
        <span class="text-[10px] px-1 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] border border-[var(--border-color)]">{uwpCount}</span>
      </button>

      <button
        on:click={() => selectCategory('cli')}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md font-medium transition-colors {$selectedCategory === 'cli' ? 'bg-[var(--bg-hover)] text-[var(--accent-color)] font-semibold' : 'text-[var(--text-main)] hover:bg-[var(--bg-hover)]/60'}"
      >
        <div class="flex items-center gap-2">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="4 17 10 11 4 5" /><line x1="12" x2="20" y1="19" y2="19" />
          </svg>
          <span>CLI (Scoop/Choco)</span>
        </div>
        <span class="text-[10px] px-1 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] border border-[var(--border-color)]">{cliCount}</span>
      </button>

      <button
        on:click={() => selectCategory('startup')}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md font-medium transition-colors {$selectedCategory === 'startup' ? 'bg-[var(--bg-hover)] text-[var(--accent-color)] font-semibold' : 'text-[var(--text-main)] hover:bg-[var(--bg-hover)]/60'}"
      >
        <div class="flex items-center gap-2">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
          </svg>
          <span>Inicio / Startup</span>
        </div>
        <span class="text-[10px] px-1 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] border border-[var(--border-color)]">{startupCount}</span>
      </button>

      <!-- Nueva categoría: Archivos Residuales de apps antiguas -->
      <button
        on:click={() => selectCategory('residuals')}
        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md font-medium transition-colors {$selectedCategory === 'residuals' || $selectedCategory === 'leftovers' ? 'bg-[var(--bg-hover)] text-amber-400 font-semibold' : 'text-[var(--text-main)] hover:bg-[var(--bg-hover)]/60'}"
      >
        <div class="flex items-center gap-2">
          <svg class="w-3.5 h-3.5 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            <line x1="10" x2="10" y1="11" y2="17" /><line x1="14" x2="14" y1="11" y2="17" />
          </svg>
          <span>Archivos Residuales</span>
        </div>
        {#if $isScanningOrphans}
          <div class="w-3 h-3 border-2 border-amber-400 border-t-transparent rounded-full animate-spin"></div>
        {:else if residualsCount > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded bg-amber-500/15 text-amber-400 font-semibold border border-amber-500/30">
            {residualsCount}
          </span>
        {:else}
          <span class="text-[9px] px-1 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] border border-[var(--border-color)]">
            Buscar
          </span>
        {/if}
      </button>
    </nav>
  </div>

  <!-- Acciones inferiores del sistema -->
  <div class="space-y-1.5 pt-2.5 border-t border-[var(--border-color)]">
    <!-- Configuración -->
    <button
      on:click={() => dispatch('settings')}
      class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-md text-xs font-medium text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
    >
      <svg class="w-3.5 h-3.5 text-[var(--text-sub)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
        <circle cx="12" cy="12" r="3" />
      </svg>
      <span>Configuración</span>
    </button>

    <!-- Escanear ahora -->
    <button
      on:click={() => initiateSoftwareScan()}
      disabled={$isScanning}
      class="w-full flex items-center justify-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-semibold bg-[var(--accent-color)] text-white hover:opacity-90 active:scale-98 transition-all shadow-xs disabled:opacity-50"
    >
      <svg class="w-3.5 h-3.5 {$isScanning ? 'animate-spin' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path d="M3 3v5h5" /><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" /><path d="M16 21h5v-5" />
      </svg>
      <span>{$isScanning ? 'Escaneando...' : 'Escanear'}</span>
    </button>

    <!-- Elevar UAC -->
    <button
      on:click={() => elevateUac()}
      class="w-full flex items-center justify-center gap-1.5 px-2 py-1 rounded-md text-[11px] font-medium bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] text-[var(--text-sub)] hover:text-[var(--text-main)] border border-[var(--border-color)] transition-colors"
      title="Reiniciar con privilegios SYSTEM/Administrador"
    >
      <svg class="w-3 h-3 text-amber-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
      </svg>
      <span>Elevar a UAC</span>
    </button>
  </div>
</aside>
