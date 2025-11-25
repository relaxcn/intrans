<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";
import { Bot, Keyboard, Languages } from "lucide-vue-next";

import AIProviderSettings from "./settings/AIProviderSettings.vue";
import ShortcutSettings from "./settings/ShortcutSettings.vue";
import TranslationSettings from "./settings/TranslationSettings.vue";

type Tab = "ai" | "shortcuts" | "translation";
const currentTab = ref<Tab>("ai");

interface SettingsState {
  provider: string;
  model: string;
  apiKey: string;
  baseUrl: string;
  toggleMainShortcut: string;
  openSettingsShortcut: string;
  targetLang: string;
  focus1: string;
  focus2: string;
  focus3: string;
}

const defaultState: SettingsState = {
  provider: "openai",
  model: "gpt-4.1",
  apiKey: "",
  baseUrl: "",
  toggleMainShortcut: "Ctrl+Alt+Space",
  openSettingsShortcut: "Ctrl+Shift+,",
  targetLang: "Simplified Chinese",
  focus1: "Literal & Accurate",
  focus2: "Professional & Formal",
  focus3: "Creative & Idiomatic",
};

const formData = ref<SettingsState>({ ...defaultState });
const savedState = ref<SettingsState>({ ...defaultState });

const hasChanges = computed(() => {
  return JSON.stringify(formData.value) !== JSON.stringify(savedState.value);
});

let settingsStore: Store | null = null;

async function ensureStore(): Promise<Store> {
  if (!settingsStore) {
    settingsStore = await Store.load("settings.json");
  }
  return settingsStore;
}

async function loadSettings() {
  try {
    const store = await ensureStore();

    const loaded: SettingsState = { ...defaultState };

    const savedProvider = await store.get<string>("provider");
    const savedModel = await store.get<string>("model");
    const savedApiKey = await store.get<string>("apiKey");
    const savedBaseUrl = await store.get<string>("baseUrl");
    const savedToggleMain = await store.get<string>("toggleMainShortcut");
    const savedOpenSettings = await store.get<string>("openSettingsShortcut");
    const savedTargetLang = await store.get<string>("targetLang");
    const savedFocus1 = await store.get<string>("focus1");
    const savedFocus2 = await store.get<string>("focus2");
    const savedFocus3 = await store.get<string>("focus3");

    if (savedProvider) loaded.provider = savedProvider;
    if (savedModel) loaded.model = savedModel;
    if (savedApiKey) loaded.apiKey = savedApiKey;
    if (savedBaseUrl) loaded.baseUrl = savedBaseUrl;
    if (savedToggleMain) loaded.toggleMainShortcut = savedToggleMain;
    if (savedOpenSettings) loaded.openSettingsShortcut = savedOpenSettings;
    if (savedTargetLang) loaded.targetLang = savedTargetLang;
    if (savedFocus1) loaded.focus1 = savedFocus1;
    if (savedFocus2) loaded.focus2 = savedFocus2;
    if (savedFocus3) loaded.focus3 = savedFocus3;

    // Update both current form and saved state
    formData.value = { ...loaded };
    savedState.value = { ...loaded };
  } catch (error) {
    console.error("Failed to load settings", error);
  }
}

onMounted(() => {
  loadSettings();
});

function closeWindow() {
  const win = getCurrentWindow();
  win.hide();
}

async function saveSettings() {
  try {
    const store = await ensureStore();
    await store.set("provider", formData.value.provider);
    await store.set("model", formData.value.model);
    await store.set("apiKey", formData.value.apiKey);
    await store.set("baseUrl", formData.value.baseUrl);
    
    await store.set("toggleMainShortcut", formData.value.toggleMainShortcut);
    await store.set("openSettingsShortcut", formData.value.openSettingsShortcut);

    await store.set("targetLang", formData.value.targetLang);
    await store.set("focus1", formData.value.focus1);
    await store.set("focus2", formData.value.focus2);
    await store.set("focus3", formData.value.focus3);

    await store.save();
    await emit("settings-changed");
    
    // Update saved state to match current form
    savedState.value = { ...formData.value };
  } catch (error) {
    console.error("Failed to save settings", error);
  }
}

async function handleApply() {
  await saveSettings();
}

async function handleOK() {
  await saveSettings();
  closeWindow();
}

async function handleCancel() {
  await loadSettings(); // Revert changes
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
          @click="currentTab = 'translation'"
          :class="[
            currentTab === 'translation' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900',
            'group w-full flex items-center px-3 py-2 text-sm font-medium rounded-md'
          ]"
        >
          <Languages class="shrink-0 -ml-1 mr-3 h-5 w-5 text-gray-500" />
          Translation
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
              {{ 
                currentTab === 'ai' ? 'AI Provider Settings' : 
                currentTab === 'translation' ? 'Translation Settings' :
                'Keyboard Shortcuts' 
              }}
            </h2>
            
            <AIProviderSettings
              v-if="currentTab === 'ai'"
              v-model:provider="formData.provider"
              v-model:model="formData.model"
              v-model:apiKey="formData.apiKey"
              v-model:baseUrl="formData.baseUrl"
            />

            <TranslationSettings
              v-if="currentTab === 'translation'"
              v-model:targetLang="formData.targetLang"
              v-model:focus1="formData.focus1"
              v-model:focus2="formData.focus2"
              v-model:focus3="formData.focus3"
            />

            <ShortcutSettings
              v-if="currentTab === 'shortcuts'"
              v-model:toggleMainShortcut="formData.toggleMainShortcut"
              v-model:openSettingsShortcut="formData.openSettingsShortcut"
            />
        </div>
      </div>

      <!-- Footer -->
      <div class="shrink-0 border-t border-gray-200 p-4 bg-white">
        <div class="max-w-2xl mx-auto flex justify-end gap-3">
          <button
            type="button"
            class="px-4 py-2 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 border border-gray-300 shadow-sm transition-colors"
            @click="handleCancel"
          >
            Cancel
          </button>
          <button
            type="button"
            class="px-4 py-2 rounded-md text-sm font-medium transition-colors border shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            :class="[
              hasChanges 
                ? 'text-gray-700 bg-white hover:bg-gray-50 border-gray-300' 
                : 'text-gray-400 bg-gray-50 border-gray-200 cursor-not-allowed'
            ]"
            :disabled="!hasChanges"
            @click="handleApply"
          >
            Apply
          </button>
          <button
            type="button"
            class="px-4 py-2 rounded-md text-sm font-medium text-white bg-black hover:bg-gray-800 border border-transparent shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900 transition-colors"
            @click="handleOK"
          >
            OK
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
