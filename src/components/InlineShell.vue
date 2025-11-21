<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from "vue";
import { Plus, Send, ChevronDown } from "lucide-vue-next";
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
const models = ref<string[]>([]);
const selectedModel = ref("gpt-4.1");
const showModelMenu = ref(false);

const INITIAL_WIDTH = 700;
const INITIAL_HEIGHT = 120;
const EXPANDED_HEIGHT = 600;

let settingsStore: Store | null = null;

async function ensureStore(): Promise<Store> {
  if (!settingsStore) {
    settingsStore = await Store.load("settings.json");
  }
  return settingsStore;
}

function parseModels(raw: string): string[] {
  return raw
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

function syncModelsFromInput() {
  const list = parseModels(modelInput.value);
  models.value = list;
  if (!selectedModel.value || (list.length > 0 && !list.includes(selectedModel.value))) {
    selectedModel.value = list[0] ?? modelInput.value;
  }
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

    syncModelsFromInput();
  } catch (error) {
    console.error("Failed to load settings in inline shell", error);
  }
}

async function resizeWindow(expanded: boolean) {
  isExpanded.value = expanded;
  const appWindow = getCurrentWindow();
  await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, expanded ? EXPANDED_HEIGHT : INITIAL_HEIGHT));
}
 
function toggleModelMenu() {
  showModelMenu.value = !showModelMenu.value;
}

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

function selectModel(id: string) {
  selectedModel.value = id;
  showModelMenu.value = false;
}
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-transparent p-px" data-tauri-drag-region>
    <div
      class="w-full h-full bg-white rounded-xl border border-gray-200 shadow-sm flex flex-col transition-all duration-200 ease-in-out relative"
    >
      <div
        v-if="isExpanded"
        id="chat-container"
        class="flex-1 overflow-y-auto p-4 space-y-4 bg-gray-50/50"
      >
        <div v-for="(msg, idx) in messages" :key="msg.id" class="flex flex-col gap-2">
          <div v-if="msg.role === 'user'" class="flex justify-end">
            <div
              class="bg-blue-600 text-white px-3 py-2 rounded-lg rounded-tr-none max-w-[80%] text-sm"
            >
              {{ msg.content }}
            </div>
          </div>

          <div v-else class="flex flex-col gap-2 max-w-[90%]">
            <div class="text-xs text-gray-400 ml-1">AI Assistant</div>

            <div v-if="msg.options" class="grid gap-2">
              <button
                v-for="(opt, optIdx) in msg.options"
                :key="optIdx"
                class="text-left p-3 rounded-lg border transition-all duration-200 text-sm group hover:shadow-md"
                :class="
                  msg.selectedOptionIndex === optIdx
                    ? 'border-blue-500 bg-blue-50 text-blue-700'
                    : 'border-gray-200 bg-white hover:border-blue-300'
                "
                @click="selectOption(idx, optIdx)"
              >
                <div class="flex items-start gap-2">
                  <span
                    class="shrink-0 w-5 h-5 flex items-center justify-center rounded-full text-xs font-medium border"
                    :class="
                      msg.selectedOptionIndex === optIdx
                        ? 'border-blue-200 bg-blue-100'
                        : 'border-gray-200 bg-gray-50 text-gray-500'
                    "
                  >
                    {{ optIdx + 1 }}
                  </span>
                  <span>{{ opt }}</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-white z-10 border-t border-gray-100 mt-auto">
        <div class="p-4 pb-2">
          <textarea
            v-model="input"
            placeholder="Ask anything"
            class="w-full resize-none outline-none text-gray-700 placeholder-gray-400 text-base bg-transparent max-h-32 overflow-y-auto"
            rows="1"
            @keydown="handleKeydown"
          ></textarea>
        </div>

        <div class="flex justify-between items-center px-3 pb-3 pt-1">
          <button
            class="flex items-center gap-1.5 px-2 py-1 text-gray-500 hover:bg-gray-100 rounded-md text-sm transition-colors group"
          >
            <div class="p-0.5 bg-gray-100 group-hover:bg-gray-200 rounded-sm">
              <Plus class="w-3 h-3" />
            </div>
            <span>Add repositories...</span>
          </button>

          <div class="flex items-center gap-2">
            <div class="relative">
              <button
                class="flex items-center gap-1 text-gray-500 hover:text-gray-700 text-sm font-medium px-2 py-1 rounded-md hover:bg-gray-100 transition-colors"
                @click="toggleModelMenu"
              >
                <span>{{ selectedModel }}</span>
                <ChevronDown class="w-3 h-3" />
              </button>

              <div
                v-if="showModelMenu && models.length > 0"
                class="absolute right-0 bottom-full mb-1 w-44 bg-white border border-gray-200 rounded-md shadow-lg z-20"
              >
                <button
                  v-for="m in models"
                  :key="m"
                  class="block w-full text-left px-3 py-1.5 text-xs text-gray-700 hover:bg-gray-100"
                  :class="m === selectedModel ? 'bg-gray-100 font-medium' : ''"
                  @click="selectModel(m)"
                >
                  {{ m }}
                </button>
              </div>
            </div>

            <button
              class="p-1.5 rounded-md transition-colors flex items-center justify-center"
              :class="
                input
                  ? 'bg-black text-white hover:bg-gray-800'
                  : 'bg-gray-100 text-gray-400 cursor-not-allowed'
              "
              :disabled="!input"
              @click="handleSend"
            >
              <Send class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
