<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { Plus, Send, ChevronDown } from "lucide-vue-next";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  options?: string[];
  selectedOptionIndex?: number;
  timestamp: number;
}

const input = ref("");
const messages = ref<Message[]>([]);
const historyIndex = ref(-1); // -1 means current "new" conversation or latest.
const isExpanded = ref(false);

// Window dimensions
const INITIAL_WIDTH = 700;
const INITIAL_HEIGHT = 120; // Compact height for just input
const EXPANDED_HEIGHT = 600; // Height when chat history is visible

async function resizeWindow(expanded: boolean) {
  isExpanded.value = expanded;
  const appWindow = getCurrentWindow();
  await appWindow.setSize(new LogicalSize(INITIAL_WIDTH, expanded ? EXPANDED_HEIGHT : INITIAL_HEIGHT));
}

onMounted(() => {
  // Ensure we start in compact mode
  resizeWindow(false);
});

function handleSend() {
  if (!input.value.trim()) return;

  const userMsg: Message = {
    id: Date.now().toString(),
    role: "user",
    content: input.value,
    timestamp: Date.now(),
  };

  messages.value.push(userMsg);
  input.value = "";
  
  // Expand window if not already
  if (!isExpanded.value) {
    resizeWindow(true);
  }

  // Mock AI response
  setTimeout(() => {
    const aiMsg: Message = {
      id: (Date.now() + 1).toString(),
      role: "assistant",
      content: "Here are some translation options:",
      options: [
        "Translation Option 1: This is the first possible translation.",
        "Translation Option 2: A second, perhaps more formal translation.",
        "Translation Option 3: A creative adaptation of the text."
      ],
      timestamp: Date.now(),
    };
    messages.value.push(aiMsg);
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
  const msg = messages.value[messageIndex];
  if (msg && msg.options) {
    msg.selectedOptionIndex = optionIndex;
    console.log("Selected option:", msg.options[optionIndex]);
    // TODO: In real app, this would paste to the previous window
  }
}

// History navigation mock
function navigateHistory(direction: -1 | 1) {
  // Logic to switch between conversation sessions would go here
  console.log("Navigate history:", direction);
}

</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-transparent p-2" data-tauri-drag-region>
    <div class="w-full h-full bg-white rounded-xl border border-gray-200 shadow-sm flex flex-col overflow-hidden transition-all duration-200 ease-in-out relative">
      
      <!-- Chat History Area -->
      <div 
        v-if="isExpanded"
        id="chat-container"
        class="flex-1 overflow-y-auto p-4 space-y-4 bg-gray-50/50"
      >
        <div v-for="(msg, idx) in messages" :key="msg.id" class="flex flex-col gap-2">
          
          <!-- User Message -->
          <div v-if="msg.role === 'user'" class="flex justify-end">
            <div class="bg-blue-600 text-white px-3 py-2 rounded-lg rounded-tr-none max-w-[80%] text-sm">
              {{ msg.content }}
            </div>
          </div>

          <!-- Assistant Message -->
          <div v-else class="flex flex-col gap-2 max-w-[90%]">
            <div class="text-xs text-gray-400 ml-1">AI Assistant</div>
            
            <!-- Options Grid -->
            <div v-if="msg.options" class="grid gap-2">
              <button 
                v-for="(opt, optIdx) in msg.options" 
                :key="optIdx"
                class="text-left p-3 rounded-lg border transition-all duration-200 text-sm group hover:shadow-md"
                :class="msg.selectedOptionIndex === optIdx ? 'border-blue-500 bg-blue-50 text-blue-700' : 'border-gray-200 bg-white hover:border-blue-300'"
                @click="selectOption(idx, optIdx)"
              >
                <div class="flex items-start gap-2">
                  <span class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded-full text-xs font-medium border"
                    :class="msg.selectedOptionIndex === optIdx ? 'border-blue-200 bg-blue-100' : 'border-gray-200 bg-gray-50 text-gray-500'"
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

      <!-- Input Area (Fixed at bottom) -->
      <div class="bg-white z-10 border-t border-gray-100">
        <div class="p-4 pb-2">
          <textarea
            v-model="input"
            placeholder="Ask anything"
            class="w-full resize-none outline-none text-gray-700 placeholder-gray-400 text-base bg-transparent max-h-32 overflow-y-auto"
            rows="1"
            @keydown.enter.prevent="handleSend"
            @keydown.up.prevent="navigateHistory(-1)"
            @keydown.down.prevent="navigateHistory(1)"
          ></textarea>
        </div>

        <!-- Actions Bar -->
        <div class="flex justify-between items-center px-3 pb-3 pt-1">
          <!-- Left Action -->
          <button class="flex items-center gap-1.5 px-2 py-1 text-gray-500 hover:bg-gray-100 rounded-md text-sm transition-colors group">
            <div class="p-0.5 bg-gray-100 group-hover:bg-gray-200 rounded-sm">
               <Plus class="w-3 h-3" />
            </div>
            <span>Add repositories...</span>
          </button>

          <!-- Right Actions -->
          <div class="flex items-center gap-2">
            <button class="flex items-center gap-1 text-gray-500 hover:text-gray-700 text-sm font-medium px-2 py-1 rounded-md hover:bg-gray-100 transition-colors">
              <span>GPT-4.1</span>
              <ChevronDown class="w-3 h-3" />
            </button>
            
            <button 
              class="p-1.5 rounded-md transition-colors flex items-center justify-center"
              :class="input ? 'bg-black text-white hover:bg-gray-800' : 'bg-gray-100 text-gray-400 cursor-not-allowed'"
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

<style>
textarea {
  field-sizing: content;
}

/* Custom Scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #e5e7eb;
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: #d1d5db;
}
</style>