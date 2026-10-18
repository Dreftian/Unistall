<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { appStore } from '../stores/appStore';

  const dispatch = createEventDispatcher();
  const { settings } = appStore;

  function close() {
    dispatch('close');
  }

  function toggleTheme() {
    const next = $settings.theme === 'dark' ? 'light' : 'dark';
    appStore.updateSettings({ theme: next });
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 select-none animate-in fade-in duration-150">
  <div class="w-full max-w-lg rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] fluent-glass">
    <!-- Cabecera -->
    <div class="px-5 py-4 border-b border-[var(--border-color)] flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-[var(--accent-color)]/20 text-[var(--accent-color)] flex items-center justify-center">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </div>
        <div>
          <h2 class="text-sm font-semibold text-[var(--text-main)]">Configuración de Unistall</h2>
          <p class="text-xs text-[var(--text-sub)]">Personalización y motor de desinstalación profunda</p>
        </div>
      </div>
      <button on:click={close} aria-label="Cerrar configuración" class="p-1.5 rounded-lg text-[var(--text-sub)] hover:text-[var(--text-main)] hover:bg-[var(--bg-hover)] transition-colors">
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <!-- Opciones -->
    <div class="p-5 flex-1 overflow-y-auto space-y-5 text-xs">
      <!-- Sección Apariencia -->
      <div>
        <h3 class="text-[11px] font-bold uppercase tracking-wider text-[var(--text-sub)] mb-2">Apariencia y Estilo</h3>
        <div class="space-y-2">
          <!-- Selector de Tema -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between">
            <div>
              <div class="font-medium text-[var(--text-main)]">Tema de la interfaz</div>
              <div class="text-[11px] text-[var(--text-sub)]">Alterna entre modo Oscuro y Claro inspirado en Windows 11</div>
            </div>
            <button
              on:click={toggleTheme}
              class="px-3 py-1.5 rounded-lg border border-[var(--border-color)] bg-[var(--bg-hover)] text-[var(--text-main)] font-medium flex items-center gap-1.5 hover:border-[var(--accent-color)] transition-colors"
            >
              {#if $settings.theme === 'dark'}
                <svg class="w-3.5 h-3.5 text-yellow-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
                </svg>
                <span>Oscuro</span>
              {:else}
                <svg class="w-3.5 h-3.5 text-blue-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
                </svg>
                <span>Claro</span>
              {/if}
            </button>
          </div>

          <!-- Modo Compacto -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between">
            <div>
              <div class="font-medium text-[var(--text-main)]">Modo compacto de alta densidad</div>
              <div class="text-[11px] text-[var(--text-sub)]">Reduce la altura de las filas para visualizar más registros por pantalla</div>
            </div>
            <input
              type="checkbox"
              checked={$settings.isCompact}
              on:change={(e) => appStore.updateSettings({ isCompact: e.currentTarget.checked })}
              class="w-4 h-4 rounded text-[var(--accent-color)] focus:ring-0"
            />
          </div>

          <!-- Efecto Glass / Mica -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between">
            <div>
              <div class="font-medium text-[var(--text-main)]">Efecto translúcido Glass / Mica</div>
              <div class="text-[11px] text-[var(--text-sub)]">Aplica desenfoque acrílico y transparencias suaves de Windows 11</div>
            </div>
            <input
              type="checkbox"
              checked={$settings.glassEffect}
              on:change={(e) => appStore.updateSettings({ glassEffect: e.currentTarget.checked })}
              class="w-4 h-4 rounded text-[var(--accent-color)] focus:ring-0"
            />
          </div>
        </div>
      </div>

      <!-- Sección Desinstalación y Limpieza Profunda -->
      <div>
        <h3 class="text-[11px] font-bold uppercase tracking-wider text-[var(--text-sub)] mb-2">Motor de Limpieza Profunda</h3>
        <div class="space-y-2">
          <!-- Auto-Deep Clean -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between">
            <div>
              <div class="font-medium text-[var(--text-main)]">Desinstalación total automática</div>
              <div class="text-[11px] text-[var(--text-sub)]">Tras ejecutar el desinstalador oficial, erradica automáticamente todo rastro</div>
            </div>
            <input
              type="checkbox"
              checked={$settings.autoDeepClean}
              on:change={(e) => appStore.updateSettings({ autoDeepClean: e.currentTarget.checked })}
              class="w-4 h-4 rounded text-[var(--accent-color)] focus:ring-0"
            />
          </div>

          <!-- LocalLow y archivos ocultos -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between">
            <div>
              <div class="font-medium text-[var(--text-main)]">Rastrear LocalLow y carpetas ocultas</div>
              <div class="text-[11px] text-[var(--text-sub)]">Busca en directorios no estándar donde suelen quedar gigabytes de cachés</div>
            </div>
            <input
              type="checkbox"
              checked={$settings.scanHiddenAndLocalLow}
              on:change={(e) => appStore.updateSettings({ scanHiddenAndLocalLow: e.currentTarget.checked })}
              class="w-4 h-4 rounded text-[var(--accent-color)] focus:ring-0"
            />
          </div>

          <!-- Restart Manager -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between">
            <div>
              <div class="font-medium text-[var(--text-main)]">Desbloqueo con Restart Manager API</div>
              <div class="text-[11px] text-[var(--text-sub)]">Cierra procesos que retienen archivos bloqueados o programa borrado en reinicio</div>
            </div>
            <input
              type="checkbox"
              checked={$settings.unlockWithRestartManager}
              on:change={(e) => appStore.updateSettings({ unlockWithRestartManager: e.currentTarget.checked })}
              class="w-4 h-4 rounded text-[var(--accent-color)] focus:ring-0"
            />
          </div>

          <!-- Componentes del sistema -->
          <div class="p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] flex items-center justify-between">
            <div>
              <div class="font-medium text-[var(--text-main)]">Mostrar componentes internos del sistema</div>
              <div class="text-[11px] text-[var(--text-sub)]">Muestra dependencias y parches marcados con SystemComponent=1</div>
            </div>
            <input
              type="checkbox"
              checked={$settings.showSystemComponents}
              on:change={(e) => appStore.updateSettings({ showSystemComponents: e.currentTarget.checked })}
              class="w-4 h-4 rounded text-[var(--accent-color)] focus:ring-0"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Pie -->
    <div class="px-5 py-3 border-t border-[var(--border-color)] flex items-center justify-between">
      <span class="text-[11px] text-[var(--text-sub)]">Unistall v1.1.0 • Portable & Native</span>
      <button
        on:click={close}
        class="px-4 py-1.5 rounded-lg text-xs font-medium bg-[var(--accent-color)] text-white hover:opacity-90 transition-opacity"
      >
        Aceptar
      </button>
    </div>
  </div>
</div>
