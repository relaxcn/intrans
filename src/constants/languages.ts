export interface Language {
  code: string;
  name: string;
}

export const LANGUAGES: Language[] = [
  { code: "English", name: "English" },
  { code: "Simplified Chinese", name: "Simplified Chinese (简体中文)" },
  { code: "Traditional Chinese", name: "Traditional Chinese (繁體中文)" },
  { code: "Japanese", name: "Japanese (日本語)" },
  { code: "Korean", name: "Korean (한국어)" },
  { code: "French", name: "French (Français)" },
  { code: "German", name: "German (Deutsch)" },
  { code: "Spanish", name: "Spanish (Español)" },
  { code: "Italian", name: "Italian (Italiano)" },
  { code: "Russian", name: "Russian (Русский)" },
  { code: "Portuguese", name: "Portuguese (Português)" },
  { code: "Dutch", name: "Dutch (Nederlands)" },
  { code: "Polish", name: "Polish (Polski)" },
  { code: "Turkish", name: "Turkish (Türkçe)" },
  { code: "Vietnamese", name: "Vietnamese (Tiếng Việt)" },
  { code: "Thai", name: "Thai (ไทย)" },
  { code: "Indonesian", name: "Indonesian (Bahasa Indonesia)" },
  { code: "Hindi", name: "Hindi (हिन्दी)" },
  { code: "Arabic", name: "Arabic (العربية)" },
];
