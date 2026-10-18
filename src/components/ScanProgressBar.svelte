<script lang="ts">
  import { appStore } from '../stores/appStore';

  const { isScanning, scanPercentage, scanStage } = appStore;
</script>

{#if $isScanning}
  <div class="w-full bg-[var(--bg-secondary)] border-b border-[var(--border-color)] px-4 py-2 select-none animate-in fade-in duration-100">
    <div class="flex items-center justify-between text-[11px] mb-1">
      <div class="flex items-center gap-2 text-[var(--text-main)] font-medium truncate max-w-lg">
        <div class="w-2 h-2 rounded-full bg-[var(--accent-color)] animate-ping"></div>
        <span>{$scanStage || 'Escaneando software del sistema...'}</span>
      </div>
      <span class="font-mono text-xs font-bold text-[var(--accent-color)]">{$scanPercentage}%</span>
    </div>

    <!-- Barra de progreso visual -->
    <div class="w-full h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden relative">
      <div
        class="h-full bg-gradient-to-r from-[#0078d4] to-[var(--accent-color)] transition-all duration-200 ease-out rounded-full"
        style="width: {$scanPercentage}%;"
      ></div>
    </div>
  </div>
{/if}
