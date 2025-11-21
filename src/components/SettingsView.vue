<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";
import { Bot, Keyboard } from "lucide-vue-next";

import AIProviderSettings from "./settings/AIProviderSettings.vue";
import ShortcutSettings from "./settings/ShortcutSettings.vue";

type Tab = "ai" | "shortcuts";
const currentTab = ref<Tab>("ai");

// AI Settings
const provider = ref("openai");
const model = ref("gpt-4.1");
const apiKey = ref("");
const baseUrl = ref("");

// Shortcut Settings
const toggleMainShortcut = ref("Ctrl+Alt+Space");
const openSettingsShortcut = ref("Ctrl+Shift+,");

let settingsStore: Store | null = null;

async function ensureStore(): Promise<Store> {
  if (!settingsStore) {
    settingsStore = await Store.load("settings.json");
  }
  return settingsStore;
}

onMounted(async () => {
  try {
    const store = await ensureStore();

    const savedProvider = await store.get<string>("provider");
    const savedModel = await store.get<string>("model");
    const savedApiKey = await store.get<string>("apiKey");
    const savedBaseUrl = await store.get<string>("baseUrl");
    
    const savedToggleMain = await store.get<string>("toggleMainShortcut");
    const savedOpenSettings = await store.get<string>("openSettingsShortcut");

    if (savedProvider) provider.value = savedProvider;
    if (savedModel) model.value = savedModel;
    if (savedApiKey) apiKey.value = savedApiKey;
    if (savedBaseUrl) baseUrl.value = savedBaseUrl;
    
    if (savedToggleMain) toggleMainShortcut.value = savedToggleMain;
    if (savedOpenSettings) openSettingsShortcut.value = savedOpenSettings;
  } catch (error) {
    console.error("Failed to load settings", error);
  }
});

function closeWindow() {
  const win = getCurrentWindow();
  win.hide();
}

async function saveSettings() {
  try {
    const store = await ensureStore();
    await store.set("provider", provider.value);
    await store.set("model", model.value);
    await store.set("apiKey", apiKey.value);
    await store.set("baseUrl", baseUrl.value);
    
    await store.set("toggleMainShortcut", toggleMainShortcut.value);
    await store.set("openSettingsShortcut", openSettingsShortcut.value);

    await store.save();
    await emit("settings-changed");
  } catch (error) {
    console.error("Failed to save settings", error);
  }

  closeWindow();
}
</script>

<template>
  <div class="h-screen w-screen bg-gray-50 flex overflow-hidden">
    <!-- Sidebar -->
    <div class="w-64 shrink-0 bg-white border-r border-gray-200 flex flex-col">
      <div class="h-16 flex items-center px-6 border-b border-gray-100">
        <h1 class="text-lg font-semibold text-gray-900">Intrans Settings</h1>
      </div>
      
      <nav class="flex-1 p-4 space-y-1">
        <button
          @click="currentTab = 'ai'"
          :class="[
            currentTab === 'ai' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900',
            'group w-full flex items-center px-3 py-2 text-sm font-medium rounded-md'
          ]"
        >
          <Bot class="shrink-0 -ml-1 mr-3 h-5 w-5 text-gray-500" />
          AI Provider
        </button>

        <button
          @click="currentTab = 'shortcuts'"
          :class="[
            currentTab === 'shortcuts' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900',
            'group w-full flex items-center px-3 py-2 text-sm font-medium rounded-md'
          ]"
        >
          <Keyboard class="shrink-0 -ml-1 mr-3 h-5 w-5 text-gray-500" />
          Shortcuts
        </button>
      </nav>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
      <div class="flex-1 overflow-y-auto p-8">
        <div class="max-w-2xl mx-auto">
            <h2 class="text-lg font-medium text-gray-900 mb-6">
              {{ currentTab === 'ai' ? 'AI Provider Settings' : 'Keyboard Shortcuts' }}
            </h2>
            
            <AIProviderSettings
              v-if="currentTab === 'ai'"
              v-model:provider="provider"
              v-model:model="model"
              v-model:apiKey="apiKey"
              v-model:baseUrl="baseUrl"
            />

            <ShortcutSettings
              v-if="currentTab === 'shortcuts'"
              v-model:toggleMainShortcut="toggleMainShortcut"
              v-model:openSettingsShortcut="openSettingsShortcut"
            />
        </div>
      </div>

      <!-- Footer -->
      <div class="shrink-0 border-t border-gray-200 p-4 bg-white">
        <div class="max-w-2xl mx-auto flex justify-end gap-3">
          <button
            type="button"
            class="px-4 py-2 rounded-md text-sm text-gray-600 hover:bg-gray-100 border border-transparent"
            @click="closeWindow"
          >
            Cancel
          </button>
          <button
            type="button"
            class="px-4 py-2 rounded-md text-sm font-medium text-white bg-black hover:bg-gray-800 border border-transparent"
            @click="saveSettings"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
