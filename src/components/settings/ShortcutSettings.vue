<script setup lang="ts">
import ShortcutRecorder from './ShortcutRecorder.vue';
import { validateShortcut } from '../../services/shortcutService';

interface Props {
  toggleMainShortcut: string;
  openSettingsShortcut: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:toggleMainShortcut', value: string): void;
  (e: 'update:openSettingsShortcut', value: string): void;
}>();
</script>

<template>
  <div class="space-y-6">
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Toggle Main Window</label>
      <ShortcutRecorder
        :model-value="toggleMainShortcut"
        @update:model-value="emit('update:toggleMainShortcut', $event)"
        :validate="validateShortcut"
        placeholder="Click to record shortcut"
      />
      <p class="text-xs text-gray-500 mt-1">Global shortcut to show/hide the translation window.</p>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Open Settings</label>
      <ShortcutRecorder
        :model-value="openSettingsShortcut"
        @update:model-value="emit('update:openSettingsShortcut', $event)"
        :validate="validateShortcut"
        placeholder="Click to record shortcut"
      />
      <p class="text-xs text-gray-500 mt-1">Global shortcut to open this settings panel.</p>
    </div>
  </div>
</template>
