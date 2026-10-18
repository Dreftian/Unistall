<script lang="ts">
  import { appStore } from '../stores/appStore';

  const { apps, filteredApps, statusMessage, isScanning, settings } = appStore;

  $: totalSizeKb = $apps.reduce((acc, app) => acc + (app.estimatedSizeKb || 0), 0);
  $: formattedSize =
    totalSizeKb > 1024 * 1024
      ? (totalSizeKb / (1024 * 1024)).toFixed(2) + ' GB'
      : (totalSizeKb / 1024).toFixed(1) + ' MB';
</script>

<footer class="h-7 bg-[var(--bg-secondary)] border-t border-[var(--border-color)] px-3 flex items-center justify-between text-[11px] text-[var(--text-sub)] select-none transition-colors">
  <!-- Estado de la operación -->
  <div class="flex items-center gap-2 min-w-0 flex-1 pr-4">
    <div class="w-1.5 h-1.5 rounded-full {$isScanning ? 'bg-amber-400 animate-ping' : 'bg-emerald-500'}"></div>
    <span class="truncate">{$statusMessage}</span>
  </div>

  <!-- Métricas de almacenamiento y rendimiento -->
  <div class="flex items-center gap-3 shrink-0">
    <div>
      Apps: <span class="text-[var(--text-main)] font-semibold">{$filteredApps.length}</span> / <span class="text-[var(--text-main)] font-semibold">{$apps.length}</span>
    </div>
    <div class="w-px h-3 bg-[var(--border-color)]"></div>
    <div>
      Disco: <span class="text-[var(--text-main)] font-mono font-medium">{formattedSize}</span>
    </div>
    <div class="w-px h-3 bg-[var(--border-color)]"></div>
    <div class="flex items-center gap-1">
      <span class="w-1.5 h-1.5 rounded-full bg-[var(--accent-color)]"></span>
      RAM: <span class="text-[var(--text-main)] font-mono font-medium">&lt; 30 MB</span>
    </div>
    <div class="w-px h-3 bg-[var(--border-color)]"></div>
    <button
      on:click={() => appStore.updateSettings({ isCompact: !$settings.isCompact })}
      class="hover:text-[var(--text-main)] transition-colors"
      title="Alternar densidad compacta"
    >
      {$settings.isCompact ? 'Compacto' : 'Estándar'}
    </button>
  </div>
</footer>
