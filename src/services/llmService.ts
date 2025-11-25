import { ChatOpenAI } from "@langchain/openai";
import { HumanMessage, SystemMessage } from "@langchain/core/messages";
import { Store } from "@tauri-apps/plugin-store";

export interface TranslationResult {
  options: string[];
}

export class LlmService {
  private static instance: LlmService;
  private store: Store | null = null;

  private constructor() {}

  public static getInstance(): LlmService {
    if (!LlmService.instance) {
      LlmService.instance = new LlmService();
    }
    return LlmService.instance;
  }

  private async getStore(): Promise<Store> {
    if (!this.store) {
      this.store = await Store.load("settings.json");
    }
    return this.store;
  }

  private async getModel(): Promise<ChatOpenAI> {
    const store = await this.getStore();
    const apiKey = await store.get<string>("apiKey");
    const baseUrl = await store.get<string>("baseUrl");
    const modelName = await store.get<string>("model") || "gpt-4o";

    if (!apiKey) {
      throw new Error("API Key not found. Please configure it in settings.");
    }

    const config: any = {
      apiKey,
      modelName,
      temperature: 0.7,
    };

    if (baseUrl) {
      config.configuration = {
        baseURL: baseUrl,
      };
    }

    return new ChatOpenAI(config);
  }

  public async translate(
    text: string,
    targetLang: string,
    styles: string[]
  ): Promise<string[]> {
    try {
      const model = await this.getModel();

      const stylePrompts = styles.map((style, index) => `Option ${index + 1}: ${style}`).join("\n");
      
      const systemPrompt = `You are a professional translator.
Your task is to translate the user's text into ${targetLang}.
You must provide exactly 3 distinct translation options based on the following styles/focuses:

${stylePrompts}

Return ONLY a raw JSON array of strings, with no markdown formatting, code blocks, or extra text.
Example: ["Option 1 translation", "Option 2 translation", "Option 3 translation"]`;

      const response = await model.invoke([
        new SystemMessage(systemPrompt),
        new HumanMessage(text),
      ]);

      const content = typeof response.content === "string" ? response.content : JSON.stringify(response.content);
      
      // Clean up potential markdown code blocks if the model ignores instructions
      const cleanContent = content.replace(/^```json\s*/, "").replace(/\s*```$/, "").trim();
      
      try {
        const parsed = JSON.parse(cleanContent);
        if (Array.isArray(parsed) && parsed.length === 3) {
            // Ensure they are strings
            return parsed.map(p => String(p));
        }
        // If parsing fails or length mismatch, fall back to splitting by newline if possible or return raw
        // But for now, let's throw to catch it
        if (Array.isArray(parsed)) return parsed.map(String);
      } catch (e) {
        console.warn("Failed to parse JSON response, trying heuristic split", cleanContent);
      }

      // Fallback: if not JSON, split by newlines or just return content in array
      return [cleanContent];
      
    } catch (error) {
      console.error("Translation error:", error);
      throw error;
    }
  }

  public static async fetchModels(apiKey: string, baseUrl: string): Promise<string[]> {
    try {
      // Remove trailing slash if present to avoid double slashes
      const cleanBaseUrl = baseUrl ? baseUrl.replace(/\/+$/, "") : "https://api.openai.com/v1";
      // Handle standard OpenAI compatible /models endpoint
      // Some providers use /v1/models, some just /models depending on what the user enters as base URL.
      // Standard convention: User enters "https://api.openai.com/v1", we append "/models".
      const url = `${cleanBaseUrl}/models`;
      
      const response = await fetch(url, {
        headers: {
          Authorization: `Bearer ${apiKey}`,
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch models: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();
      // Expecting OpenAI format: { data: [{ id: "..." }, ...] }
      if (data.data && Array.isArray(data.data)) {
        return data.data.map((m: any) => m.id).sort();
      }
      return [];
    } catch (error) {
      console.error("Error fetching models:", error);
      throw error;
    }
  }

  public static async testConnection(apiKey: string, baseUrl: string, modelName: string): Promise<boolean> {
    try {
      const config: any = {
        apiKey,
        modelName,
        temperature: 0,
        maxTokens: 5,
      };

      if (baseUrl) {
        config.configuration = {
          baseURL: baseUrl,
        };
      }

      const model = new ChatOpenAI(config);
      await model.invoke([new HumanMessage("Hello")]);
      return true;
    } catch (e) {
      console.error("Connection test failed:", e);
      throw e;
    }
  }
}
