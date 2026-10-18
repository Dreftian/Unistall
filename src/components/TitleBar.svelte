<script lang="ts">
  import { onMount } from 'svelte';
  import {
    minimizeWindow,
    toggleMaximizeWindow,
    closeWindow,
    isSystemElevated,
    elevateUac,
  } from '../lib/tauriBridge';

  let isElevated = false;
  let isMaximized = false;

  onMount(async () => {
    try {
      isElevated = await isSystemElevated();
    } catch (e) {
      console.error(e);
    }
  });

  async function handleMinimize() {
    await minimizeWindow();
  }

  async function handleMaximize() {
    isMaximized = !isMaximized;
    await toggleMaximizeWindow();
  }

  async function handleClose() {
    await closeWindow();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header
  data-tauri-drag-region
  on:dblclick={handleMaximize}
  class="h-9 px-3 flex items-center justify-between border-b border-[var(--border-color)] bg-[var(--bg-secondary)] select-none shrink-0 z-50 text-xs transition-colors"
>
  <!-- Lado izquierdo: Identidad de Unistall y Privilegios UAC -->
  <div data-tauri-drag-region class="flex items-center gap-2.5 pointer-events-none">
    <div class="w-4.5 h-4.5 rounded-md bg-gradient-to-br from-red-500 to-rose-700 flex items-center justify-center text-white shadow-xs">
      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
      </svg>
    </div>

    <div class="flex items-center gap-2">
      <span class="font-bold text-[12px] tracking-wide text-[var(--text-main)]">Unistall</span>
      <span class="text-[10px] text-[var(--text-sub)] hidden sm:inline">Gestor & Limpiador Profundo</span>
      
      {#if isElevated}
        <span class="text-[9px] px-1.5 py-0.2 rounded-full font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
          <svg class="w-2.5 h-2.5 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
          </svg>
          ADMIN
        </span>
      {:else}
        <button
          type="button"
          on:click={elevateUac}
          class="pointer-events-auto text-[9px] px-1.5 py-0.2 rounded-full font-medium bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 border border-amber-500/30 transition-colors"
          title="Haz clic para elevar privilegios con UAC"
        >
          Usuario Estándar (Elevar)
        </button>
      {/if}
    </div>
  </div>

  <!-- Zona central arrastrable -->
  <div data-tauri-drag-region class="flex-1 h-full cursor-default"></div>

  <!-- Lado derecho: Botones de control de ventana estilo Windows 11 Fluent -->
  <div class="flex items-center h-full -mr-3">
    <!-- Minimizar -->
    <button
      type="button"
      on:click={handleMinimize}
      class="w-11 h-9 flex items-center justify-center text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
      title="Minimizar"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M5 12h14" />
      </svg>
    </button>

    <!-- Maximizar / Restaurar -->
    <button
      type="button"
      on:click={handleMaximize}
      class="w-11 h-9 flex items-center justify-center text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
      title={isMaximized ? 'Restaurar' : 'Maximizar'}
    >
      {#if isMaximized}
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <rect width="11" height="11" x="7" y="3" rx="1.5" />
          <path d="M5 7H4a1.5 1.5 0 0 0-1.5 1.5v9A1.5 1.5 0 0 0 4 19h9a1.5 1.5 0 0 0 1.5-1.5v-1" />
        </svg>
      {:else}
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect width="14" height="14" x="5" y="5" rx="1.5" />
        </svg>
      {/if}
    </button>

    <!-- Cerrar -->
    <button
      type="button"
      on:click={handleClose}
      class="w-11 h-9 flex items-center justify-center text-[var(--text-sub)] hover:text-white hover:bg-[#c42b1c] active:bg-[#a82518] transition-colors"
      title="Cerrar"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6 6 18M6 6l12 12" />
      </svg>
    </button>
  </div>
</header>
