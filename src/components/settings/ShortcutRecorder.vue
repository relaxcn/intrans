<script setup lang="ts">
import { ref, computed } from 'vue';
import { X, Command } from 'lucide-vue-next';

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

const isRecording = ref(false);
const elementRef = ref<HTMLElement | null>(null);

const displayKeys = computed(() => {
  if (!props.modelValue) return [];
  return props.modelValue.split('+').map(k => k.trim());
});

function startRecording() {
  isRecording.value = true;
  elementRef.value?.focus();
}

function stopRecording() {
  isRecording.value = false;
  elementRef.value?.blur();
}

function clearShortcut(e: Event) {
  e.stopPropagation();
  emit('update:modelValue', '');
  stopRecording();
}

function handleKeyDown(e: KeyboardEvent) {
  if (!isRecording.value) return;

  e.preventDefault();
  
  // Allow Escape to cancel recording without changes
  if (e.key === 'Escape') {
    stopRecording();
    return;
  }

  // Don't update if only modifiers are pressed
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    return;
  }

  const keys: string[] = [];
  
  if (e.ctrlKey) keys.push('Ctrl');
  if (e.shiftKey) keys.push('Shift');
  if (e.altKey) keys.push('Alt');
  if (e.metaKey) keys.push('Super');

  let key = e.key.toUpperCase();
  
  // Normalize special keys
  if (key === ' ') key = 'Space';
  if (e.code === 'Space') key = 'Space'; // Fallback
  
  // Fix for special characters if needed, but e.key is usually good enough for display
  // Map some symbols to names if strictly needed, but "COMMA" vs "," is preference.
  // User used "Ctrl+Shift+," so "," is fine.

  keys.push(key);

  const shortcut = keys.join('+');
  emit('update:modelValue', shortcut);
  stopRecording();
}

function handleBlur() {
  // Optional: delay stop to allow clicks to process
  // isRecording.value = false; 
  // Actually, we might want to keep recording until a key is pressed or user clicks away explicitly.
  // But simpler is to stop on blur.
  if (isRecording.value) {
    isRecording.value = false;
  }
}
</script>

<template>
  <div
    ref="elementRef"
    class="relative w-full min-h-[42px] rounded-md border transition-all duration-200 flex items-center px-3 py-2 cursor-pointer outline-none group select-none"
    :class="[
      isRecording 
        ? 'border-black ring-2 ring-black/10 bg-white' 
        : 'border-gray-300 bg-white hover:border-gray-400'
    ]"
    tabindex="0"
    @click="startRecording"
    @keydown="handleKeyDown"
    @blur="handleBlur"
  >
    <!-- Placeholder / Recording State -->
    <div v-if="isRecording && displayKeys.length === 0" class="text-sm text-gray-500 flex items-center gap-2">
      <Command class="w-4 h-4 animate-pulse" />
      <span>Press shortcut keys...</span>
    </div>

    <div v-else-if="displayKeys.length === 0" class="text-sm text-gray-400">
      {{ placeholder || 'Click to record shortcut' }}
    </div>

    <!-- Keys Display -->
    <div v-else class="flex flex-wrap gap-1.5">
      <template v-if="isRecording">
         <span class="text-sm text-gray-500 mr-2">Recording...</span>
      </template>
      <kbd
        v-for="(key, index) in displayKeys"
        :key="index"
        class="hidden sm:inline-block px-2 py-0.5 rounded border border-gray-200 bg-gray-100 text-gray-600 text-xs font-medium font-mono shadow-sm"
      >
        {{ key }}
      </kbd>
    </div>

    <!-- Clear Button -->
    <button
      v-if="modelValue && !isRecording"
      type="button"
      class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-full transition-colors"
      @click="clearShortcut"
      title="Clear shortcut"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
</template>
