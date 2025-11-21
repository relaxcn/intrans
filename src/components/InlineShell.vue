<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from "vue";
import { Image, Mic, ArrowUp } from "lucide-vue-next";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  options?: string[];
  selectedOptionIndex?: number;
  timestamp: number;
}

interface Session {
  id: string;
  messages: Message[];
}

const input = ref("");
const sessions = ref<Session[]>([
  { id: `${Date.now()}`, messages: [] },
]);
const activeSessionIndex = ref(0);
const messages = computed(() => {
  const session = sessions.value[activeSessionIndex.value];
  return session ? session.messages : [];
});
const isExpanded = ref(false);
const provider = ref("openai");
const modelInput = ref("gpt-4.1");

const INITIAL_WIDTH = 700;
const INITIAL_HEIGHT = 120;
const MAX_HEIGHT = 600;

let settingsStore: Store | null = null;

async function ensureStore(): Promise<Store> {
  if (!settingsStore) {
    settingsStore = await Store.load("settings.json");
  }
  return settingsStore;
}


function getLiveSession(): Session {
  const lastIndex = sessions.value.length - 1;
  return sessions.value[lastIndex];
}

function ensureViewingLiveSession() {
  activeSessionIndex.value = sessions.value.length - 1;
}

async function loadSettings() {
  try {
    const store = await ensureStore();

    const savedProvider = await store.get<string>("provider");
    const savedModel = await store.get<string>("model");

    if (savedProvider) provider.value = savedProvider;
    if (savedModel) {
      modelInput.value = savedModel;
    }
  } catch (error) {
    console.error("Failed to load settings in inline shell", error);
  }
}

async function resizeWindow(expanded: boolean) {
  isExpanded.value = expanded;
  const appWindow = getCurrentWindow();

  if (!expanded) {
    await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, INITIAL_HEIGHT));
    return;
  }

  await nextTick();
  const container = document.getElementById("chat-container");
  const inputArea = document.getElementById("input-area");

  if (container && inputArea) {
    // 计算所需高度：内容高度 + 输入框区域高度 + 边框/Padding修正
    const contentHeight = container.scrollHeight + inputArea.offsetHeight + 4; // +4 for border/padding safety
    const newHeight = Math.min(Math.max(contentHeight, INITIAL_HEIGHT), MAX_HEIGHT);
    await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, newHeight));
  } else {
    // Fallback if elements not found
    await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, MAX_HEIGHT));
  }
}

watch(
  messages,
  async () => {
    if (isExpanded.value) {
      await resizeWindow(true);
      scrollToBottom();
    }
  },
  { deep: true }
);
 
let unlistenSettingsChanged: null | (() => void) = null;
let unlistenSessionEnded: null | (() => void) = null;

onMounted(async () => {
  await resizeWindow(false);

  await loadSettings();

  try {
    unlistenSettingsChanged = await listen("settings-changed", async () => {
      await loadSettings();
    });
  } catch (error) {
    console.error("Failed to listen for settings changes", error);
  }

  try {
    unlistenSessionEnded = await listen("session-ended", async () => {
      await handleSessionEnd();
    });
  } catch (error) {
    console.error("Failed to listen for session end", error);
  }
});

onBeforeUnmount(() => {
  if (unlistenSettingsChanged) {
    unlistenSettingsChanged();
    unlistenSettingsChanged = null;
  }

  if (unlistenSessionEnded) {
    unlistenSessionEnded();
    unlistenSessionEnded = null;
  }
});

function handleSend() {
  if (!input.value.trim()) return;

  ensureViewingLiveSession();
  const session = getLiveSession();

  const userMsg: Message = {
    id: Date.now().toString(),
    role: "user",
    content: input.value,
    timestamp: Date.now(),
  };

  session.messages.push(userMsg);
  input.value = "";

  if (!isExpanded.value) {
    resizeWindow(true);
  }

  setTimeout(() => {
    const aiMsg: Message = {
      id: (Date.now() + 1).toString(),
      role: "assistant",
      content: "Here are some translation options:",
      options: [
        "Translation Option 1: This is the first possible translation.",
        "Translation Option 2: A second, perhaps more formal translation.",
        "Translation Option 3: A creative adaptation of the text.",
      ],
      timestamp: Date.now(),
    };
    session.messages.push(aiMsg);
    scrollToBottom();
  }, 600);

  scrollToBottom();
}

function scrollToBottom() {
  nextTick(() => {
    const container = document.getElementById("chat-container");
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  });
}

function selectOption(messageIndex: number, optionIndex: number) {
  const session = sessions.value[activeSessionIndex.value];
  if (!session) return;

  const msg = session.messages[messageIndex];
  if (msg && msg.options) {
    msg.selectedOptionIndex = optionIndex;
    console.log("Selected option:", msg.options[optionIndex]);
  }
}

