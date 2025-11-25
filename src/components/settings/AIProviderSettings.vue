<script setup lang="ts">
import { ref } from "vue";
import { RefreshCw, Check, X, Loader2, Zap } from "lucide-vue-next";
import { LlmService } from "../../services/llmService";

interface Props {
  provider: string;
  model: string;
  apiKey: string;
  baseUrl: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:provider', value: string): void;
  (e: 'update:model', value: string): void;
  (e: 'update:apiKey', value: string): void;
  (e: 'update:baseUrl', value: string): void;
}>();

const availableModels = ref<string[]>([]);
const isLoadingModels = ref(false);
const isTestingConnection = ref(false);
const testStatus = ref<"idle" | "success" | "error">("idle");
const testMessage = ref("");

async function fetchModels() {
  if (!props.apiKey) {
    return;
  }
  
  isLoadingModels.value = true;
  try {
    const models = await LlmService.fetchModels(props.apiKey, props.baseUrl);
    availableModels.value = models;
  } catch (e) {
    console.error("Failed to fetch models", e);
  } finally {
    isLoadingModels.value = false;
  }
}

async function handleTestConnection() {
  if (!props.apiKey || !props.model) {
    testStatus.value = "error";
    testMessage.value = "API Key and Model are required";
    return;
  }

  isTestingConnection.value = true;
  testStatus.value = "idle";
  testMessage.value = "";

  try {
    await LlmService.testConnection(props.apiKey, props.baseUrl, props.model);
    testStatus.value = "success";
    testMessage.value = "Connection successful!";
  } catch (e: any) {
    testStatus.value = "error";
    testMessage.value = e.message || "Connection failed";
  } finally {
    isTestingConnection.value = false;
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Provider Selection -->
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Provider</label>
      <div class="relative">
        <select
          :value="provider"
          @input="emit('update:provider', ($event.target as HTMLSelectElement).value)"
          class="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-transparent bg-white shadow-sm transition-shadow"
        >
          <option value="openai">OpenAI compatible</option>
          <option value="anthropic">Anthropic</option>
        </select>
      </div>
    </div>

    <!-- API Configuration Group -->
    <div class="bg-gray-50 rounded-xl p-4 border border-gray-200 space-y-4">
        <div class="flex items-center gap-2 mb-2">
            <Zap class="w-4 h-4 text-gray-500" />
            <h3 class="text-sm font-medium text-gray-900">Connection Details</h3>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Base URL <span class="text-gray-400 font-normal">(Optional)</span></label>
          <input
            :value="baseUrl"
            @input="emit('update:baseUrl', ($event.target as HTMLInputElement).value)"
            type="text"
            placeholder="https://api.openai.com/v1"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-transparent shadow-sm placeholder-gray-400"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">API Key</label>
          <input
            :value="apiKey"
            @input="emit('update:apiKey', ($event.target as HTMLInputElement).value)"
            type="password"
            autocomplete="off"
            placeholder="sk-..."
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-transparent shadow-sm placeholder-gray-400"
          />
        </div>
    </div>

    <!-- Model Selection -->
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Model</label>
      <div class="flex gap-2">
        <div class="relative flex-1">
          <input
            :value="model"
            @input="emit('update:model', ($event.target as HTMLInputElement).value)"
            type="text"
            list="model-options"
            placeholder="Select or type model name..."
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-transparent shadow-sm"
            @keydown.enter.prevent
          />
          <datalist id="model-options">
            <option v-for="m in availableModels" :key="m" :value="m" />
          </datalist>
        </div>
        
        <button
          @click="fetchModels"
          class="px-3 py-2 bg-white border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 hover:border-gray-400 transition-colors focus:outline-none focus:ring-2 focus:ring-black focus:ring-offset-1"
          title="Fetch available models"
          :disabled="isLoadingModels || !apiKey"
        >
            <Loader2 v-if="isLoadingModels" class="w-5 h-5 animate-spin text-gray-500" />
            <RefreshCw v-else class="w-5 h-5" />
        </button>
      </div>
      <p class="mt-1.5 text-xs text-gray-500">
        Press Enter to confirm manual entry, or click the refresh button to fetch models from the provider.
      </p>
    </div>

    <!-- Test Connection -->
    <div class="pt-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm">
             <span v-if="testStatus === 'success'" class="flex items-center gap-1.5 text-green-600 font-medium">
                <Check class="w-4 h-4" />
                {{ testMessage }}
             </span>
             <span v-else-if="testStatus === 'error'" class="flex items-center gap-1.5 text-red-600 font-medium">
                <X class="w-4 h-4" />
                {{ testMessage }}
             </span>
        </div>

        <button
          @click="handleTestConnection"
          :disabled="isTestingConnection || !apiKey"
          class="flex items-center gap-2 px-4 py-2 bg-black text-white text-sm font-medium rounded-lg hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-black disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm"
        >
          <Loader2 v-if="isTestingConnection" class="w-4 h-4 animate-spin" />
          <span>{{ isTestingConnection ? 'Testing...' : 'Test Connection' }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

