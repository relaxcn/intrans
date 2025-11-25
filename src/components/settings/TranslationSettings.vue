<script setup lang="ts">
defineProps<{
  targetLang: string;
  focus1: string;
  focus2: string;
  focus3: string;
}>();

defineEmits<{
  (e: "update:targetLang", value: string): void;
  (e: "update:focus1", value: string): void;
  (e: "update:focus2", value: string): void;
  (e: "update:focus3", value: string): void;
}>();
</script>

<template>
  <div class="space-y-6">
    <div>
      <label class="block text-sm font-medium text-gray-700">Target Language</label>
      <div class="mt-1">
        <input
          type="text"
          :value="targetLang"
          @input="$emit('update:targetLang', ($event.target as HTMLInputElement).value)"
          placeholder="e.g. Simplified Chinese"
          class="block w-full rounded-md border-gray-300 shadow-sm focus:border-black focus:ring-black sm:text-sm p-2 border"
        />
        <p class="mt-2 text-sm text-gray-500">
          The language you want to translate into.
        </p>
      </div>
    </div>

    <div class="border-t border-gray-200 pt-6">
      <h3 class="text-md font-medium text-gray-900 mb-4">Translation Options Focus</h3>
      <p class="text-sm text-gray-500 mb-4">
        Define the style or focus for each of the 3 translation options provided by the AI.
      </p>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700">Option 1 Focus</label>
          <div class="mt-1">
            <input
              type="text"
              :value="focus1"
              @input="$emit('update:focus1', ($event.target as HTMLInputElement).value)"
              placeholder="e.g. Literal translation"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-black focus:ring-black sm:text-sm p-2 border"
            />
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700">Option 2 Focus</label>
          <div class="mt-1">
            <input
              type="text"
              :value="focus2"
              @input="$emit('update:focus2', ($event.target as HTMLInputElement).value)"
              placeholder="e.g. Professional/Formal"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-black focus:ring-black sm:text-sm p-2 border"
            />
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700">Option 3 Focus</label>
          <div class="mt-1">
            <input
              type="text"
              :value="focus3"
              @input="$emit('update:focus3', ($event.target as HTMLInputElement).value)"
              placeholder="e.g. Creative/Idiomatic"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-black focus:ring-black sm:text-sm p-2 border"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
