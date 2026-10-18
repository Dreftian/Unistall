<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { InstalledApp, UninstallOptions, UninstallProgressUpdate } from '../types/app';
  import { startStreamUninstall } from '../lib/tauriBridge';

  export let app: InstalledApp;

  const dispatch = createEventDispatcher();

  // Estados: 'prompt' (elección) | 'running' (progreso) | 'finished' (resumen) | 'error'
  let state: 'prompt' | 'running' | 'finished' | 'error' = 'prompt';

  // Opciones de desinstalación elegibles por el usuario
  let options: UninstallOptions = {
    runOfficialUninstaller: true,
    removeInstallationDirectory: true,
    removeAppData: true,
    removeLocalLowAndHidden: true,
    removeRegistryKeys: true,
    removeShortcuts: true,
    removeTempAndJunk: true,
  };

  // Progreso en vivo
  let percentage = 0;
  let currentTask = '';
  let currentItem = '';

  // Resumen final
  let totalDeleted = 0;
  let bytesFreed = 0;
  let summaryItems: string[] = [];
  let errorMessage = '';

  async function handleStartUninstall() {
    state = 'running';
    percentage = 5;
    currentTask = 'Iniciando proceso de desinstalación...';
    currentItem = app.displayName;

    try {
      await startStreamUninstall(app, options, (update: UninstallProgressUpdate) => {
        if (update.event === 'step') {
          percentage = update.data.percentage;
          currentTask = update.data.taskName;
          currentItem = update.data.currentItem;
        } else if (update.event === 'finished') {
          percentage = 100;
          totalDeleted = update.data.totalItemsDeleted;
          bytesFreed = update.data.totalBytesFreed;
          summaryItems = update.data.summary;
          state = 'finished';
          dispatch('uninstalled', app);
        } else if (update.event === 'error') {
          errorMessage = update.data.message;
          state = 'error';
        }
      });
    } catch (err: any) {
      console.error(err);
      errorMessage = err?.toString() || 'Error al comunicarse con el motor de desinstalación';
      state = 'error';
    }
  }

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
  <div class="w-full max-w-xl rounded-2xl shadow-2xl overflow-hidden flex flex-col fluent-glass max-h-[88vh]">
    <!-- Cabecera -->
    <div class="px-5 py-3.5 border-b border-[var(--border-color)] flex items-center justify-between">
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
          <div class="flex items-center gap-2">
            <h2 class="text-sm font-bold text-[var(--text-main)] truncate max-w-sm">{app.displayName}</h2>
            <span class="text-[9px] px-1.5 py-0.2 rounded bg-[var(--bg-hover)] text-[var(--accent-color)] font-medium border border-[var(--border-color)]">
              {app.category}
            </span>
          </div>
          <p class="text-[11px] text-[var(--text-sub)] truncate">{app.publisher || 'Desconocido'} • v{app.displayVersion || '1.0'}</p>
        </div>
      </div>
      {#if state !== 'running'}
        <button on:click={close} aria-label="Cerrar modal" class="p-1.5 rounded-lg text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      {/if}
    </div>

    <!-- Contenido del modal -->
    <div class="p-5 flex-1 overflow-y-auto space-y-4">
      {#if state === 'prompt'}
        <!-- QUÉ ES Y QUÉ HACE -->
        <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)]">
          <div class="text-[10px] font-bold uppercase tracking-wider text-[var(--text-sub)] mb-1">Propósito de la aplicación</div>
          <p class="text-xs text-[var(--text-main)] leading-relaxed">{app.description}</p>
        </div>

        <!-- EVALUACIÓN DE SEGURIDAD / ADVERTENCIA SI ES DEL SISTEMA -->
        {#if app.isCriticalSystem}
          <div class="p-3.5 rounded-xl border border-rose-500/40 bg-rose-500/10 text-rose-300">
            <div class="flex items-center gap-2 font-bold text-xs text-rose-400 mb-1">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
                <line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              <span>¡COMPONENTE CRÍTICO DEL SISTEMA DETECTADO!</span>
            </div>
            <p class="text-[11px] leading-relaxed text-rose-200">
              {app.systemRiskWarning || 'Este paquete es una dependencia vital para Windows u otras aplicaciones. Eliminarlo puede provocar inestabilidad o impedir que otros programas inicien.'}
            </p>
          </div>
        {/if}

        <!-- COMPROBACIÓN DE QUÉ DESINSTALAR -->
        <div>
          <div class="text-xs font-bold text-[var(--text-main)] mb-2">Comprobación de elementos a desinstalar:</div>
          <div class="space-y-1.5 text-xs">
            <label class="flex items-center gap-2.5 p-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] cursor-pointer hover:bg-[var(--bg-hover)] transition-colors">
              <input type="checkbox" bind:checked={options.runOfficialUninstaller} class="rounded text-[var(--accent-color)] focus:ring-0" />
              <div class="flex-1">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-[var(--text-main)]">Desinstalador oficial de la aplicación</span>
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-sky-500/10 text-sky-400 border border-sky-500/20 font-semibold">Recomendado</span>
                </div>
                <p class="text-[10px] text-[var(--text-sub)]">Ejecuta el asistente oficial de desinstalación provisto por el desarrollador</p>
              </div>
            </label>

            <label class="flex items-center gap-2.5 p-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] cursor-pointer hover:bg-[var(--bg-hover)] transition-colors">
              <input type="checkbox" bind:checked={options.removeInstallationDirectory} class="rounded text-[var(--accent-color)] focus:ring-0" />
              <div class="flex-1">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-[var(--text-main)]">Directorio principal de instalación (Program Files / Binarios)</span>
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-semibold">100% Recomendable</span>
                </div>
                <p class="text-[10px] text-[var(--text-sub)]">Erradica la carpeta raíz del software si el desinstalador la deja intacta</p>
              </div>
            </label>

            <label class="flex items-center gap-2.5 p-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] cursor-pointer hover:bg-[var(--bg-hover)] transition-colors">
              <input type="checkbox" bind:checked={options.removeAppData} class="rounded text-[var(--accent-color)] focus:ring-0" />
              <div class="flex-1">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-[var(--text-main)]">Carpetas de datos de usuario (AppData Roaming y Local)</span>
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-sky-500/10 text-sky-400 border border-sky-500/20 font-semibold">Recomendado</span>
                </div>
                <p class="text-[10px] text-[var(--text-sub)]">Elimina perfiles, historiales y configuraciones de la app</p>
              </div>
            </label>

            <label class="flex items-center gap-2.5 p-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] cursor-pointer hover:bg-[var(--bg-hover)] transition-colors">
              <input type="checkbox" bind:checked={options.removeLocalLowAndHidden} class="rounded text-[var(--accent-color)] focus:ring-0" />
              <div class="flex-1">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-[var(--text-main)]">Archivos ocultos, secretos y cachés profundas (LocalLow)</span>
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-semibold">100% Basura</span>
                </div>
                <p class="text-[10px] text-[var(--text-sub)]">Erradica rastreadores, registros ocultos y gigabytes de cachés no estándar</p>
              </div>
            </label>

            <label class="flex items-center gap-2.5 p-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] cursor-pointer hover:bg-[var(--bg-hover)] transition-colors">
              <input type="checkbox" bind:checked={options.removeRegistryKeys} class="rounded text-[var(--accent-color)] focus:ring-0" />
              <div class="flex-1">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-[var(--text-main)]">Ramas residuales en el Registro de Windows (HKCU/HKLM)</span>
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-sky-500/10 text-sky-400 border border-sky-500/20 font-semibold">Recomendado</span>
                </div>
                <p class="text-[10px] text-[var(--text-sub)]">Limpia claves de software, asociaciones y entradas en Panel de Control</p>
              </div>
            </label>

            <label class="flex items-center gap-2.5 p-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] cursor-pointer hover:bg-[var(--bg-hover)] transition-colors">
              <input type="checkbox" bind:checked={options.removeShortcuts} class="rounded text-[var(--accent-color)] focus:ring-0" />
              <div class="flex-1">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-[var(--text-main)]">Accesos directos en Escritorio y Menú Inicio</span>
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-sky-500/10 text-sky-400 border border-sky-500/20 font-semibold">Recomendado</span>
                </div>
                <p class="text-[10px] text-[var(--text-sub)]">Elimina archivos .lnk huérfanos de todos los perfiles</p>
              </div>
            </label>

            <label class="flex items-center gap-2.5 p-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] cursor-pointer hover:bg-[var(--bg-hover)] transition-colors">
              <input type="checkbox" bind:checked={options.removeTempAndJunk} class="rounded text-[var(--accent-color)] focus:ring-0" />
              <div class="flex-1">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-[var(--text-main)]">Archivos temporales y basura del instalador (%TEMP%)</span>
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-semibold">100% Basura</span>
                </div>
                <p class="text-[10px] text-[var(--text-sub)]">Purga paquetes de descarga y logs acumulados</p>
              </div>
            </label>
          </div>
        </div>

      {:else if state === 'running'}
        <!-- PROGRESO EN VIVO CON % REAL -->
        <div class="py-6 space-y-5">
          <div class="flex items-center justify-between text-xs">
            <span class="font-semibold text-[var(--text-main)] truncate max-w-sm">{currentTask}</span>
            <span class="font-mono text-base font-bold text-[var(--accent-color)]">{percentage}%</span>
          </div>

          <!-- Barra de porcentaje -->
          <div class="w-full h-2 rounded-full bg-[var(--bg-hover)] overflow-hidden relative">
            <div
              class="h-full bg-gradient-to-r from-[#0078d4] to-[var(--accent-color)] transition-all duration-300 ease-out rounded-full shadow-md shadow-cyan-500/20"
              style="width: {percentage}%;"
            ></div>
          </div>

          <!-- Ticker del archivo / registro actual -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] text-[11px] font-mono text-[var(--text-sub)] truncate">
            <span class="text-[var(--accent-color)] font-bold">Analizando/Borrando:</span> {currentItem}
          </div>

          <p class="text-[11px] text-center text-[var(--text-sub)]">
            Por favor espera mientras el motor erradica los archivos bloqueados y ramas del registro...
          </p>
        </div>

      {:else if state === 'finished'}
        <!-- RESUMEN FINAL -->
        <div class="py-4 text-center space-y-3">
          <div class="w-12 h-12 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center mx-auto">
            <svg class="w-7 h-7" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <h3 class="text-base font-bold text-[var(--text-main)]">¡Desinstalación y Limpieza al 100%!</h3>
          <p class="text-xs text-[var(--text-sub)] max-w-md mx-auto">
            Se ha desinstalado la aplicación y eliminado todo rastro de archivos temporales, ocultos y claves del registro.
          </p>

          <div class="grid grid-cols-2 gap-3 pt-2">
            <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)]">
              <div class="text-xl font-bold text-[var(--accent-color)] font-mono">{totalDeleted}</div>
              <div class="text-[10px] uppercase font-semibold text-[var(--text-sub)]">Elementos Erradicados</div>
            </div>
            <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)]">
              <div class="text-xl font-bold text-emerald-400 font-mono">{formatBytes(bytesFreed)}</div>
              <div class="text-[10px] uppercase font-semibold text-[var(--text-sub)]">Espacio Liberado</div>
            </div>
          </div>

          <div class="p-2.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-300 text-[11px] text-center">
            ✓ Limpieza inmediata completada: Todos los procesos fueron cerrados y los archivos purgados. No es necesario reiniciar el equipo.
          </div>
        </div>

      {:else if state === 'error'}
        <div class="py-6 text-center space-y-2 text-rose-400">
          <svg class="w-10 h-10 mx-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
          <div class="text-sm font-bold text-[var(--text-main)]">Error en la desinstalación</div>
          <p class="text-xs text-rose-300 font-mono max-w-md mx-auto break-all">{errorMessage}</p>
        </div>
      {/if}
    </div>

    <!-- Pie del modal -->
    <div class="px-5 py-3 border-t border-[var(--border-color)] flex items-center justify-between">
      {#if state === 'prompt'}
        <button on:click={close} class="px-4 py-1.5 rounded-lg text-xs font-medium bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] text-[var(--text-main)] border border-[var(--border-color)] transition-colors">
          Cancelar
        </button>
        <button
          on:click={handleStartUninstall}
          class="px-5 py-1.5 rounded-lg text-xs font-bold bg-rose-600 hover:bg-rose-500 text-white transition-all shadow-md shadow-rose-600/20 flex items-center gap-1.5"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
          </svg>
          <span>Iniciar Desinstalación Completa</span>
        </button>
      {:else if state === 'finished' || state === 'error'}
        <div class="w-full flex justify-end">
          <button on:click={close} class="px-5 py-1.5 rounded-lg text-xs font-bold bg-[var(--accent-color)] text-white hover:opacity-90 transition-opacity">
            Aceptar y Finalizar
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
