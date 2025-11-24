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
    // Clean up existing shortcuts
    await unregisterAllShortcuts();

    // Register global shortcut for toggling main window
    await register(toggleMain, async (event) => {
      if (event.state === "Pressed") {
        await invoke("toggle_main_window");
      }
    });

    // Register local shortcut for opening settings (only works when window has focus)
    if (openSettings) {
      setupLocalShortcut(openSettings);
    }
    
    console.log('Shortcuts registered:', { toggleMain, openSettings });
  } catch (err) {
    console.error('Failed to register shortcuts:', err);
  }
}

let localKeyHandler: ((e: KeyboardEvent) => void) | null = null;

function setupLocalShortcut(shortcut: string) {
  if (!shortcut) return;

  localKeyHandler = async (e: KeyboardEvent) => {
    if (matchShortcut(e, shortcut)) {
      e.preventDefault();
      const settingsWin = await WebviewWindow.getByLabel('settings');
      if (settingsWin) {
        await settingsWin.unminimize();
        await settingsWin.show();
        await settingsWin.setFocus();
        await settingsWin.center();
      }
    }
  };

  window.addEventListener('keydown', localKeyHandler);
}

// Key mapping consistent with ShortcutRecorder.vue
const CODE_MAP: Record<string, string> = {
    'Backquote': '`',
    'Minus': '-',
    'Equal': '=',
    'BracketLeft': '[',
    'BracketRight': ']',
    'Backslash': '\\',
    'Semicolon': ';',
    'Quote': "'",
    'Comma': ',',
    'Period': '.',
    'Slash': '/',
    'Space': 'Space',
    'ArrowUp': 'Up',
    'ArrowDown': 'Down',
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right',
    'Tab': 'Tab',
    'Enter': 'Enter',
    'Backspace': 'Backspace',
    'Delete': 'Delete',
    'Insert': 'Insert',
    'Home': 'Home',
    'End': 'End',
    'PageUp': 'PageUp',
    'PageDown': 'PageDown',
    'Escape': 'Esc',
};

// Add F-keys
for (let i = 1; i <= 24; i++) {
    CODE_MAP[`F${i}`] = `F${i}`;
}

function getStandardKey(e: KeyboardEvent): string {
    const code = e.code;

    // Letters
    if (code.startsWith('Key')) {
        return code.slice(3).toUpperCase();
    }

    // Digits
    if (code.startsWith('Digit')) {
        return code.slice(5);
    }

    // Numpad
    if (code.startsWith('Numpad')) {
        if (code.length === 7 && !isNaN(Number(code[6]))) {
            return code[6];
        }
        switch (code) {
            case 'NumpadAdd': return '+';
            case 'NumpadSubtract': return '-';
            case 'NumpadMultiply': return '*';
            case 'NumpadDivide': return '/';
            case 'NumpadDecimal': return '.';
            case 'NumpadEnter': return 'Enter';
        }
    }

    // Standard Map
    if (CODE_MAP[code]) {
        return CODE_MAP[code];
    }

    // Fallback
    let key = e.key.toUpperCase();
    if (key === ' ') return 'Space';
    return key;
}

function matchShortcut(e: KeyboardEvent, shortcut: string): boolean {
    const parts = shortcut.split('+').map(p => p.trim());
    const modifiers = {
        ctrl: parts.includes('Ctrl'),
        alt: parts.includes('Alt'),
        shift: parts.includes('Shift'),
        meta: parts.includes('Super') || parts.includes('Command') || parts.includes('Meta'),
    };

    if (e.ctrlKey !== modifiers.ctrl) return false;
    if (e.altKey !== modifiers.alt) return false;
    if (e.shiftKey !== modifiers.shift) return false;
    if (e.metaKey !== modifiers.meta) return false;

    const mainKey = parts.find(p => !['Ctrl', 'Alt', 'Shift', 'Super', 'Command', 'Meta'].includes(p));
    if (!mainKey) return false;

    // Use the same logic as recorder to get the key name
    const pressedKey = getStandardKey(e);
    
    return pressedKey.toLowerCase() === mainKey.toLowerCase();
}

export async function unregisterAllShortcuts() {
  try {
    await unregisterAll();
    
    if (localKeyHandler) {
      window.removeEventListener('keydown', localKeyHandler);
      localKeyHandler = null;
    }
    
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