function navigateHistory(direction: -1 | 1) {
  const total = sessions.value.length;
  if (total <= 1) return;

  if (direction === -1) {
    if (activeSessionIndex.value > 0) {
      activeSessionIndex.value -= 1;
    }
  } else {
    if (activeSessionIndex.value < total - 1) {
      activeSessionIndex.value += 1;
    }
  }

  // 如果切换到的会话有内容，自动展开窗口
  const currentSession = sessions.value[activeSessionIndex.value];
  if (currentSession && currentSession.messages.length > 0 && !isExpanded.value) {
    resizeWindow(true);
  }

  nextTick(() => {
    const container = document.getElementById("chat-container");
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  });
}

async function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    await handleSessionEnd();
    const appWindow = getCurrentWindow();
    await appWindow.hide();
    return;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    handleSend();
    return;
  }

  if (
    event.key === "," &&
    event.ctrlKey &&
    !event.shiftKey &&
    !event.altKey &&
    !event.metaKey
  ) {
    event.preventDefault();
    navigateHistory(-1);
    return;
  }

  if (
    event.key === "." &&
    event.ctrlKey &&
    !event.shiftKey &&
    !event.altKey &&
    !event.metaKey
  ) {
    event.preventDefault();
    navigateHistory(1);
    return;
  }
}

async function handleSessionEnd() {
  const liveSession = getLiveSession();

  if (liveSession.messages.length === 0) {
    // 没有对话内容，不需要新开会话
    return;
  }

  sessions.value.push({
    id: `${Date.now()}-${sessions.value.length}`,
    messages: [],
  });
  activeSessionIndex.value = sessions.value.length - 1;

  input.value = "";
  isExpanded.value = false;
  await resizeWindow(false);
}
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-transparent p-px" data-tauri-drag-region>
    <div
      class="w-full h-full bg-white rounded-xl border border-gray-200 shadow-sm flex flex-col transition-all duration-200 ease-in-out relative overflow-hidden"
    >
      <div
        v-if="isExpanded"
        id="chat-container"
        class="flex-1 overflow-y-auto p-4 space-y-4 bg-gray-50/50"
      >
        <div v-for="(msg, idx) in messages" :key="msg.id" class="flex flex-col gap-2">
          <div v-if="msg.role === 'user'" class="flex justify-end">
            <div
              class="bg-gray-100 text-gray-900 px-4 py-2.5 rounded-2xl rounded-tr-sm max-w-[80%] text-sm border border-gray-200"
            >
              {{ msg.content }}
            </div>
          </div>

          <div v-else class="flex flex-col gap-2 w-full">
            <div v-if="msg.options" class="grid gap-1">
              <button
                v-for="(opt, optIdx) in msg.options"
                :key="optIdx"
                class="text-left px-2 py-2 rounded-lg transition-all duration-200 text-sm group hover:bg-gray-50 flex items-start gap-3"
                @click="selectOption(idx, optIdx)"
              >
                <span
                  class="shrink-0 w-6 h-6 flex items-center justify-center rounded-md text-xs font-medium border transition-colors mt-0.5"
                  :class="
                    msg.selectedOptionIndex === optIdx
                      ? 'bg-gray-900 text-white border-gray-900'
                      : 'bg-white border-gray-400 text-gray-600 group-hover:border-gray-600'
                  "
                >
                  {{ optIdx + 1 }}
                </span>
                <span 
                  class="text-gray-700 leading-relaxed"
                  :class="msg.selectedOptionIndex === optIdx ? 'font-medium text-gray-900' : ''"
                >
                  {{ opt }}
                </span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-white z-10 border-t border-gray-100 mt-auto" id="input-area">
        <div class="p-4 pb-2">
          <textarea
            v-model="input"
            placeholder="What would you like to translate?"
            class="w-full resize-none outline-none text-gray-700 placeholder-gray-400 text-base bg-transparent max-h-32 overflow-y-auto"
            rows="1"
            @keydown="handleKeydown"
          ></textarea>
        </div>

        <div class="flex justify-between items-center px-3 pb-3 pt-1">
          <div class="flex items-center gap-3">
            <button
              class="p-1.5 text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
              title="Upload image"
            >
              <Image class="w-5 h-5" />
            </button>
            <button
              class="p-1.5 text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
              title="Voice input"
            >
              <Mic class="w-5 h-5" />
            </button>
          </div>

          <button
            class="p-2 rounded-full transition-colors flex items-center justify-center"
            :class="
              input
                ? 'bg-gray-800 text-white hover:bg-gray-700'
                : 'bg-gray-200 text-gray-400 cursor-not-allowed'
            "
            :disabled="!input"
            @click="handleSend"
          >
            <ArrowUp class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
