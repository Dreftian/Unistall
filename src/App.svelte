<script lang="ts">
  import { onMount } from 'svelte';
  import TitleBar from './components/TitleBar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import VirtualList from './components/VirtualList.svelte';
  import AppRow from './components/AppRow.svelte';
  import ContextMenu from './components/ContextMenu.svelte';
  import LeftoverModal from './components/LeftoverModal.svelte';
  import SettingsModal from './components/SettingsModal.svelte';
  import UninstallWizardModal from './components/UninstallWizardModal.svelte';
  import ScanProgressBar from './components/ScanProgressBar.svelte';
  import ResidualsView from './components/ResidualsView.svelte';
  import { appStore } from './stores/appStore';
  import {
    initiateSoftwareScan,
    findLeftoversForApp,
    disableStartupItem,
  } from './lib/tauriBridge';
  import type { InstalledApp, LeftoverItem } from './types/app';

  const {
    filteredApps,
    startupEntries,
    selectedCategory,
    searchQuery,
    isScanning,
    settings,
  } = appStore;

  // Estado del menú contextual
  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuApp: InstalledApp | null = null;

  // Estado del modal de restos
  let leftoverModalOpen = false;
  let leftoverApp: InstalledApp | null = null;
  let leftoverItems: LeftoverItem[] = [];
  let isHunting = false;

  // Estado del asistente de desinstalación
  let wizardModalOpen = false;
  let wizardTargetApp: InstalledApp | null = null;

  // Estado del modal de configuración
  let settingsModalOpen = false;

  onMount(() => {
    initiateSoftwareScan();
  });

  function openContextMenu(e: CustomEvent<{ event: MouseEvent; app: InstalledApp }>) {
    contextMenuX = e.detail.event.clientX;
    contextMenuY = e.detail.event.clientY;
    contextMenuApp = e.detail.app;
    contextMenuVisible = true;
  }

  async function handleHunt(app: InstalledApp) {
    leftoverApp = app;
    leftoverModalOpen = true;
    isHunting = true;
    try {
      leftoverItems = await findLeftoversForApp(app);
    } catch (e) {
      console.error(e);
      leftoverItems = [];
    } finally {
      isHunting = false;
    }
  }

  function handleStartUninstall(app: InstalledApp) {
    wizardTargetApp = app;
    wizardModalOpen = true;
  }

  function handleMenuAction(e: CustomEvent<{ type: string; app: InstalledApp }>) {
    const { type, app } = e.detail;
    if (type === 'uninstall' || type === 'quietUninstall' || type === 'forceDelete') {
      handleStartUninstall(app);
    }
  }
</script>

