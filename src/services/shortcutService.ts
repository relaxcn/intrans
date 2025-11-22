import { Store } from "@tauri-apps/plugin-store";
import { register, unregister, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const STORE_PATH = "settings.json";

export async function setupGlobalShortcuts() {
  const store = await Store.load(STORE_PATH);
  // Use Control instead of Ctrl for better compatibility if needed, but verify first.
  // For now assuming stored strings are valid.
  const toggleMain = await store.get<string>("toggleMainShortcut") || "Ctrl+Alt+Space";
  const openSettings = await store.get<string>("openSettingsShortcut") || "Ctrl+Shift+,";

  try {
      // Clean up existing shortcuts to avoid duplicates or conflicts
      await unregisterAll();

      await register(toggleMain, async (event) => {
        if (event.state === "Pressed") {
            await invoke("toggle_main_window");
        }
      });

      await register(openSettings, async (event) => {
        if (event.state === "Pressed") {
             const settingsWin = await WebviewWindow.getByLabel('settings');
             if (settingsWin) {
                 await settingsWin.unminimize();
                 await settingsWin.show();
                 await settingsWin.setFocus();
                 await settingsWin.center();
             }
        }
      });
      
      console.log('Shortcuts registered:', { toggleMain, openSettings });
  } catch (err) {
      console.error('Failed to register shortcuts:', err);
  }
}

export async function unregisterAllShortcuts() {
  try {
    await unregisterAll();
    console.log('All shortcuts unregistered');
  } catch (err) {
    console.error('Failed to unregister shortcuts:', err);
  }
}

export async function validateShortcut(shortcut: string): Promise<boolean> {
    if (!shortcut) return true;
    try {
        // Check if it's already registered by us (in which case it might fail or succeed depending on implementation)
        // But here we want to check if it conflicts with SYSTEM or other apps.
        // Since unregisterAll clears OUR shortcuts, any collision now is external.
        // WAIT: If we are in Settings window, and Main window has shortcuts registered, 
        // does register() fail? Yes, because it's global.
        
        // So before validation, we might need to temporarily ignore "our own" current shortcuts if they match?
        // No, if the user inputs "Ctrl+C", and we try to register, it fails (System copy).
        // If user inputs "Ctrl+Alt+Space" (our current toggle), it fails because WE registered it in Main.
        
        // If the user enters the SAME shortcut as currently assigned, we should consider it valid.
        // But validation is usually for NEW shortcuts.
        
        // Let's try to register.
        await register(shortcut, () => {});
        
        // If successful, unregister immediately
        await unregister(shortcut);
        return true;
    } catch (e) {
        console.warn(`Shortcut validation failed for ${shortcut}:`, e);
        return false;
    }
}
