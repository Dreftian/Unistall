import { invoke, Channel } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type {
  InstalledApp,
  LeftoverItem,
  OrphanedItem,
  StartupEntry,
  DeepCleanResult,
  UninstallOptions,
  UninstallProgressUpdate,
  ResidualCleanProgressUpdate,
  ResidualFileDetail,
} from '../types/app';
import { appStore } from '../stores/appStore';

type ScanEvent =
  | { event: 'batch'; data: { apps: InstalledApp[] } }
  | { event: 'progress'; data: { percentage: number; stage: string; count: number } }
  | { event: 'status'; data: { message: string; scannedCount: number } }
  | { event: 'finished'; data: { totalCount: number } };

export async function initiateSoftwareScan(): Promise<void> {
  appStore.setScanning(true);
  appStore.setScanProgress(0, 'Iniciando escaneo...');
  appStore.clearApps();

  const scanChannel = new Channel<ScanEvent>();

  scanChannel.onmessage = (message: ScanEvent) => {
    switch (message.event) {
      case 'progress':
        appStore.setScanProgress(message.data.percentage, message.data.stage);
        appStore.setStatus(`${message.data.stage} (${message.data.percentage}%)`);
        break;
      case 'batch':
        appStore.appendBatch(message.data.apps);
        break;
      case 'status':
        appStore.setStatus(message.data.message);
        break;
      case 'finished': {
        appStore.setScanProgress(100, 'Escaneo finalizado');
        setTimeout(() => {
          appStore.setScanning(false);
        }, 400);
        const count = message.data?.totalCount ?? get(appStore.apps).length;
        appStore.setStatus(`Escaneo completado: ${count} aplicaciones encontradas.`);
        break;
      }
    }
  };

  try {
    await invoke('start_app_scan_stream', { onEvent: scanChannel });
  } catch (err) {
    console.error('Error durante streaming de escaneo:', err);
    try {
      const apps = await invoke<InstalledApp[]>('get_all_apps_instant');
      appStore.setAllApps(apps);
      appStore.setStatus(`Escaneo finalizado: ${apps.length} aplicaciones encontradas.`);
    } catch (e) {
      console.error('Error en fallback:', e);
    } finally {
      appStore.setScanning(false);
    }
  }
}

export async function startStreamUninstall(
  app: InstalledApp,
  options: UninstallOptions,
  onUpdate: (update: UninstallProgressUpdate) => void
): Promise<void> {
  const channel = new Channel<UninstallProgressUpdate>();
  channel.onmessage = (msg: UninstallProgressUpdate) => {
    onUpdate(msg);
  };

  await invoke('stream_full_uninstall', {
    app,
    options,
    onProgress: channel,
  });
}

export async function uninstallSoftware(uninstallString: string): Promise<void> {
  await invoke('run_uninstall', { uninstallString });
}

export async function executeFullDeepUninstall(app: InstalledApp, waitForOfficial: boolean): Promise<DeepCleanResult> {
  return await invoke<DeepCleanResult>('execute_full_deep_uninstall', {
    app,
    waitForOfficial,
  });
}

export async function findLeftoversForApp(app: InstalledApp): Promise<LeftoverItem[]> {
  return await invoke<LeftoverItem[]>('hunt_leftovers', { app });
}

export async function deleteLeftoversBatch(items: LeftoverItem[]): Promise<number> {
  return await invoke<number>('clean_leftovers_batch', { items });
}

export async function scanOrphanedResiduals(): Promise<OrphanedItem[]> {
  appStore.isScanningOrphans.set(true);
  try {
    const items = await invoke<OrphanedItem[]>('scan_orphaned_residuals');
    appStore.orphanedItems.set(items);
    return items;
  } finally {
    appStore.isScanningOrphans.set(false);
  }
}

export async function streamCleanOrphanedResiduals(
  items: OrphanedItem[],
  onProgress: (update: ResidualCleanProgressUpdate) => void
): Promise<void> {
  const channel = new Channel<ResidualCleanProgressUpdate>();
  channel.onmessage = (msg: ResidualCleanProgressUpdate) => {
    onProgress(msg);
  };

  await invoke('stream_clean_orphaned_residuals', {
    items,
    onProgress: channel,
  });
}

export async function cleanOrphanedResiduals(items: OrphanedItem[]): Promise<DeepCleanResult> {
  const result = await invoke<DeepCleanResult>('clean_orphaned_residuals', { items });
  // Filtrar los elementos eliminados del almacén local
  const deletedPaths = new Set(result.details.map((d) => d.replace('Eliminado: ', '')));
  appStore.orphanedItems.update((current) => current.filter((item) => !deletedPaths.has(item.path)));
  return result;
}

export async function getResidualFileDetails(path: string): Promise<ResidualFileDetail[]> {
  return await invoke<ResidualFileDetail[]>('get_residual_file_details', { path });
}

export async function isSystemElevated(): Promise<boolean> {
  return await invoke<boolean>('is_system_elevated');
}

export async function loadStartupList(): Promise<StartupEntry[]> {
  const entries = await invoke<StartupEntry[]>('get_startup_entries');
  appStore.setStartupEntries(entries);
  return entries;
}

export async function disableStartupItem(id: string): Promise<void> {
  await invoke('toggle_startup_entry', { id });
  appStore.toggleStartupEntry(id);
}

export async function openFolder(path: string): Promise<void> {
  await invoke('open_in_explorer', { path });
}

export async function openRegedit(): Promise<void> {
  await invoke('open_in_regedit');
}

export async function elevateUac(): Promise<void> {
  await invoke('request_uac_elevation');
}

export async function rebootSystemNow(): Promise<void> {
  await invoke('reboot_system_now');
}

export async function minimizeWindow(): Promise<void> {
  try {
    await invoke('window_minimize');
  } catch (e) {
    console.error('Error minimizing window:', e);
  }
}

export async function toggleMaximizeWindow(): Promise<void> {
  try {
    await invoke('window_maximize_toggle');
  } catch (e) {
    console.error('Error maximizing window:', e);
  }
}

export async function closeWindow(): Promise<void> {
  try {
    await invoke('window_close');
  } catch (e) {
    console.error('Error closing window:', e);
  }
}
