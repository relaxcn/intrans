<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from "vue";
import { Image, Mic, ArrowUp, ChevronDown } from "lucide-vue-next";
import { getCurrentWindow, LogicalPosition, LogicalSize } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { LlmService } from "../services/llmService";
import { LANGUAGES } from "../constants/languages";

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
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const sessions = ref<Session[]>([
  { id: `${Date.now()}`, messages: [] },
]);
const activeSessionIndex = ref(0);
const messages = computed(() => {
  const session = sessions.value[activeSessionIndex.value];
  return session ? session.messages : [];
});
const isExpanded = ref(false);
const isThinking = ref(false);
const provider = ref("openai");
const modelInput = ref("gpt-4.1");
const targetLang = ref("Simplified Chinese");
const focusStyles = ref(["Literal & Accurate", "Professional & Formal", "Creative & Idiomatic"]);

const containerStyle = ref<Record<string, string>>({
  height: '100%',
});

const INITIAL_WIDTH = 700;
const INITIAL_HEIGHT = 120;
const WINDOW_MOVE_STEP = 40;
const MIN_EXPANDED_HEIGHT = 200; // 展开时的最小高度

let settingsStore: Store | null = null;

async function ensureStore(): Promise<Store> {
  if (!settingsStore) {
    settingsStore = await Store.load("settings.json");
  }
  return settingsStore;
}

async function moveWindowBy(deltaX: number, deltaY: number) {
  const appWindow = getCurrentWindow();

  try {
    const factor = await appWindow.scaleFactor();
    const currentPosition = await appWindow.outerPosition();
    const logicalPos = currentPosition.toLogical(factor);
    
    const newX = Math.max(0, logicalPos.x + deltaX);
    const newY = Math.max(0, logicalPos.y + deltaY);

    await appWindow.setPosition(new LogicalPosition(newX, newY));
  } catch (error) {
    console.error("Failed to move window", error);
  }
}

function getLiveSession(): Session {
  const lastIndex = sessions.value.length - 1;
  return sessions.value[lastIndex];
}

async function loadSettings() {
  try {
    const store = await ensureStore();

    const savedProvider = await store.get<string>("provider");
    const savedModel = await store.get<string>("model");
    const savedTargetLang = await store.get<string>("targetLang");
    const savedFocus1 = await store.get<string>("focus1");
    const savedFocus2 = await store.get<string>("focus2");
    const savedFocus3 = await store.get<string>("focus3");

    if (savedProvider) provider.value = savedProvider;
    if (savedModel) {
      modelInput.value = savedModel;
    }
    if (savedTargetLang) targetLang.value = savedTargetLang;
    
    const styles = [];
    if (savedFocus1) styles.push(savedFocus1);
    if (savedFocus2) styles.push(savedFocus2);
    if (savedFocus3) styles.push(savedFocus3);
    if (styles.length === 3) focusStyles.value = styles;

  } catch (error) {
    console.error("Failed to load settings in inline shell", error);
  }
}

/**
 * 计算最后一轮对话的高度（用户消息 + 助手回复）
 */
function getLastRoundHeight(): number {
  const chatContent = document.getElementById("chat-content");
  if (!chatContent) return 0;

  const messageGroups = chatContent.querySelectorAll(':scope > div > .flex');
  if (messageGroups.length === 0) return 0;

  // 找到最后一轮对话：最后一个用户消息 + 其后的助手回复
  let lastUserIndex = -1;
  for (let i = messageGroups.length - 1; i >= 0; i--) {
    const el = messageGroups[i];
    if (el.classList.contains('justify-end')) {
      lastUserIndex = i;
      break;
    }
  }

  if (lastUserIndex === -1) {
    // 没有用户消息，取最后两个元素
    const startIdx = Math.max(0, messageGroups.length - 2);
    let height = 0;
    for (let i = startIdx; i < messageGroups.length; i++) {
      height += (messageGroups[i] as HTMLElement).offsetHeight;
    }
    return height;
  }

  // 计算最后一轮对话高度：用户消息 + 后续所有助手回复
  let roundHeight = 0;
  for (let i = lastUserIndex; i < messageGroups.length; i++) {
    roundHeight += (messageGroups[i] as HTMLElement).offsetHeight;
  }

  return roundHeight;
}

