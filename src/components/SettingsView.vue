<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";

const provider = ref("openai");
const model = ref("gpt-4.1");
const apiKey = ref("");
const baseUrl = ref("");

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

    if (savedProvider) provider.value = savedProvider;
    if (savedModel) model.value = savedModel;
    if (savedApiKey) apiKey.value = savedApiKey;
    if (savedBaseUrl) baseUrl.value = savedBaseUrl;
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
    await store.save();
    await emit("settings-changed");
  } catch (error) {
    console.error("Failed to save settings", error);
  }

  closeWindow();
}
</script>

<template>
  <div class="h-screen w-screen flex items-center justify-center bg-gray-50">
    <div class="w-full max-w-xl bg-white rounded-xl border border-gray-200 shadow-sm p-6 flex flex-col gap-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-lg font-semibold text-gray-900">Intrans Settings</h1>
          <p class="text-sm text-gray-500 mt-1">Configure your LLM provider and model.</p>
        </div>
      </div>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Provider</label>
          <select
            v-model="provider"
            class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black bg-white"
          >
            <option value="openai">OpenAI compatible</option>
            <option value="anthropic">Anthropic</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Models (comma separated)</label>
          <input
            v-model="model"
            type="text"
            placeholder="gpt-4.1, gpt-4.1-mini"
            class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">API Key</label>
          <input
            v-model="apiKey"
            type="password"
            autocomplete="off"
            class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Base URL (optional)</label>
          <input
            v-model="baseUrl"
            type="text"
            class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
          />
        </div>
      </div>

      <div class="flex justify-end gap-3 pt-2">
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
</template>
