<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { InstalledApp, LeftoverItem } from '../types/app';
  import { deleteLeftoversBatch } from '../lib/tauriBridge';

  export let app: InstalledApp;
  export let items: LeftoverItem[] = [];
  export let isLoading = false;

  const dispatch = createEventDispatcher();
  let isDeleting = false;
  let successCount: number | null = null;

  $: selectedCount = items.filter((i) => i.isSelected).length;

  function toggleItem(index: number) {
    items[index].isSelected = !items[index].isSelected;
    items = [...items];
  }

  function toggleAll() {
    const allSelected = items.every((i) => i.isSelected);
    items = items.map((i) => ({ ...i, isSelected: !allSelected }));
  }

  async function handleDelete() {
    if (selectedCount === 0) return;
    isDeleting = true;
    try {
      const removed = await deleteLeftoversBatch(items);
      successCount = removed;
      setTimeout(() => {
        dispatch('cleaned');
        close();
      }, 1500);
    } catch (e) {
      console.error('Error eliminando restos:', e);
    } finally {
      isDeleting = false;
    }
  }

  function close() {
    dispatch('close');
  }

  function formatBytes(bytes?: number) {
    if (!bytes) return '--';
    if (bytes > 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
    if (bytes > 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return bytes + ' B';
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 animate-in fade-in duration-150 select-none">
  <div class="w-full max-w-2xl rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[82vh] fluent-glass">
    <!-- Cabecera del modal -->
    <div class="px-5 py-4 border-b border-[var(--border-color)] flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-[var(--bg-hover)] flex items-center justify-center border border-[var(--border-color)] overflow-hidden shrink-0">
          {#if app.iconBase64}
            <img src="data:image/png;base64,{app.iconBase64}" alt="" class="w-6 h-6 object-contain" />
          {:else}
            <svg class="w-5 h-5 text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
            </svg>
          {/if}
        </div>
        <div>
          <h2 class="text-sm font-semibold text-[var(--text-main)]">Cazador Heurístico de Restos</h2>
          <p class="text-xs text-[var(--text-sub)] truncate max-w-md">Análisis profundo para: <span class="text-[var(--text-main)] font-semibold">{app.displayName}</span></p>
        </div>
      </div>
      <button on:click={close} aria-label="Cerrar modal" class="p-1.5 rounded-lg text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors">
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <!-- Contenido del modal -->
    <div class="p-5 flex-1 overflow-y-auto min-h-[220px]">
      {#if isLoading}
        <div class="flex flex-col items-center justify-center h-48 gap-3">
          <div class="w-8 h-8 rounded-full border-2 border-[var(--accent-color)] border-t-transparent animate-spin"></div>
          <span class="text-xs text-[var(--text-sub)]">Escaneando AppData, LocalLow, ProgramData y claves de registro...</span>
        </div>
      {:else if successCount !== null}
        <div class="flex flex-col items-center justify-center h-48 gap-3 text-emerald-400">
          <svg class="w-12 h-12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          <span class="text-sm font-semibold text-[var(--text-main)]">¡Limpieza completada! {successCount} elementos eliminados de forma segura.</span>
        </div>
      {:else if items.length === 0}
        <div class="flex flex-col items-center justify-center h-48 gap-2 text-[var(--text-sub)]">
          <svg class="w-10 h-10 text-[var(--text-sub)]/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="10" /><path d="m9 12 2 2 4-4" />
          </svg>
          <span class="text-xs">No se detectaron archivos huérfanos ni claves residuales. Sistema limpio.</span>
        </div>
      {:else}
        <!-- Lista de restos -->
        <div class="flex items-center justify-between mb-3 text-xs text-[var(--text-sub)]">
          <span>Se detectaron <strong>{items.length}</strong> elementos residuales:</span>
          <button on:click={toggleAll} class="text-[var(--accent-color)] hover:underline text-[11px] font-medium">
            {items.every((i) => i.isSelected) ? 'Deseleccionar todos' : 'Seleccionar todos'}
          </button>
        </div>

        <div class="space-y-2">
          {#each items as item, index}
            <div
              role="button"
              tabindex="0"
              on:click={() => toggleItem(index)}
              on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleItem(index)}
              class="p-2.5 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between gap-3 cursor-pointer transition-colors {item.isSelected ? 'border-[var(--accent-color)]/50 bg-[var(--bg-hover)]' : ''}"
            >
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <input
                  type="checkbox"
                  checked={item.isSelected}
                  class="rounded bg-[var(--bg-card)] border-[var(--border-color)] text-[var(--accent-color)] focus:ring-0"
                />
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="text-[10px] font-semibold px-1.5 py-0.5 rounded uppercase font-mono {item.itemType === 'directory' ? 'bg-blue-500/15 text-blue-400' : 'bg-purple-500/15 text-purple-400'}">
                      {item.itemType}
                    </span>
                    <span class="text-xs text-[var(--text-main)] truncate font-mono">{item.pathOrKey}</span>
                  </div>
                  <p class="text-[11px] text-[var(--text-sub)] truncate mt-0.5">{item.rationale}</p>
                </div>
              </div>

              <div class="text-right shrink-0">
                <div class="text-[11px] font-semibold text-emerald-400">
                  {item.confidenceScore}% Confianza
                </div>
                <div class="text-[10px] text-[var(--text-sub)]">
                  {formatBytes(item.sizeBytes)}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Pie del modal -->
    <div class="px-5 py-3 border-t border-[var(--border-color)] flex items-center justify-between">
      <span class="text-xs text-[var(--text-sub)]">
        {#if items.length > 0 && successCount === null}
          {selectedCount} de {items.length} seleccionados
        {/if}
      </span>
      <div class="flex items-center gap-2">
        <button
          on:click={close}
          class="px-4 py-1.5 rounded-lg text-xs font-medium bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] text-[var(--text-main)] border border-[var(--border-color)] transition-colors"
        >
          Cerrar
        </button>
        {#if items.length > 0 && successCount === null}
          <button
            on:click={handleDelete}
            disabled={selectedCount === 0 || isDeleting}
            class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-rose-500 hover:bg-rose-600 text-white transition-all shadow-xs disabled:opacity-50 flex items-center gap-1.5"
          >
            {#if isDeleting}
              <div class="w-3 h-3 rounded-full border-2 border-white border-t-transparent animate-spin"></div>
              <span>Eliminando...</span>
            {:else}
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
              </svg>
              <span>Eliminar {selectedCount} restos</span>
            {/if}
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>
