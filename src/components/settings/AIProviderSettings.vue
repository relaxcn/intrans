<script setup lang="ts">

interface Props {
  provider: string;
  model: string;
  apiKey: string;
  baseUrl: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:provider', value: string): void;
  (e: 'update:model', value: string): void;
  (e: 'update:apiKey', value: string): void;
  (e: 'update:baseUrl', value: string): void;
}>();
</script>

<template>
  <div class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Provider</label>
      <select
        :value="provider"
        @input="emit('update:provider', ($event.target as HTMLSelectElement).value)"
        class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black bg-white"
      >
        <option value="openai">OpenAI compatible</option>
        <option value="anthropic">Anthropic</option>
      </select>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Models (comma separated)</label>
      <input
        :value="model"
        @input="emit('update:model', ($event.target as HTMLInputElement).value)"
        type="text"
        placeholder="gpt-4.1, gpt-4.1-mini"
        class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
      />
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">API Key</label>
      <input
        :value="apiKey"
        @input="emit('update:apiKey', ($event.target as HTMLInputElement).value)"
        type="password"
        autocomplete="off"
        class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
      />
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Base URL (optional)</label>
      <input
        :value="baseUrl"
        @input="emit('update:baseUrl', ($event.target as HTMLInputElement).value)"
        type="text"
        class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
      />
    </div>
  </div>
</template>
