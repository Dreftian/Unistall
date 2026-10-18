import { writable, derived } from 'svelte/store';
import type {
  InstalledApp,
  StartupEntry,
  LeftoverItem,
  OrphanedItem,
  FilterCategory,
  AppSettings,
} from '../types/app';

const defaultSettings: AppSettings = {
  theme: 'dark',
  isCompact: true,
  glassEffect: true,
  autoDeepClean: true,
  unlockWithRestartManager: true,
  scanHiddenAndLocalLow: true,
  showSystemComponents: false,
};

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem('unistall_settings');
    if (raw) return { ...defaultSettings, ...JSON.parse(raw) };
  } catch (e) {
    console.error(e);
  }
  return defaultSettings;
}

function createAppStore() {
  const apps = writable<InstalledApp[]>([]);
  const startupEntries = writable<StartupEntry[]>([]);
  const isScanning = writable<boolean>(false);
  const scanPercentage = writable<number>(0);
  const scanStage = writable<string>('');
  const statusMessage = writable<string>('Listo');
  const selectedCategory = writable<FilterCategory>('all');
  const searchQuery = writable<string>('');
  const settings = writable<AppSettings>(loadSettings());

  // Restos de aplicaciones antiguas (huérfanos)
  const orphanedItems = writable<OrphanedItem[]>([]);
  const isScanningOrphans = writable<boolean>(false);

  // Modal de restos por app
  const activeLeftoversApp = writable<InstalledApp | null>(null);
  const leftoverItems = writable<LeftoverItem[]>([]);
  const isHuntingLeftovers = writable<boolean>(false);

  settings.subscribe((s) => {
    try {
      localStorage.setItem('unistall_settings', JSON.stringify(s));
      if (typeof document !== 'undefined') {
        if (s.theme === 'light') {
          document.documentElement.classList.add('theme-light');
          document.documentElement.classList.remove('theme-dark');
        } else {
          document.documentElement.classList.add('theme-dark');
          document.documentElement.classList.remove('theme-light');
        }
      }
    } catch (e) {
      console.error(e);
    }
  });

  const normalize = (str: string) =>
    str
      .toLowerCase()
      .normalize('NFD')
      .replace(/[\u0300-\u036f]/g, '');

  const filteredApps = derived(
    [apps, selectedCategory, searchQuery, settings],
    ([$apps, $cat, $query, $settings]) => {
      const q = normalize($query.trim());

      return $apps.filter((app) => {
        if (!$settings.showSystemComponents && app.isSystemComponent) {
          return false;
        }

        if ($cat === 'win32' && app.source !== 'win32Registry') return false;
        if ($cat === 'uwp' && app.source !== 'windowsStoreUwp') return false;
        if ($cat === 'cli' && app.source !== 'scoop' && app.source !== 'chocolatey' && app.source !== 'winget') return false;

        if (q.length > 0) {
          const nameNorm = normalize(app.displayName);
          const pubNorm = normalize(app.publisher || '');
          const descNorm = normalize(app.description || '');
          const catNorm = normalize(app.category || '');
          if (!nameNorm.includes(q) && !pubNorm.includes(q) && !descNorm.includes(q) && !catNorm.includes(q)) {
            return false;
          }
        }

        return true;
      });
    }
  );

  return {
    apps,
    startupEntries,
    isScanning,
    scanPercentage,
    scanStage,
    statusMessage,
    selectedCategory,
    searchQuery,
    settings,
    filteredApps,
    activeLeftoversApp,
    leftoverItems,
    isHuntingLeftovers,
    orphanedItems,
    isScanningOrphans,

    setScanning: (val: boolean) => isScanning.set(val),
    setScanProgress: (pct: number, stage: string) => {
      scanPercentage.set(pct);
      scanStage.set(stage);
    },
    setStatus: (msg: string) => statusMessage.set(msg),
    clearApps: () => apps.set([]),
    appendBatch: (batch: InstalledApp[]) => {
      apps.update((current) => [...current, ...batch]);
    },
    setAllApps: (newApps: InstalledApp[]) => apps.set(newApps),
    removeAppById: (id: string) => {
      apps.update((current) => current.filter((a) => a.id !== id));
    },
    setStartupEntries: (entries: StartupEntry[]) => startupEntries.set(entries),
    toggleStartupEntry: (id: string) => {
      startupEntries.update((entries) =>
        entries.map((e) => (e.id === id ? { ...e, enabled: !e.enabled } : e))
      );
    },
    updateSettings: (partial: Partial<AppSettings>) => {
      settings.update((s) => ({ ...s, ...partial }));
    },
    setOrphanedItems: (items: OrphanedItem[]) => orphanedItems.set(items),
  };
}

export const appStore = createAppStore();