async function resizeWindow(expanded: boolean) {
  const appWindow = getCurrentWindow();
  const factor = await appWindow.scaleFactor();
  const size = await appWindow.outerSize();
  const currentHeight = size.toLogical(factor).height;

  if (expanded) {
    isExpanded.value = true;
    await nextTick();
    const content = document.getElementById("chat-content");
    const inputArea = document.getElementById("input-area");

    if (content && inputArea) {
      // 计算最后一轮对话的高度作为最大高度限制
      const lastRoundHeight = getLastRoundHeight();
      const padding = 32; // chat-content 的 padding (p-4 = 16px * 2)
      const gap = 16; // space-y-4 的间距
      const maxHeight = lastRoundHeight + padding + gap + inputArea.offsetHeight + 4;

      const totalContentHeight = content.scrollHeight + inputArea.offsetHeight + 4;
      // 使用最后一轮对话高度作为上限，但至少要有 MIN_EXPANDED_HEIGHT
      const effectiveMaxHeight = Math.max(maxHeight, MIN_EXPANDED_HEIGHT);
      const targetHeight = Math.min(Math.max(totalContentHeight, INITIAL_HEIGHT), effectiveMaxHeight);

      if (targetHeight > currentHeight) {
        // Expand: Resize window first (transparent area increases), then animate UI
        containerStyle.value = {
          height: `${currentHeight}px`,
          transition: 'none',
          overflow: 'hidden',
        };
        
        await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, targetHeight));
        
        // Force layout reflow
        document.body.offsetHeight; 

        requestAnimationFrame(() => {
          containerStyle.value = {
            height: `${targetHeight}px`,
            transition: 'height 0.3s cubic-bezier(0.25, 0.8, 0.25, 1)',
            overflow: 'hidden',
          };
          
          setTimeout(() => {
            containerStyle.value = { height: '100%' };
          }, 300);
        });
      } else if (targetHeight < currentHeight) {
         // Shrink logic if needed when expanded (e.g. content removed)
         // For now, assume mostly expanding when active, handled same as below or simple resize
         await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, targetHeight));
      }
    }
  } else {
    // Collapse: Animate UI shrink first, then resize window
    containerStyle.value = {
      height: `${currentHeight}px`,
      transition: 'none',
      overflow: 'hidden',
    };

    // Force reflow
    document.body.offsetHeight;

    requestAnimationFrame(() => {
      containerStyle.value = {
        height: `${INITIAL_HEIGHT}px`,
        transition: 'height 0.3s cubic-bezier(0.25, 0.8, 0.25, 1)',
        overflow: 'hidden',
      };
    });

    setTimeout(async () => {
      await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, INITIAL_HEIGHT));
      isExpanded.value = false;
      containerStyle.value = { height: '100%' };
    }, 300);
  }
}

