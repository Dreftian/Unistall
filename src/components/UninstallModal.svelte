<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { InstalledApp, DeepCleanResult } from '../types/app';
  import { executeFullDeepUninstall } from '../lib/tauriBridge';

  export let app: InstalledApp;
  export let waitForOfficial = true;

  const dispatch = createEventDispatcher();

  let currentStep: 'running_official' | 'hunting_leftovers' | 'cleaning' | 'finished' | 'error' = 'running_official';
  let result: DeepCleanResult | null = null;
  let errorMessage = '';

  onMount(async () => {
    try {
      currentStep = 'running_official';
      // Iniciar el proceso completo en backend
      const res = await executeFullDeepUninstall(app, waitForOfficial);
      result = res;
      currentStep = 'finished';
      dispatch('uninstalled', app);
    } catch (err: any) {
      console.error('Error durante la desinstalación profunda:', err);
      errorMessage = err?.toString() || 'Error desconocido';
      currentStep = 'error';
    }
  });

  function close() {
    dispatch('close');
  }

  function formatBytes(bytes: number) {
    if (bytes > 1024 * 1024 * 1024) return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
    if (bytes > 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    if (bytes > 1024) return (bytes / 1024).toFixed(0) + ' KB';
    return bytes + ' B';
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 select-none animate-in fade-in duration-150">
  <div class="w-full max-w-lg rounded-2xl shadow-2xl overflow-hidden flex flex-col fluent-glass">
    <!-- Cabecera -->
    <div class="px-5 py-4 border-b border-[var(--border-color)] flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-[var(--bg-hover)] flex items-center justify-center border border-[var(--border-color)] overflow-hidden shrink-0">
          {#if app.iconBase64}
            <img src="data:image/png;base64,{app.iconBase64}" alt="" class="w-6 h-6 object-contain" />
          {:else}
            <svg class="w-5 h-5 text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
            </svg>
          {/if}
        </div>
        <div class="min-w-0">
          <h2 class="text-sm font-semibold text-[var(--text-main)] truncate max-w-sm">Desinstalando {app.displayName}</h2>
          <p class="text-[11px] text-[var(--text-sub)]">Eliminación completa de archivos, registros y datos ocultos</p>
        </div>
      </div>
      {#if currentStep === 'finished' || currentStep === 'error'}
        <button on:click={close} aria-label="Cerrar" class="p-1.5 rounded-lg text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      {/if}
    </div>

    <!-- Cuerpo del estado -->
    <div class="p-6 space-y-4">
      {#if currentStep === 'running_official'}
        <div class="flex flex-col items-center justify-center py-6 gap-3">
          <div class="w-10 h-10 rounded-full border-3 border-[var(--accent-color)] border-t-transparent animate-spin"></div>
          <div class="text-center">
            <div class="text-sm font-medium text-[var(--text-main)]">Ejecutando desinstalación y rastreando restos...</div>
            <p class="text-xs text-[var(--text-sub)] mt-1">
              Esperando cierre del proceso oficial y analizando AppData, LocalLow, ProgramData y Registro...
            </p>
          </div>
        </div>
      {:else if currentStep === 'finished' && result}
        <div class="flex flex-col items-center justify-center py-4 gap-2 text-center">
          <div class="w-12 h-12 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center mb-1">
            <svg class="w-7 h-7" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <h3 class="text-base font-semibold text-[var(--text-main)]">¡Aplicación Desinstalada al 100%!</h3>
          <p class="text-xs text-[var(--text-sub)] max-w-sm">
            Se ha erradicado el software junto con todos sus archivos residuales, directorios ocultos y claves del registro.
          </p>

          <div class="grid grid-cols-2 gap-3 w-full mt-4 pt-3 border-t border-[var(--border-color)]">
            <div class="p-3 rounded-xl bg-[var(--bg-card)] border border-[var(--border-color)]">
              <div class="text-lg font-bold text-[var(--accent-color)]">{result.totalItemsDeleted}</div>
              <div class="text-[10px] uppercase font-semibold text-[var(--text-sub)]">Restos erradicados</div>
            </div>
            <div class="p-3 rounded-xl bg-[var(--bg-card)] border border-[var(--border-color)]">
              <div class="text-lg font-bold text-emerald-400">{formatBytes(result.totalBytesFreed)}</div>
              <div class="text-[10px] uppercase font-semibold text-[var(--text-sub)]">Espacio liberado</div>
            </div>
          </div>
        </div>
      {:else if currentStep === 'error'}
        <div class="flex flex-col items-center justify-center py-4 gap-2 text-center text-rose-400">
          <svg class="w-10 h-10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
          <div class="text-sm font-semibold text-[var(--text-main)]">Ocurrió un error en la desinstalación</div>
          <p class="text-xs text-rose-300 font-mono max-w-md break-all">{errorMessage}</p>
        </div>
      {/if}
    </div>

    <!-- Pie -->
    <div class="px-5 py-3 border-t border-[var(--border-color)] flex justify-end">
      {#if currentStep === 'finished' || currentStep === 'error'}
        <button
          on:click={close}
          class="px-5 py-1.5 rounded-lg text-xs font-semibold bg-[var(--accent-color)] text-white hover:opacity-90 transition-opacity"
        >
          Cerrar
        </button>
      {/if}
    </div>
  </div>
</div>
