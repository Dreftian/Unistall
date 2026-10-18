export type AppArchitecture = 'x64' | 'x86' | 'arm64' | 'universal';

export type AppInstallSource = 
  | 'win32Registry' 
  | 'windowsStoreUwp' 
  | 'winget' 
  | 'scoop' 
  | 'chocolatey';

export type RegistryHiveSource = 'hklm64' | 'hklm32' | 'hkcu';

export interface InstalledApp {
  id: string;
  displayName: string;
  displayVersion?: string;
  publisher?: string;
  installDate?: string;
  installLocation?: string;
  uninstallString?: string;
  quietUninstallString?: string;
  displayIcon?: string;
  iconBase64?: string;
  estimatedSizeKb?: number;
  realSizeBytes?: number;
  architecture: AppArchitecture;
  source: AppInstallSource;
  registryHive?: RegistryHiveSource;
  registryKeyPath?: string;
  isSystemComponent: boolean;
  isStartupEnabled: boolean;
  windowsInstallerGuid?: string;

  // Inteligencia y seguridad
  description: string;
  category: string;
  isCriticalSystem: boolean;
  systemRiskWarning?: string;
}

export type LeftoverType = 'directory' | 'file' | 'registryKey' | 'registryValue' | 'comClass';

export interface LeftoverItem {
  id: string;
  itemType: LeftoverType;
  pathOrKey: string;
  confidenceScore: number;
  sizeBytes?: number;
  rationale: string;
  isSelected: boolean;
}

export interface OrphanedItem {
  id: string;
  name: string;
  originalAppHint: string;
  path: string;
  itemType: string;
  sizeBytes: number;
  fileCount: number;
  lastModified?: string;
  confidenceScore: number;
  description: string;
  isSelected: boolean;
  contentBreakdown: string[];
  recommendationLevel: 'highly_recommended' | 'recommended' | 'caution' | 'optional';
  recommendationBadge: string;
  impactExplanation: string;
  requiresReboot: boolean;
  isGameSave?: boolean;
  gameSaveCount?: number;
}

export interface ResidualFileDetail {
  name: string;
  path: string;
  relativePath: string;
  sizeBytes: number;
  extension: string;
  category: string;
  isGameSave: boolean;
}

export interface StartupEntry {
  id: string;
  name: string;
  command: string;
  location: string;
  enabled: boolean;
  publisher?: string;
}

export type FilterCategory = 'all' | 'win32' | 'uwp' | 'cli' | 'startup' | 'residuals' | 'leftovers';

export interface DeepCleanResult {
  totalItemsDeleted: number;
  totalBytesFreed: number;
  details: string[];
  rebootRequired: boolean;
  rebootReason?: string;
}

export interface AppSettings {
  theme: 'dark' | 'light';
  isCompact: boolean;
  glassEffect: boolean;
  autoDeepClean: boolean;
  unlockWithRestartManager: boolean;
  scanHiddenAndLocalLow: boolean;
  showSystemComponents: boolean;
}

export interface UninstallOptions {
  runOfficialUninstaller: boolean;
  removeInstallationDirectory: boolean;
  removeAppData: boolean;
  removeLocalLowAndHidden: boolean;
  removeRegistryKeys: boolean;
  removeShortcuts: boolean;
  removeTempAndJunk: boolean;
}

export type UninstallProgressUpdate =
  | { event: 'step'; data: { percentage: number; taskName: string; currentItem: string } }
  | { event: 'finished'; data: { totalItemsDeleted: number; totalBytesFreed: number; summary: string[] } }
  | { event: 'error'; data: { message: string } };

export type ResidualCleanProgressUpdate =
  | { event: 'progress'; data: { percentage: number; currentItem: string; deletedCount: number; totalCount: number; bytesFreed: number } }
  | { event: 'finished'; data: { result: DeepCleanResult } }
  | { event: 'error'; data: { message: string } };