<main class="w-screen h-screen flex flex-col bg-[var(--bg-primary)] text-[var(--text-main)] select-none font-sans overflow-hidden transition-colors">
  <!-- Barra de título Frameless nativa Windows 11 Fluent -->
  <TitleBar />

  <!-- Área Principal (Sidebar + Contenido) -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Barra lateral -->
    <Sidebar on:settings={() => (settingsModalOpen = true)} />

    <!-- Panel de aplicaciones -->
    <section class="flex-1 flex flex-col bg-[var(--bg-primary)] overflow-hidden transition-colors">
      <!-- Barra superior: Búsqueda y Filtros con Glass effect -->
      <header class="h-12 px-4 border-b border-[var(--border-color)] flex items-center justify-between gap-4 {$settings.glassEffect ? 'fluent-glass' : 'bg-[var(--bg-secondary)]'}">
        <!-- Campo de búsqueda instantáneo -->
        <div class="relative flex-1 max-w-sm">
          <svg class="w-3.5 h-3.5 text-[var(--text-sub)] absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
          </svg>
          <input
            type="text"
            bind:value={$searchQuery}
            placeholder="Buscar por nombre, editor o categoría..."
            class="w-full pl-8 pr-7 py-1 rounded-lg bg-[var(--bg-card)] border border-[var(--border-color)] hover:border-[var(--accent-color)]/50 focus:border-[var(--accent-color)] focus:outline-none text-xs text-[var(--text-main)] placeholder-[var(--text-sub)] transition-colors"
          />
          {#if $searchQuery}
            <button
              on:click={() => ($searchQuery = '')}
              aria-label="Limpiar búsqueda"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--text-sub)] hover:text-[var(--text-main)]"
            >
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          {/if}
        </div>

        <!-- Indicador de total y accesos rápidos -->
        <div class="flex items-center gap-2 text-xs">
          <span class="px-2 py-0.5 rounded bg-[var(--bg-card)] text-[var(--text-sub)] font-mono text-[11px] border border-[var(--border-color)]">
            {$filteredApps.length} aplicaciones
          </span>
          <button
            on:click={() => (settingsModalOpen = true)}
            aria-label="Configuración"
            class="p-1 rounded-md text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
            title="Configuración de Unistall"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>
        </div>
      </header>

      <!-- Barra de progreso real (%) durante el escaneo -->
      <ScanProgressBar />

      <!-- Encabezados de tabla si estamos en la vista de software -->
      {#if $selectedCategory !== 'startup' && $selectedCategory !== 'residuals' && $selectedCategory !== 'leftovers'}
        <div class="h-7 px-3 flex items-center justify-between text-[10px] font-bold text-[var(--text-sub)] uppercase tracking-wider border-b border-[var(--border-color)] bg-[var(--bg-secondary)]">
          <div class="flex-1">Aplicación / Categoría</div>
          <div class="flex items-center gap-4 pr-3">
            <div class="w-28 text-right">Editor</div>
            <div class="w-20 text-right">Instalado</div>
            <div class="w-16 text-right">Tamaño</div>
          </div>
          <div class="w-36 text-right pr-2">Acciones</div>
        </div>
      {/if}

      <!-- Vista de Residuales, Startup o Lista de Aplicaciones -->
      <div class="flex-1 overflow-hidden relative">
        {#if $selectedCategory === 'residuals' || $selectedCategory === 'leftovers'}
          <ResidualsView />
        {:else if $selectedCategory === 'startup'}
          <div class="h-full overflow-y-auto p-3 space-y-1.5">
            {#if $startupEntries.length === 0}
              <div class="flex flex-col items-center justify-center h-64 text-[var(--text-sub)] gap-2">
                <svg class="w-10 h-10 text-[var(--text-sub)]/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
                </svg>
                <span class="text-xs">No se encontraron entradas de inicio registradas en Run/RunOnce.</span>
              </div>
            {:else}
              {#each $startupEntries as entry}
                <div class="p-2.5 rounded-xl bg-[var(--bg-card)] border border-[var(--border-color)] flex items-center justify-between gap-3 hover:bg-[var(--bg-hover)] transition-colors">
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-semibold text-[var(--text-main)]">{entry.name}</span>
                      <span class="text-[9px] px-1.5 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] font-mono border border-[var(--border-color)]">{entry.location}</span>
                    </div>
                    <p class="text-[11px] text-[var(--text-sub)] font-mono truncate mt-0.5">{entry.command}</p>
                  </div>
                  <button
                    on:click={() => disableStartupItem(entry.id)}
                    class="px-2.5 py-1 rounded text-[11px] font-medium {entry.enabled ? 'bg-rose-500/15 hover:bg-rose-500/25 text-rose-500 border border-rose-500/30' : 'bg-[var(--bg-card)] text-[var(--text-sub)] border border-[var(--border-color)]'}"
                  >
                    {entry.enabled ? 'Deshabilitar' : 'Deshabilitado'}
                  </button>
                </div>
              {/each}
            {/if}
          </div>
        {:else}
          {#if $filteredApps.length === 0 && !$isScanning}
            <div class="flex flex-col items-center justify-center h-64 text-[var(--text-sub)] gap-2">
              <svg class="w-10 h-10 text-[var(--text-sub)]/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
              </svg>
              <span class="text-xs">No se encontraron aplicaciones con el filtro actual.</span>
            </div>
          {:else}
            <VirtualList items={$filteredApps} itemHeight={$settings.isCompact ? 42 : 52} let:item let:index>
              <AppRow
                app={item}
                {index}
                on:uninstall={(e) => handleStartUninstall(e.detail)}
                on:hunt={(e) => handleHunt(e.detail)}
                on:contextmenu={openContextMenu}
              />
            </VirtualList>
          {/if}
        {/if}
      </div>
    </section>
  </div>

  <!-- Barra de estado inferior -->
  <StatusBar />

  <!-- Menú Contextual flotante -->
  {#if contextMenuVisible && contextMenuApp}
    <ContextMenu
      x={contextMenuX}
      y={contextMenuY}
      app={contextMenuApp}
      on:close={() => (contextMenuVisible = false)}
      on:action={handleMenuAction}
    />
  {/if}

  <!-- Modal de Limpieza Heurística de Restos -->
  {#if leftoverModalOpen && leftoverApp}
    <LeftoverModal
      app={leftoverApp}
      items={leftoverItems}
      isLoading={isHunting}
      on:close={() => (leftoverModalOpen = false)}
      on:cleaned={() => {
        if (leftoverApp) {
          appStore.removeAppById(leftoverApp.id);
        }
      }}
    />
  {/if}

  <!-- Asistente de Desinstalación Profunda Total con Opciones y Progreso % Real -->
  {#if wizardModalOpen && wizardTargetApp}
    <UninstallWizardModal
      app={wizardTargetApp}
      on:close={() => (wizardModalOpen = false)}
      on:uninstalled={(e) => {
        appStore.removeAppById(e.detail.id);
        setTimeout(() => {
          initiateSoftwareScan();
        }, 500);
      }}
    />
  {/if}

  <!-- Modal de Configuración -->
  {#if settingsModalOpen}
    <SettingsModal on:close={() => (settingsModalOpen = false)} />
  {/if}
</main>
