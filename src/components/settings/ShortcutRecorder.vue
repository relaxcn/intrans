<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
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
const isMac = ref(false);
const pendingShortcut = ref('');

onMounted(() => {
  isMac.value = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
});

const displayKeys = computed(() => {
  const rawValue = isRecording.value && pendingShortcut.value 
    ? pendingShortcut.value 
    : props.modelValue;

  if (!rawValue) return [];
  
  const keys = rawValue.split('+').map(k => k.trim());
  
  if (isMac.value) {
    return keys.map(k => {
      if (k === 'Ctrl') return '⌃';
      if (k === 'Alt') return '⌥';
      if (k === 'Shift') return '⇧';
      if (k === 'Super' || k === 'Meta' || k === 'Command') return '⌘';
      return k;
    });
  }
  
  return keys;
});

function startRecording() {
  isRecording.value = true;
  pendingShortcut.value = '';
  elementRef.value?.focus();
}

function stopRecording() {
  isRecording.value = false;
  pendingShortcut.value = '';
  elementRef.value?.blur();
}

function clearShortcut(e: Event) {
  e.stopPropagation();
  emit('update:modelValue', '');
  stopRecording();
}

function getCurrentModifiers(e: KeyboardEvent): string[] {
  const keys: string[] = [];
  if (e.ctrlKey) keys.push('Ctrl');
  if (e.altKey) keys.push('Alt');
  if (e.shiftKey) keys.push('Shift');
  if (e.metaKey) keys.push('Super');
  return keys;
}

function handleKeyDown(e: KeyboardEvent) {
  if (!isRecording.value) return;

  e.preventDefault();
  
  if (e.key === 'Escape') {
    stopRecording();
    return;
  }

  const keys = getCurrentModifiers(e);
  const isModifier = ['Control', 'Shift', 'Alt', 'Meta'].includes(e.key);

  if (!isModifier) {
    let key = e.key.toUpperCase();
    
    if (key === ' ') key = 'Space';
    if (e.code === 'Space') key = 'Space';
    
    if (key === 'ARROWUP') key = 'Up';
    if (key === 'ARROWDOWN') key = 'Down';
    if (key === 'ARROWLEFT') key = 'Left';
    if (key === 'ARROWRIGHT') key = 'Right';

    keys.push(key);
  }

  const shortcut = keys.join('+');
  
  if (!isModifier && keys.length > 0) {
    // Finalize shortcut
    emit('update:modelValue', shortcut);
    stopRecording();
  } else {
    // Update pending display for modifiers
    pendingShortcut.value = shortcut;
  }
}

function handleKeyUp(e: KeyboardEvent) {
  if (!isRecording.value) return;
  
  // Update display when modifiers are released
  const keys = getCurrentModifiers(e);
  pendingShortcut.value = keys.join('+');
}

function handleBlur() {
  if (isRecording.value) {
    isRecording.value = false;
    pendingShortcut.value = '';
  }
}
</script>

<template>
  <div
    ref="elementRef"
    class="relative w-full min-h-[42px] rounded-lg border transition-all duration-200 flex items-center px-3 py-2 cursor-pointer outline-none group select-none bg-white"
    :class="[
      isRecording 
        ? 'border-black ring-2 ring-black/5' 
        : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
    ]"
    tabindex="0"
    @click="startRecording"
    @keydown="handleKeyDown"
    @keyup="handleKeyUp"
    @blur="handleBlur"
  >
    <!-- Placeholder -->
    <div v-if="displayKeys.length === 0" class="text-sm text-gray-400 flex items-center gap-2">
      <Command class="w-4 h-4 opacity-50" />
      <span v-if="isRecording" class="text-indigo-600 font-medium animate-pulse">Press keys...</span>
      <span v-else>{{ placeholder || 'Click to record' }}</span>
    </div>

    <!-- Keys Display -->
    <div v-else class="flex flex-wrap gap-1.5 items-center">
      <template v-if="isRecording">
         <span class="text-xs font-medium text-indigo-600 uppercase tracking-wider mr-2">Recording</span>
      </template>
      <kbd
        v-for="(key, index) in displayKeys"
        :key="index"
        class="hidden sm:inline-flex items-center justify-center min-w-6 px-2 py-1 rounded-md border border-b-2 border-gray-200 bg-gray-50 text-gray-600 text-xs font-bold font-mono shadow-sm"
      >
        {{ key }}
      </kbd>
    </div>

    <!-- Clear Button -->
    <button
      v-if="modelValue && !isRecording"
      type="button"
      class="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-md transition-all opacity-0 group-hover:opacity-100"
      @click="clearShortcut"
      title="Clear shortcut"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
</template>
