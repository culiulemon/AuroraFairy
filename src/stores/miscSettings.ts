import { invoke } from '@tauri-apps/api/core'

export interface MiscSettings {
  globalWorkingDir: string | null
  defaultWorkingDir: string
}

export async function loadMiscSettings(): Promise<MiscSettings> {
  const result = await invoke<{ global_working_dir: string | null; default_working_dir: string }>('get_working_dir_config')
  return {
    globalWorkingDir: result.global_working_dir,
    defaultWorkingDir: result.default_working_dir,
  }
}

export async function setWorkingDir(path: string | null): Promise<void> {
  await invoke('set_working_dir_config', { workingDir: path })
}

export function getEffectiveWorkingDir(settings: MiscSettings): string {
  return settings.globalWorkingDir || settings.defaultWorkingDir
}