watch(
  messages,
  async (newMessages) => {
    if (newMessages && newMessages.length > 0) {
      const lastMsg = newMessages[newMessages.length - 1];
      // 只有当最后一条消息是 assistant 回复时，才重新计算窗口高度
      // 用户消息发送后仅滚动到底部，不调整窗口大小
      if (lastMsg.role === 'assistant') {
        await resizeWindow(true);
      }
      scrollToBottom();
    } else {
      if (isExpanded.value) {
        await resizeWindow(false);
      }
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

  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("focus", handleWindowFocus);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("focus", handleWindowFocus);

  if (unlistenSettingsChanged) {
    unlistenSettingsChanged();
    unlistenSettingsChanged = null;
  }

  if (unlistenSessionEnded) {
    unlistenSessionEnded();
    unlistenSessionEnded = null;
  }
});

async function handleSend() {
  if (!input.value.trim() || isThinking.value) return;

  // 将当前会话移动到末尾
  const currentIndex = activeSessionIndex.value;
  if (currentIndex < sessions.value.length - 1) {
    const currentSession = sessions.value[currentIndex];
    sessions.value.splice(currentIndex, 1);
    sessions.value.push(currentSession);
    activeSessionIndex.value = sessions.value.length - 1;
  }

  const session = sessions.value[activeSessionIndex.value];

  const userMsg: Message = {
    id: Date.now().toString(),
    role: "user",
    content: input.value,
    timestamp: Date.now(),
  };

  session.messages.push(userMsg);
  input.value = "";
  isThinking.value = true;

  if (!isExpanded.value) {
    resizeWindow(true);
  }
  scrollToBottom();

  try {
    const options = await LlmService.getInstance().translate(
      userMsg.content,
      targetLang.value,
      focusStyles.value
    );

    const aiMsg: Message = {
      id: (Date.now() + 1).toString(),
      role: "assistant",
      content: "Here are some translation options:",
      options: options,
      timestamp: Date.now(),
    };
    session.messages.push(aiMsg);
  } catch (error) {
    console.error("Translation failed", error);
    const errorMsg: Message = {
      id: (Date.now() + 1).toString(),
      role: "assistant",
      content: "Sorry, translation failed. Please check your API key and settings.",
      timestamp: Date.now(),
    };
    session.messages.push(errorMsg);
  } finally {
    isThinking.value = false;
    scrollToBottom();
  }
}

function scrollToBottom() {
  nextTick(() => {
    const container = document.getElementById("chat-container");
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  });
}

async function selectOption(messageIndex: number, optionIndex: number) {
  const session = sessions.value[activeSessionIndex.value];
  if (!session) return;

  const msg = session.messages[messageIndex];
  if (msg && msg.options) {
    msg.selectedOptionIndex = optionIndex;
    const textToCopy = msg.options[optionIndex];
    try {
      await writeText(textToCopy);
      console.log("Copied to clipboard:", textToCopy);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
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
  // 这一步交给 watch(messages) 来处理，确保逻辑统一
  // 仅仅处理滚动
  nextTick(() => {
    const container = document.getElementById("chat-container");
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  });
}

function handleWindowFocus() {
  nextTick(() => {
    textareaRef.value?.focus();
  });
}

async function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    const liveSession = getLiveSession();

    if (liveSession.messages.length > 0) {
      await handleSessionEnd();
    } else {
      const appWindow = getCurrentWindow();
      await appWindow.hide();
    }
    return;
  }
  
  const session = sessions.value[activeSessionIndex.value];
  const lastMsg = session?.messages[session.messages.length - 1];
  const hasOptions = lastMsg && lastMsg.role === 'assistant' && lastMsg.options && lastMsg.options.length > 0;

  if (event.key === "Tab" && hasOptions) {
    event.preventDefault();
    const currentIdx = lastMsg.selectedOptionIndex ?? -1;
    const nextIdx = (currentIdx + 1) % (lastMsg.options?.length || 3);
    lastMsg.selectedOptionIndex = nextIdx;
    return;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    
    if (hasOptions && typeof lastMsg.selectedOptionIndex === 'number' && lastMsg.selectedOptionIndex >= 0) {
        await selectOption(session.messages.length - 1, lastMsg.selectedOptionIndex);
        // 复制后关闭窗口
        const appWindow = getCurrentWindow();
        await appWindow.hide();
    } else {
        handleSend();
    }
    return;
  }

  if (
    event.ctrlKey &&
    !event.shiftKey &&
    !event.altKey &&
    !event.metaKey &&
    ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(event.key)
  ) {
    event.preventDefault();
    if (event.key === "ArrowUp") {
      await moveWindowBy(0, -WINDOW_MOVE_STEP);
    } else if (event.key === "ArrowDown") {
      await moveWindowBy(0, WINDOW_MOVE_STEP);
    } else if (event.key === "ArrowLeft") {
      await moveWindowBy(-WINDOW_MOVE_STEP, 0);
    } else {
      await moveWindowBy(WINDOW_MOVE_STEP, 0);
    }
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
  await resizeWindow(false);
}
</script>

<template>
  <div
    class="h-screen w-screen flex flex-col bg-transparent"
    data-tauri-drag-region
  >
    <div
      class="w-full h-full bg-white rounded-xl shadow-xl border border-gray-200 flex flex-col relative overflow-hidden"
      :style="containerStyle"
    >
      <div
        v-if="isExpanded"
        id="chat-container"
        class="flex-1 overflow-y-auto bg-gray-50/50"
      >
        <div id="chat-content" class="p-4 space-y-4">
          <div v-for="(msg, idx) in messages" :key="msg.id" class="flex flex-col gap-2">
          <div v-if="msg.role === 'user'" class="flex justify-end">
            <div
              class="bg-gray-100 text-gray-900 px-4 py-2.5 rounded-2xl rounded-tr-sm max-w-[80%] text-sm border border-gray-200 break-words whitespace-pre-wrap"
            >
              {{ msg.content }}
            </div>
          </div>

          <div v-else class="flex flex-col gap-2 w-full">
            <div v-if="msg.options" class="grid gap-1">
              <button
                v-for="(opt, optIdx) in msg.options"
                :key="optIdx"
                class="text-left px-2 py-2 rounded-lg transition-all duration-200 text-sm group flex items-start gap-3"
                :class="[
                  msg.selectedOptionIndex === optIdx
                    ? 'bg-blue-50/80 ring-2 ring-blue-400/60 shadow-[0_0_12px_rgba(59,130,246,0.4)]'
                    : 'hover:bg-gray-50'
                ]"
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
    </div>

    <div
      class="bg-white z-10 mt-auto"
      :class="{ 'border-t border-gray-100 rounded-t-xl': isExpanded }"
      id="input-area"
    >
        <div class="p-4 pb-2">
          <textarea
            ref="textareaRef"
            v-model="input"
            placeholder="What would you like to translate?"
            class="w-full resize-none outline-none text-gray-700 placeholder-gray-400 text-base bg-transparent max-h-32 overflow-y-auto"
            rows="1"
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

            <div class="h-4 w-px bg-gray-200 mx-1"></div>

            <div class="relative flex items-center group">
              <select
                v-model="targetLang"
                class="appearance-none bg-transparent text-gray-600 hover:bg-gray-100 rounded-md py-1.5 pl-2 pr-7 text-xs font-medium focus:outline-none focus:ring-0 cursor-pointer transition-colors truncate max-w-[140px]"
                title="Target Language"
              >
                <option v-for="lang in LANGUAGES" :key="lang.code" :value="lang.code">
                  {{ lang.name }}
                </option>
              </select>
              <ChevronDown class="w-3 h-3 text-gray-400 absolute right-2 pointer-events-none group-hover:text-gray-600" />
            </div>
          </div>

          <button
            class="p-2 rounded-full transition-colors flex items-center justify-center"
            :class="
              input && !isThinking
                ? 'bg-gray-800 text-white hover:bg-gray-700'
                : 'bg-gray-200 text-gray-400 cursor-not-allowed'
            "
            :disabled="!input || isThinking"
            @click="handleSend"
          >
            <ArrowUp class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
