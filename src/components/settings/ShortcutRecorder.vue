<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { X, Command, AlertCircle } from 'lucide-vue-next';
import { emit as tauriEmit } from '@tauri-apps/api/event';

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
  validate?: (shortcut: string) => Promise<boolean>;
}>();

const ERROR_DISPLAY_DURATION = 3000;

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

const isRecording = ref(false);
const elementRef = ref<HTMLElement | null>(null);
const isMac = ref(false);
const pendingShortcut = ref('');
const conflictShortcut = ref('');
let conflictTimeout: number | null = null;
const errorMessage = ref('');
let errorTimeout: number | null = null;

onMounted(() => {
  isMac.value = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
});

const displayKeys = computed(() => {
  const rawValue = conflictShortcut.value || (isRecording.value && pendingShortcut.value 
    ? pendingShortcut.value 
    : props.modelValue);

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

function showError(msg: string) {
  errorMessage.value = msg;
  if (errorTimeout) clearTimeout(errorTimeout);
  errorTimeout = window.setTimeout(() => {
    errorMessage.value = '';
  }, ERROR_DISPLAY_DURATION);
}

async function startRecording() {
  isRecording.value = true;
  pendingShortcut.value = '';

  if (conflictTimeout) {
    clearTimeout(conflictTimeout);
    conflictTimeout = null;
  }
  conflictShortcut.value = '';

  errorMessage.value = '';
  elementRef.value?.focus();
  await tauriEmit('pause-global-shortcuts');
}

async function stopRecording() {
  isRecording.value = false;
  pendingShortcut.value = '';
  elementRef.value?.blur();
  await tauriEmit('resume-global-shortcuts');
}

async function clearShortcut(e: Event) {
  e.stopPropagation();
  emit('update:modelValue', '');
  // If we were recording, stop it (which handles resume)
  // If we weren't, we don't need to resume.
  if (isRecording.value) {
      await stopRecording();
  }
}

function getCurrentModifiers(e: KeyboardEvent): string[] {
  const keys: string[] = [];
  if (e.ctrlKey) keys.push('Ctrl');
  if (e.altKey) keys.push('Alt');
  if (e.shiftKey) keys.push('Shift');
  if (e.metaKey) keys.push('Super');
  return keys;
}

const CODE_MAP: Record<string, string> = {
  'Backquote': '`',
  'Minus': '-',
  'Equal': '=',
  'BracketLeft': '[',
  'BracketRight': ']',
  'Backslash': '\\',
  'Semicolon': ';',
  'Quote': "'",
  'Comma': ',',
  'Period': '.',
  'Slash': '/',
  'Space': 'Space',
  'ArrowUp': 'Up',
  'ArrowDown': 'Down',
  'ArrowLeft': 'Left',
  'ArrowRight': 'Right',
  'Tab': 'Tab',
  'Enter': 'Enter',
  'Backspace': 'Backspace',
  'Delete': 'Delete',
  'Insert': 'Insert',
  'Home': 'Home',
  'End': 'End',
  'PageUp': 'PageUp',
  'PageDown': 'PageDown',
  'Escape': 'Esc',
};

// Add F-keys
for (let i = 1; i <= 24; i++) {
  CODE_MAP[`F${i}`] = `F${i}`;
}

function getStandardKey(e: KeyboardEvent): string {
  const code = e.code;
  
  // Letters
  if (code.startsWith('Key')) {
    return code.slice(3).toUpperCase();
  }

  // Digits
  if (code.startsWith('Digit')) {
    return code.slice(5);
  }

  // Numpad
  if (code.startsWith('Numpad')) {
    if (code.length === 7 && !isNaN(Number(code[6]))) {
        return code[6];
    }
    switch (code) {
        case 'NumpadAdd': return '+';
        case 'NumpadSubtract': return '-';
        case 'NumpadMultiply': return '*';
        case 'NumpadDivide': return '/';
        case 'NumpadDecimal': return '.';
        case 'NumpadEnter': return 'Enter';
    }
  }

  // Standard Map
  if (CODE_MAP[code]) {
    return CODE_MAP[code];
  }

  // Fallback
  let key = e.key.toUpperCase();
  if (key === ' ') return 'Space';
  return key;
}

async function handleKeyDown(e: KeyboardEvent) {
  if (!isRecording.value) return;

  e.preventDefault();
  
  if (e.key === 'Escape') {
    await stopRecording();
    return;
  }

  const keys = getCurrentModifiers(e);
  const isModifier = ['Control', 'Shift', 'Alt', 'Meta'].includes(e.key);

  if (!isModifier) {
    const key = getStandardKey(e);
    keys.push(key);
  }

  const shortcut = keys.join('+');
  
  if (!isModifier && keys.length > 0) {
    // Finalize shortcut
    if (shortcut === props.modelValue) {
        await stopRecording();
        return;
    }

    if (props.validate) {
        const isValid = await props.validate(shortcut);
        if (!isValid) {
            showError('System shortcut conflict');
            // Show the conflicting shortcut temporarily
            conflictShortcut.value = shortcut;
            await stopRecording();
            
            // Reset after delay
            if (conflictTimeout) clearTimeout(conflictTimeout);
            conflictTimeout = window.setTimeout(() => {
                conflictShortcut.value = '';
                conflictTimeout = null;
            }, ERROR_DISPLAY_DURATION);
            return;
        }
    }

    emit('update:modelValue', shortcut);
    await stopRecording();
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

async function handleBlur() {
  if (isRecording.value) {
    isRecording.value = false;
    pendingShortcut.value = '';
    await tauriEmit('resume-global-shortcuts');
  }
}
</script>

<template>
  <div class="relative">
    <!-- Error Bubble -->
    <div 
        v-if="errorMessage"
        class="absolute bottom-full left-0 mb-2 z-10 flex items-center gap-1.5 px-3 py-1.5 bg-red-50 text-red-600 text-xs font-medium rounded-md border border-red-100 shadow-sm animate-in fade-in slide-in-from-bottom-1"
    >
        <AlertCircle class="w-3.5 h-3.5" />
        {{ errorMessage }}
        <!-- Arrow -->
        <div class="absolute -bottom-1 left-4 w-2 h-2 bg-red-50 border-b border-r border-red-100 rotate-45"></div>
    </div>

    <div
        ref="elementRef"
        class="relative w-full min-h-[42px] rounded-lg border transition-all duration-200 flex items-center px-3 py-2 cursor-pointer outline-none group select-none bg-white"
        :class="[
        isRecording 
            ? 'border-black ring-2 ring-black/5' 
            : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50',
        errorMessage ? 'border-red-200 bg-red-50/30' : ''
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
  </div>
</template>
