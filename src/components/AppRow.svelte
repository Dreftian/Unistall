<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { InstalledApp } from '../types/app';
  import { appStore } from '../stores/appStore';

  export let app: InstalledApp;
  export const index: number = 0;

  const dispatch = createEventDispatcher();
  const { settings } = appStore;

  $: isCompact = $settings.isCompact;

  $: formattedSize = app.realSizeBytes
    ? app.realSizeBytes >= 1024 * 1024 * 1024
      ? (app.realSizeBytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
      : (app.realSizeBytes / (1024 * 1024)).toFixed(1) + ' MB'
    : app.estimatedSizeKb
    ? app.estimatedSizeKb >= 1024 * 1024
      ? (app.estimatedSizeKb / (1024 * 1024)).toFixed(2) + ' GB'
      : (app.estimatedSizeKb / 1024).toFixed(1) + ' MB'
    : '--';

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    dispatch('contextmenu', { event: e, app });
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  on:contextmenu={onContextMenu}
  class="{isCompact ? 'h-[42px]' : 'h-[52px]'} px-3 flex items-center justify-between border-b border-[var(--border-color)] hover:bg-[var(--bg-hover)] transition-colors select-none group"
>
  <!-- Columna izquierda: Icono Original, Advertencia Crítica y Nombre -->
  <div class="flex items-center gap-2.5 min-w-0 flex-1 pr-3">
    <div class="{isCompact ? 'w-6 h-6' : 'w-7 h-7'} rounded-md bg-[var(--bg-card)] flex items-center justify-center shrink-0 border border-[var(--border-color)] overflow-hidden shadow-xs relative">
      {#if app.iconBase64}
        <img src="data:image/png;base64,{app.iconBase64}" alt="" class="{isCompact ? 'w-4 h-4' : 'w-5 h-5'} object-contain" />
      {:else}
        <svg class="{isCompact ? 'w-3.5 h-3.5' : 'w-4 h-4'} text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect width="18" height="18" x="3" y="3" rx="2" /><path d="M9 3v18" /><path d="M14 9h7" /><path d="M14 15h7" />
        </svg>
      {/if}
    </div>

    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-1.5">
        {#if app.isCriticalSystem}
          <span
            class="text-[9px] px-1 py-0.2 rounded bg-amber-500/20 text-amber-400 font-bold flex items-center gap-1 border border-amber-500/30 shrink-0"
            title="Componente del Sistema. Desinstalar puede afectar el funcionamiento de Windows o de otros programas."
          >
            <svg class="w-2.5 h-2.5 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
            </svg>
            <span>SISTEMA</span>
          </span>
        {/if}
        <span class="text-xs font-semibold text-[var(--text-main)] truncate max-w-sm group-hover:text-[var(--accent-color)] transition-colors">
          {app.displayName}
        </span>
        {#if app.architecture}
          <span class="text-[9px] px-1 py-0.2 rounded bg-[var(--bg-card)] text-[var(--text-sub)] font-mono border border-[var(--border-color)]">
            {app.architecture}
          </span>
        {/if}
      </div>

      <div class="text-[10px] text-[var(--text-sub)] truncate flex items-center gap-2 mt-0.5">
        <span class="text-[var(--accent-color)]/90 font-medium">{app.category}</span>
        <span>•</span>
        <span class="truncate">{app.description}</span>
      </div>
    </div>
  </div>

  <!-- Columna central: Editor, Fecha y Tamaño -->
  <div class="flex items-center gap-4 shrink-0 text-xs text-[var(--text-sub)] pr-3">
    <div class="w-28 truncate text-[11px] text-right">
      {app.publisher || '--'}
    </div>
    <div class="w-20 text-right text-[11px]">
      {app.installDate || '--'}
    </div>
    <div class="w-16 text-right font-mono text-[11px] text-[var(--text-main)] font-medium">
      {formattedSize}
    </div>
  </div>

  <!-- Columna derecha: Botones de acción directa -->
  <div class="flex items-center gap-1 shrink-0 opacity-70 group-hover:opacity-100 transition-opacity">
    <button
      on:click={() => dispatch('hunt', app)}
      class="{isCompact ? 'px-2 py-0.5 text-[10px]' : 'px-2.5 py-1 text-xs'} rounded font-medium bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] text-[var(--text-main)] border border-[var(--border-color)] transition-colors flex items-center gap-1"
      title="Rastrear restos y claves huérfanas"
    >
      <svg class="w-3 h-3 text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
      </svg>
      <span>Restos</span>
    </button>

    <button
      on:click={() => dispatch('uninstall', app)}
      class="{isCompact ? 'px-2 py-0.5 text-[10px]' : 'px-2.5 py-1 text-xs'} rounded font-semibold bg-rose-500/15 hover:bg-rose-500/25 text-rose-500 border border-rose-500/30 transition-colors flex items-center gap-1"
      title="Desinstalar con comprobación completa"
    >
      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
      </svg>
      <span>Desinstalar</span>
    </button>

    <button
      on:click={(e) => dispatch('contextmenu', { event: e, app })}
      aria-label="Más opciones"
      class="p-1 rounded text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
      title="Más opciones"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="1" /><circle cx="12" cy="5" r="1" /><circle cx="12" cy="19" r="1" />
      </svg>
    </button>
  </div>
</div>
