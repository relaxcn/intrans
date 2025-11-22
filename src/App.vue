<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import InlineShell from "./components/InlineShell.vue";
import SettingsView from "./components/SettingsView.vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { setupGlobalShortcuts, unregisterAllShortcuts } from "./services/shortcutService";

const currentLabel = ref("main");
const unlisteners: (() => void)[] = [];

onMounted(async () => {
  const win = getCurrentWindow();
  currentLabel.value = win.label;

  if (currentLabel.value === "main") {
    await setupGlobalShortcuts();
    
    // Listen for settings changes to reload shortcuts
    unlisteners.push(await listen("settings-changed", async () => {
      console.log("Settings changed, reloading shortcuts...");
      await setupGlobalShortcuts();
    }));

    // Handle pause/resume from settings window
    unlisteners.push(await listen("pause-global-shortcuts", async () => {
        console.log("Pausing global shortcuts for recording...");
        await unregisterAllShortcuts();
    }));

    unlisteners.push(await listen("resume-global-shortcuts", async () => {
        console.log("Resuming global shortcuts...");
        await setupGlobalShortcuts();
    }));
  }
});

onUnmounted(() => {
  unlisteners.forEach(unlisten => unlisten());
});
</script>

<template>
  <InlineShell v-if="currentLabel === 'main'" />
  <SettingsView v-else-if="currentLabel === 'settings'" />
  <div v-else class="h-screen w-screen flex items-center justify-center">
    <span class="text-gray-500 text-sm">Unknown window: {{ currentLabel }}</span>
  </div>
</template>

<style>
textarea {
  field-sizing: content;
}

/* Custom Scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #e5e7eb;
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: #d1d5db;
}
</style>