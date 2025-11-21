<script setup lang="ts">
import { ref, onMounted } from "vue";
import InlineShell from "./components/InlineShell.vue";
import SettingsView from "./components/SettingsView.vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const currentLabel = ref("main");

onMounted(() => {
  const win = getCurrentWindow();
  currentLabel.value = win.label;
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