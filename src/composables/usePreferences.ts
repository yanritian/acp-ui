import { ref, onMounted } from 'vue'
import { canPickFolder, pickFolder, loadKvStore, type KVStore } from '../lib/host'
import { initTelemetry } from '../lib/telemetry'

/**
 * Composable for managing user preferences.
 * Handles KVStore-based persistence for selectedCwd and telemetry settings.
 */
export function usePreferences() {
  const selectedCwd = ref('')
  
  // On mobile / web there is no native folder picker (and the cwd refers to
  // a path on the *agent's* machine, not the local device), so we expose a
  // free-text field instead of the picker button.
  const folderPickerAvailable = canPickFolder()
  
  let prefsStore: KVStore | null = null
  
  async function loadPreferences() {
    prefsStore = await loadKvStore('preferences.json')
    
    // Initialize telemetry (check user preference)
    const telemetryEnabled = await prefsStore.get<boolean>('telemetryEnabled') ?? true
    await initTelemetry(telemetryEnabled)
    
    // Load saved CWD
    const savedCwd = await prefsStore.get<string>('lastCwd')
    if (savedCwd) {
      selectedCwd.value = savedCwd
    }
    
    return prefsStore
  }
  
  async function handleSelectFolder() {
    const folder = await pickFolder('Select Working Directory')
    if (folder) {
      selectedCwd.value = folder
      // Persist the selection
      if (prefsStore) {
        await prefsStore.set('lastCwd', folder)
        await prefsStore.save()
      }
    }
  }
  
  /** Persist a typed cwd as the user edits it (mobile / web field). */
  async function handleCwdInput(event: Event) {
    const value = (event.target as HTMLInputElement).value
    selectedCwd.value = value
    if (prefsStore) {
      await prefsStore.set('lastCwd', value)
      await prefsStore.save()
    }
  }
  
  return {
    selectedCwd,
    folderPickerAvailable,
    loadPreferences,
    handleSelectFolder,
    handleCwdInput,
  }
}
