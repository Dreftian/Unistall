<script lang="ts">
  import { appStore } from '../stores/appStore';
  import {
    scanOrphanedResiduals,
    streamCleanOrphanedResiduals,
    openFolder,
    rebootSystemNow,
    getResidualFileDetails,
  } from '../lib/tauriBridge';
  import type { OrphanedItem, DeepCleanResult, ResidualFileDetail } from '../types/app';

  const { orphanedItems, isScanningOrphans } = appStore;

  // Estado de limpieza y streaming en tiempo real
  let isCleaning = false;
  let cleanProgress = {
    percentage: 0,
    currentItem: '',
    deletedCount: 0,
    totalCount: 0,
    bytesFreed: 0,
  };
  let cleanResult: DeepCleanResult | null = null;
  let showResultModal = false;
  let showConfirmModal = false;

  // Filtros
  let activeFilter: 'all' | 'safe' | 'game_saves' | 'cache' | 'config' = 'all';
  let expandedItemId: string | null = null;

  // Estado del modal de inspección detallada de archivos
  let showFilesModal = false;
  let inspectingItem: OrphanedItem | null = null;
  let inspectingFiles: ResidualFileDetail[] = [];
  let isLoadingFiles = false;
  let fileSearchQuery = '';

  $: filteredItems = $orphanedItems.filter((item) => {
    if (activeFilter === 'safe') return item.recommendationLevel === 'highly_recommended';
    if (activeFilter === 'game_saves') return item.isGameSave === true;
    if (activeFilter === 'cache') return item.itemType.includes('local') || item.itemType === 'locallow';
    if (activeFilter === 'config') return item.itemType === 'appdata_roaming' || item.itemType === 'programdata';
    return true;
  });

  $: selectedItems = filteredItems.filter((item) => item.isSelected);
  $: totalSize = $orphanedItems.reduce((acc, item) => acc + item.sizeBytes, 0);
  $: selectedSize = selectedItems.reduce((acc, item) => acc + item.sizeBytes, 0);
  $: safeCount = $orphanedItems.filter((i) => i.recommendationLevel === 'highly_recommended').length;
  $: gameSaveCount = $orphanedItems.filter((i) => i.isGameSave === true).length;

  $: filteredInspectFiles = inspectingFiles.filter((f) => {
    if (!fileSearchQuery.trim()) return true;
    const q = fileSearchQuery.toLowerCase();
    return f.name.toLowerCase().includes(q) || f.relativePath.toLowerCase().includes(q);
  });

  function toggleSelectAll() {
    const allSelected = selectedItems.length === filteredItems.length;
    const targetIds = new Set(filteredItems.map((i) => i.id));
    appStore.orphanedItems.update((items) =>
      items.map((i) => (targetIds.has(i.id) ? { ...i, isSelected: !allSelected } : i))
    );
  }

  function toggleItem(id: string) {
    appStore.orphanedItems.update((items) =>
      items.map((i) => (i.id === id ? { ...i, isSelected: !i.isSelected } : i))
    );
  }

  function toggleExpand(id: string) {
    expandedItemId = expandedItemId === id ? null : id;
  }

  async function handleScan() {
    await scanOrphanedResiduals();
  }

  // Inspección detallada de archivos dentro de una carpeta
  async function handleInspectFiles(item: OrphanedItem) {
    inspectingItem = item;
    showFilesModal = true;
    isLoadingFiles = true;
    fileSearchQuery = '';
    try {
      inspectingFiles = await getResidualFileDetails(item.path);
    } catch (e) {
      console.error('Error al obtener lista de archivos:', e);
      inspectingFiles = [];
    } finally {
      isLoadingFiles = false;
    }
  }

  // Limpieza en Streaming con barra de progreso en tiempo real
  async function handleCleanSelected() {
    if (selectedItems.length === 0) return;
    showConfirmModal = false;
    isCleaning = true;
    cleanProgress = {
      percentage: 0,
      currentItem: 'Iniciando limpieza profunda...',
      deletedCount: 0,
      totalCount: selectedItems.length,
      bytesFreed: 0,
    };

    try {
      await streamCleanOrphanedResiduals(selectedItems, (update) => {
        if (update.event === 'progress') {
          cleanProgress = {
            percentage: update.data.percentage,
            currentItem: update.data.currentItem,
            deletedCount: update.data.deletedCount,
            totalCount: update.data.totalCount,
            bytesFreed: update.data.bytesFreed,
          };
        } else if (update.event === 'finished') {
          cleanResult = update.data.result;
          const deletedPaths = new Set(cleanResult.details.map((d) => d.replace('Eliminado: ', '')));
          appStore.orphanedItems.update((current) => current.filter((item) => !deletedPaths.has(item.path)));
          setTimeout(() => {
            isCleaning = false;
            showResultModal = true;
          }, 500);
        } else if (update.event === 'error') {
          console.error(update.data.message);
          isCleaning = false;
        }
      });
    } catch (err) {
      console.error(err);
      isCleaning = false;
    }
  }

  async function handleCleanSingle(item: OrphanedItem) {
    isCleaning = true;
    cleanProgress = {
      percentage: 10,
      currentItem: `Eliminando ${item.path}...`,
      deletedCount: 0,
      totalCount: 1,
      bytesFreed: 0,
    };

    try {
      await streamCleanOrphanedResiduals([item], (update) => {
        if (update.event === 'progress') {
          cleanProgress = {
            percentage: update.data.percentage,
            currentItem: update.data.currentItem,
            deletedCount: update.data.deletedCount,
            totalCount: update.data.totalCount,
            bytesFreed: update.data.bytesFreed,
          };
        } else if (update.event === 'finished') {
          cleanResult = update.data.result;
          const deletedPaths = new Set(cleanResult.details.map((d) => d.replace('Eliminado: ', '')));
          appStore.orphanedItems.update((current) => current.filter((i) => !deletedPaths.has(i.path)));
          setTimeout(() => {
            isCleaning = false;
            showResultModal = true;
          }, 500);
        }
      });
    } catch (err) {
      console.error(err);
      isCleaning = false;
    }
  }

  async function handleRebootNow() {
    try {
      await rebootSystemNow();
    } catch (e) {
      console.error(e);
    }
  }

  function formatBytes(bytes: number) {
    if (bytes >= 1024 * 1024 * 1024) return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
    if (bytes >= 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    if (bytes >= 1024) return (bytes / 1024).toFixed(0) + ' KB';
    return bytes + ' B';
  }

  function getCategoryLabel(type: string) {
    switch (type) {
      case 'appdata_roaming':
        return 'AppData Roaming';
      case 'appdata_local':
        return 'AppData Local';
      case 'locallow':
        return 'LocalLow (Oculto)';
      case 'programdata':
        return 'ProgramData';
      case 'program_files':
        return 'Program Files';
      default:
        return 'Sistema';
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden bg-[var(--bg-primary)] p-4 select-none">
  <!-- Cabecera y Estadísticas Fluent -->
  <div class="p-4 rounded-2xl border border-[var(--border-color)] bg-[var(--bg-card)] mb-3 flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div>
      <div class="flex items-center gap-2 mb-1">
        <div class="w-7 h-7 rounded-lg bg-amber-500/20 text-amber-400 flex items-center justify-center border border-amber-500/30">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
        </div>
        <h2 class="text-sm font-bold text-[var(--text-main)]">Rastreador de Archivos Residuales de Apps Antiguas</h2>
      </div>
      <p class="text-xs text-[var(--text-sub)]">
        Filtro inteligente de restos: detecta cachés, logs y carpetas abandonadas, protegiendo enlaces del sistema y partidas guardadas.
      </p>
    </div>

    <!-- Tarjetas de métricas -->
    <div class="flex items-center gap-2.5 shrink-0">
      <div class="px-3 py-1.5 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-right">
        <div class="text-[9px] text-[var(--text-sub)] uppercase font-semibold">Espacio Recuperable</div>
        <div class="text-sm font-extrabold text-amber-400 font-mono">{formatBytes(totalSize)}</div>
      </div>
      <div class="px-3 py-1.5 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-right">
        <div class="text-[9px] text-[var(--text-sub)] uppercase font-semibold">100% Seguros</div>
        <div class="text-sm font-extrabold text-emerald-400 font-mono">{safeCount} carpetas</div>
      </div>
      {#if gameSaveCount > 0}
        <div class="px-3 py-1.5 rounded-xl bg-[var(--bg-primary)] border border-amber-500/30 text-right">
          <div class="text-[9px] text-amber-400 uppercase font-semibold">Partidas Detectadas</div>
          <div class="text-sm font-extrabold text-amber-400 font-mono">{gameSaveCount} juegos</div>
        </div>
      {/if}
    </div>
  </div>

  <!-- AVISO PREVENTIVO DE PARTIDAS GUARDADAS DE VIDEOJUEGOS -->
  {#if gameSaveCount > 0}
    <div class="mb-3 p-3 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-start gap-2.5 text-xs text-amber-200">
      <svg class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" /><line x1="12" y1="9" x2="12" y2="13" /><line x1="12" y1="17" x2="12.01" y2="17" />
      </svg>
      <div class="leading-relaxed">
        <strong class="text-amber-300 font-bold">Aviso sobre Partidas Guardadas de Videojuegos (.sav):</strong>
        Se detectaron carpetas con archivos de guardado local. Por seguridad vienen <strong class="underline">desmarcadas por defecto</strong>. Si no usas Steam Cloud o juegas offline, perderás el avance de tu juego. En cambio, los archivos de shaders y caché sí se regeneran automáticamente al volver a jugar.
      </div>
    </div>
  {/if}

  <!-- Pestañas de filtrado rápido y acciones -->
  <div class="flex items-center justify-between gap-2 mb-2 px-1">
    <div class="flex items-center gap-1.5 text-xs flex-wrap">
      <button
        on:click={() => (activeFilter = 'all')}
        class="px-2.5 py-1 rounded-lg font-medium transition-colors {activeFilter === 'all' ? 'bg-[var(--accent-color)] text-white' : 'text-[var(--text-sub)] hover:bg-[var(--bg-hover)]'}"
      >
        Todos ({$orphanedItems.length})
      </button>
      <button
        on:click={() => (activeFilter = 'safe')}
        class="px-2.5 py-1 rounded-lg font-medium transition-colors {activeFilter === 'safe' ? 'bg-emerald-500 text-white' : 'text-[var(--text-sub)] hover:bg-[var(--bg-hover)]'}"
      >
        100% Recomendables ({safeCount})
      </button>
      {#if gameSaveCount > 0}
        <button
          on:click={() => (activeFilter = 'game_saves')}
          class="px-2.5 py-1 rounded-lg font-medium transition-colors {activeFilter === 'game_saves' ? 'bg-amber-500 text-black font-bold' : 'text-amber-400 hover:bg-amber-500/10'}"
        >
          Partidas de Juegos ⚠️ ({gameSaveCount})
        </button>
      {/if}
      <button
        on:click={() => (activeFilter = 'cache')}
        class="px-2.5 py-1 rounded-lg font-medium transition-colors {activeFilter === 'cache' ? 'bg-[var(--accent-color)] text-white' : 'text-[var(--text-sub)] hover:bg-[var(--bg-hover)]'}"
      >
        Cachés y Shaders
      </button>
      <button
        on:click={() => (activeFilter = 'config')}
        class="px-2.5 py-1 rounded-lg font-medium transition-colors {activeFilter === 'config' ? 'bg-[var(--accent-color)] text-white' : 'text-[var(--text-sub)] hover:bg-[var(--bg-hover)]'}"
      >
        Configuraciones
      </button>
    </div>

    <div class="flex items-center gap-2 shrink-0">
      <button
        on:click={handleScan}
        disabled={$isScanningOrphans || isCleaning}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] text-[var(--text-main)] border border-[var(--border-color)] transition-colors disabled:opacity-50"
      >
        <svg class="w-3.5 h-3.5 {$isScanningOrphans ? 'animate-spin text-amber-400' : 'text-[var(--text-sub)]'}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path d="M3 3v5h5" /><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" /><path d="M16 21h5v-5" />
        </svg>
        <span>{$isScanningOrphans ? 'Rastreando el disco...' : 'Escanear ahora'}</span>
      </button>

      {#if filteredItems.length > 0}
        <button
          on:click={toggleSelectAll}
          class="px-2.5 py-1.5 rounded-lg text-xs font-medium text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
        >
          {selectedItems.length === filteredItems.length ? 'Deseleccionar' : 'Seleccionar todo'}
        </button>
      {/if}

      {#if selectedItems.length > 0}
        <button
          on:click={() => (showConfirmModal = true)}
          disabled={isCleaning}
          class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-bold bg-amber-500 hover:bg-amber-600 text-black shadow-md shadow-amber-500/20 active:scale-98 transition-all disabled:opacity-50"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
          </svg>
          <span>Limpiar ({formatBytes(selectedSize)})</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Lista de carpetas huérfanas con detalles expandibles y recomendaciones -->
  <div class="flex-1 overflow-y-auto space-y-2 pr-1">
    {#if $isScanningOrphans}
      <div class="flex flex-col items-center justify-center h-64 text-[var(--text-sub)] gap-3">
        <div class="w-8 h-8 border-3 border-amber-400 border-t-transparent rounded-full animate-spin"></div>
        <p class="text-xs font-medium">Analizando contenidos de archivos huérfanos y evaluando recomendaciones...</p>
      </div>
    {:else if filteredItems.length === 0}
      <div class="flex flex-col items-center justify-center h-64 text-[var(--text-sub)] gap-3 p-6 text-center">
        <div class="w-12 h-12 rounded-2xl bg-[var(--bg-card)] border border-[var(--border-color)] flex items-center justify-center text-emerald-400">
          <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
        <div>
          <h3 class="text-xs font-bold text-[var(--text-main)]">¡No se encontraron elementos residuales en esta categoría!</h3>
          <p class="text-[11px] text-[var(--text-sub)] mt-1 max-w-sm">
            Tu almacenamiento no tiene residuos de aplicaciones antiguas en esta sección.
          </p>
        </div>
      </div>
    {:else}
      {#each filteredItems as item (item.id)}
        <div class="rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] overflow-hidden hover:border-[var(--accent-color)]/40 transition-colors">
          <!-- Fila principal -->
          <div class="p-3 flex items-center justify-between gap-3">
            <div class="flex items-center gap-3 min-w-0 flex-1">
              <input
                type="checkbox"
                checked={item.isSelected}
                on:change={() => toggleItem(item.id)}
                class="rounded text-amber-500 focus:ring-0 cursor-pointer"
              />

              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-xs font-bold text-[var(--text-main)] truncate">{item.name}</span>

                  <!-- Badge de Partida de Videojuego o Recomendación -->
                  {#if item.isGameSave}
                    <span class="text-[9px] px-1.5 py-0.2 rounded font-bold bg-amber-500/20 text-amber-400 border border-amber-500/40 flex items-center gap-1">
                      <span>⚠️ Partidas Guardadas (Precaución)</span>
                    </span>
                  {:else}
                    <span class="text-[9px] px-1.5 py-0.2 rounded font-bold {item.recommendationLevel === 'highly_recommended' ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30' : 'bg-sky-500/15 text-sky-400 border border-sky-500/30'}">
                      {item.recommendationBadge}
                    </span>
                  {/if}

                  <!-- Categoría de ubicación -->
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-[var(--bg-hover)] text-[var(--text-sub)] border border-[var(--border-color)]">
                    {getCategoryLabel(item.itemType)}
                  </span>

                  <!-- Aviso si requiere reinicio -->
                  {#if item.requiresReboot}
                    <span class="text-[9px] px-1.5 py-0.2 rounded bg-amber-500/15 text-amber-400 border border-amber-500/30">
                      Archivo en uso (requiere reinicio)
                    </span>
                  {/if}
                </div>

                <div class="flex items-center gap-2 text-[11px] text-[var(--text-sub)] mt-0.5">
                  <span class="font-medium text-amber-400/90">{item.originalAppHint}</span>
                  <span>•</span>
                  <span class="font-mono truncate">{item.path}</span>
                </div>
              </div>
            </div>

            <!-- Métricas y botones -->
            <div class="flex items-center gap-2.5 shrink-0 text-xs">
              <div class="text-right">
                <div class="font-bold text-amber-400 font-mono">{formatBytes(item.sizeBytes)}</div>
                <div class="text-[10px] text-[var(--text-sub)]">{item.fileCount} archivos</div>
              </div>

              <!-- BOTÓN: VER ARCHIVOS A ELIMINAR A DETALLE -->
              <button
                on:click={() => handleInspectFiles(item)}
                class="px-2.5 py-1 rounded-lg text-[11px] font-semibold bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] text-[var(--accent-color)] border border-[var(--border-color)] transition-colors flex items-center gap-1"
                title="Inspeccionar la lista detallada de archivos que serán borrados"
              >
                <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" /><circle cx="12" cy="12" r="3" />
                </svg>
                <span>Ver archivos</span>
              </button>

              <!-- Botón ver qué contiene / desglose -->
              <button
                on:click={() => toggleExpand(item.id)}
                class="px-2 py-1 rounded-lg text-[11px] font-medium bg-[var(--bg-hover)] text-[var(--text-sub)] hover:text-[var(--text-main)] border border-[var(--border-color)] transition-colors"
                title="Ver resumen de composición"
              >
                {expandedItemId === item.id ? 'Ocultar' : 'Diagnóstico'}
              </button>

              <!-- Botón abrir en explorador -->
              <button
                on:click={() => openFolder(item.path)}
                aria-label="Abrir en Explorador de Windows"
                class="p-1.5 rounded-lg text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] border border-[var(--border-color)] transition-colors"
                title="Abrir carpeta en el Explorador de Windows"
              >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                  <polyline points="15 3 21 3 21 9" /><line x1="10" x2="21" y1="14" y2="3" />
                </svg>
              </button>

              <!-- Botón eliminar individual -->
              <button
                on:click={() => handleCleanSingle(item)}
                disabled={isCleaning}
                class="px-2.5 py-1 rounded-lg text-[11px] font-semibold text-rose-400 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 transition-colors disabled:opacity-50"
              >
                Eliminar
              </button>
            </div>
          </div>

          <!-- Panel expandible de diagnóstico detallado -->
          {#if expandedItemId === item.id}
            <div class="px-4 py-3 bg-[var(--bg-secondary)] border-t border-[var(--border-color)] text-xs space-y-2 animate-in fade-in duration-100">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <!-- Desglose de tipos de archivos -->
                <div class="p-2.5 rounded-lg bg-[var(--bg-card)] border border-[var(--border-color)]">
                  <div class="text-[10px] font-bold uppercase tracking-wider text-[var(--text-sub)] mb-1.5">Composición de archivos basuras:</div>
                  <ul class="space-y-1">
                    {#each item.contentBreakdown as line}
                      <li class="flex items-center gap-1.5 text-[11px] text-[var(--text-main)]">
                        <span class="w-1.5 h-1.5 rounded-full {line.includes('⚠️') ? 'bg-amber-400' : 'bg-emerald-400'}"></span>
                        <span>{line}</span>
                      </li>
                    {/each}
                  </ul>
                </div>

                <!-- Evaluación de impacto y recomendación -->
                <div class="p-2.5 rounded-lg bg-[var(--bg-card)] border border-[var(--border-color)]">
                  <div class="text-[10px] font-bold uppercase tracking-wider text-[var(--text-sub)] mb-1.5">¿Es recomendable eliminarlo?</div>
                  <p class="text-[11px] leading-relaxed text-[var(--text-main)]">
                    {item.impactExplanation}
                  </p>
                  <div class="mt-2 text-[10px] text-[var(--text-sub)]">
                    {item.requiresReboot ? '⚠️ Contiene archivos que actualmente están retenidos por Windows. Si lo eliminas, se programará su borrado para el próximo reinicio.' : '✅ Los archivos se pueden purgar de inmediato sin necesidad de reiniciar.'}
                  </div>
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- MODAL DE PROGRESO DE LIMPIEZA EN TIEMPO REAL CON PORCENTAJE (0% a 100%) -->
  {#if isCleaning}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-md p-4 select-none">
      <div class="w-full max-w-lg rounded-2xl p-6 border border-[var(--border-color)] bg-[var(--bg-secondary)] shadow-2xl space-y-5 animate-in zoom-in-95 duration-150">
        <!-- Encabezado con spinner -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-xl bg-amber-500/20 text-amber-400 flex items-center justify-center border border-amber-500/30">
              <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path d="M3 3v5h5" /><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" /><path d="M16 21h5v-5" />
              </svg>
            </div>
            <div>
              <h3 class="text-sm font-bold text-[var(--text-main)]">Eliminando Archivos Residuales en Tiempo Real</h3>
              <p class="text-[11px] text-[var(--text-sub)]">Erradicando carpetas huérfanas y liberando espacio...</p>
            </div>
          </div>
          <div class="text-2xl font-black font-mono text-amber-400">
            {cleanProgress.percentage}%
          </div>
        </div>

        <!-- Barra de progreso fluida -->
        <div class="space-y-1.5">
          <div class="w-full h-3 rounded-full bg-[var(--bg-primary)] overflow-hidden border border-[var(--border-color)] relative">
            <div
              class="h-full bg-gradient-to-r from-amber-500 via-amber-400 to-emerald-400 transition-all duration-150 ease-out shadow-lg shadow-amber-500/30"
              style="width: {cleanProgress.percentage}%"
            ></div>
          </div>
          <div class="flex items-center justify-between text-[11px] text-[var(--text-sub)] font-mono">
            <span>Progreso general: {cleanProgress.percentage}%</span>
            <span>Eliminados: {cleanProgress.deletedCount} de {cleanProgress.totalCount}</span>
          </div>
        </div>

        <!-- Visor del archivo actual que se está eliminando en vivo -->
        <div class="p-3 rounded-xl bg-[var(--bg-card)] border border-[var(--border-color)] text-xs space-y-1">
          <div class="text-[10px] text-[var(--text-sub)] uppercase font-semibold">Elemento actual siendo purgado:</div>
          <div class="font-mono text-[11px] text-amber-300 truncate" title={cleanProgress.currentItem}>
            {cleanProgress.currentItem || 'Iniciando proceso...'}
          </div>
          <div class="text-[10px] text-emerald-400 font-semibold pt-1">
            Espacio liberado hasta el momento: <span class="font-mono">{formatBytes(cleanProgress.bytesFreed)}</span>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- MODAL DE INSPECCIÓN DETALLADA DE ARCHIVOS ("Ver archivos a eliminar") -->
  {#if showFilesModal && inspectingItem}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-md p-4 select-none">
      <div class="w-full max-w-2xl max-h-[85vh] rounded-2xl p-5 border border-[var(--border-color)] bg-[var(--bg-secondary)] shadow-2xl flex flex-col gap-3 animate-in zoom-in-95 duration-150">
        <!-- Encabezado del modal -->
        <div class="flex items-center justify-between pb-2 border-b border-[var(--border-color)]">
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-bold text-[var(--text-main)]">Archivos a Eliminar: {inspectingItem.name}</h3>
              {#if inspectingItem.isGameSave}
                <span class="text-[9px] px-1.5 py-0.2 rounded font-bold bg-amber-500/20 text-amber-400 border border-amber-500/40">
                  ⚠️ Partidas Guardadas
                </span>
              {/if}
            </div>
            <p class="text-[11px] text-[var(--text-sub)] font-mono truncate max-w-lg mt-0.5" title={inspectingItem.path}>
              {inspectingItem.path}
            </p>
          </div>
          <button
            on:click={() => (showFilesModal = false)}
            aria-label="Cerrar modal de inspección"
            class="p-1 rounded-lg text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <!-- Barra de búsqueda interna y métricas -->
        <div class="flex items-center justify-between gap-3 text-xs">
          <div class="relative flex-1 max-w-sm">
            <input
              type="text"
              bind:value={fileSearchQuery}
              placeholder="Filtrar archivos por nombre o extensión..."
              class="w-full pl-7 pr-3 py-1 rounded-lg bg-[var(--bg-card)] border border-[var(--border-color)] text-xs text-[var(--text-main)] focus:outline-none focus:border-[var(--accent-color)]"
            />
            <svg class="w-3.5 h-3.5 text-[var(--text-sub)] absolute left-2 top-1/2 -translate-y-1/2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
            </svg>
          </div>
          <div class="text-[11px] text-[var(--text-sub)] font-mono">
            {filteredInspectFiles.length} de {inspectingFiles.length} archivos ({formatBytes(inspectingItem.sizeBytes)})
          </div>
        </div>

        <!-- Lista de archivos -->
        <div class="flex-1 overflow-y-auto min-h-[300px] max-h-[460px] border border-[var(--border-color)] rounded-xl bg-[var(--bg-card)] divide-y divide-[var(--border-color)]">
          {#if isLoadingFiles}
            <div class="flex flex-col items-center justify-center h-48 text-[var(--text-sub)] gap-2">
              <div class="w-6 h-6 border-2 border-amber-400 border-t-transparent rounded-full animate-spin"></div>
              <span class="text-xs">Inspeccionando archivos en disco...</span>
            </div>
          {:else if filteredInspectFiles.length === 0}
            <div class="flex items-center justify-center h-48 text-xs text-[var(--text-sub)]">
              No se encontraron archivos que coincidan con la búsqueda.
            </div>
          {:else}
            {#each filteredInspectFiles as file}
              <div class="p-2 px-3 flex items-center justify-between gap-3 text-xs hover:bg-[var(--bg-hover)] transition-colors">
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    {#if file.isGameSave}
                      <span class="text-[9px] px-1 py-0.2 rounded font-bold bg-amber-500/20 text-amber-400 border border-amber-500/30 shrink-0">
                        SAVE
                      </span>
                    {/if}
                    <span class="font-medium text-[var(--text-main)] truncate">{file.name}</span>
                  </div>
                  <div class="text-[10px] text-[var(--text-sub)] font-mono truncate mt-0.5">
                    {file.relativePath}
                  </div>
                </div>

                <div class="flex items-center gap-3 shrink-0 text-right">
                  <span class="text-[10px] px-1.5 py-0.2 rounded bg-[var(--bg-primary)] text-[var(--text-sub)] border border-[var(--border-color)]">
                    {file.category}
                  </span>
                  <span class="font-mono text-[11px] font-bold text-amber-400">
                    {formatBytes(file.sizeBytes)}
                  </span>
                </div>
              </div>
            {/each}
          {/if}
        </div>

        <!-- Pie del modal -->
        <div class="flex items-center justify-between pt-2 border-t border-[var(--border-color)] text-xs">
          <button
            on:click={() => openFolder(inspectingItem.path)}
            class="px-3 py-1.5 rounded-lg border border-[var(--border-color)] text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors flex items-center gap-1.5"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" /><line x1="10" x2="21" y1="14" y2="3" />
            </svg>
            <span>Abrir en el Explorador</span>
          </button>

          <button
            on:click={() => (showFilesModal = false)}
            class="px-4 py-1.5 rounded-lg font-bold bg-[var(--accent-color)] text-white hover:opacity-90 transition-opacity"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Modal de confirmación antes de limpiar -->
  {#if showConfirmModal}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 select-none">
      <div class="w-full max-w-md rounded-2xl p-5 border border-[var(--border-color)] bg-[var(--bg-secondary)] shadow-2xl">
        <h3 class="text-sm font-bold text-[var(--text-main)] mb-1.5">¿Eliminar {selectedItems.length} elementos residuales?</h3>
        <p class="text-xs text-[var(--text-sub)] leading-relaxed mb-3">
          Se eliminarán de forma permanente <strong class="text-amber-400 font-bold">{formatBytes(selectedSize)}</strong> de datos basura y configuraciones huérfanas.
        </p>

        {#if selectedItems.some((i) => i.isGameSave)}
          <div class="p-2.5 rounded-xl bg-amber-500/15 border border-amber-500/30 text-amber-200 text-[11px] mb-3">
            ⚠️ <strong>Atención:</strong> Has seleccionado elementos que contienen partidas guardadas de videojuegos. Si no tienes activo Steam Cloud, perderás el progreso de esos juegos.
          </div>
        {:else}
          <div class="p-2.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-300 text-[11px] mb-4">
            ✓ 100% Seguro: Ninguna aplicación activa ni componente de Windows se verá afectado.
          </div>
        {/if}

        <div class="flex items-center justify-end gap-2 text-xs">
          <button
            on:click={() => (showConfirmModal = false)}
            class="px-3 py-1.5 rounded-lg border border-[var(--border-color)] text-[var(--text-main)] hover:bg-[var(--bg-hover)]"
          >
            Cancelar
          </button>
          <button
            on:click={handleCleanSelected}
            class="px-3.5 py-1.5 rounded-lg font-bold bg-amber-500 hover:bg-amber-600 text-black shadow-md shadow-amber-500/20"
          >
            Confirmar eliminación
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Modal de resultado de limpieza con AVISO DE REINICIO AHORA O DESPUÉS -->
  {#if showResultModal && cleanResult}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 select-none">
      <div class="w-full max-w-md rounded-2xl p-5 border border-[var(--border-color)] bg-[var(--bg-secondary)] shadow-2xl text-center">
        <div class="w-10 h-10 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center mx-auto mb-3">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
        <h3 class="text-sm font-bold text-[var(--text-main)] mb-1">¡Limpieza completada con éxito!</h3>
        <p class="text-xs text-[var(--text-sub)] mb-3">
          Se eliminaron <strong class="text-[var(--text-main)]">{cleanResult.totalItemsDeleted} carpetas</strong> y se recuperaron <strong class="text-emerald-400 font-bold">{formatBytes(cleanResult.totalBytesFreed)}</strong> de espacio en disco.
        </p>

        <!-- SECCIÓN DE DECISIÓN DE REINICIO SI FUE NECESARIO -->
        {#if cleanResult.rebootRequired}
          <div class="p-3 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-200 text-left text-xs mb-4">
            <div class="font-bold flex items-center gap-1.5 text-amber-400 mb-1">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
              </svg>
              <span>Acción de Reinicio Pendiente</span>
            </div>
            <p class="text-[11px] leading-relaxed">
              {cleanResult.rebootReason || 'Algunos archivos estaban en uso por servicios de Windows. Se programó su borrado al reiniciar.'}
            </p>
            <div class="text-[10px] text-amber-300/80 mt-1">¿Deseas reiniciar la computadora ahora o después?</div>
          </div>

          <div class="grid grid-cols-2 gap-2 text-xs">
            <button
              on:click={() => (showResultModal = false)}
              class="py-2 rounded-lg font-medium border border-[var(--border-color)] text-[var(--text-main)] hover:bg-[var(--bg-hover)]"
            >
              Reiniciar después
            </button>
            <button
              on:click={handleRebootNow}
              class="py-2 rounded-lg font-bold bg-amber-500 hover:bg-amber-600 text-black shadow-md shadow-amber-500/20"
            >
              Reiniciar ahora
            </button>
          </div>
        {:else}
          <div class="p-2.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-300 text-[11px] mb-4">
            ✓ Todos los archivos fueron eliminados al instante. No es necesario reiniciar el equipo.
          </div>
          <button
            on:click={() => (showResultModal = false)}
            class="w-full py-1.5 rounded-lg font-bold text-xs bg-[var(--accent-color)] text-white hover:opacity-90"
          >
            Aceptar
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
