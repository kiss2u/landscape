import { createI18n } from "vue-i18n";

import en from "./en/main";
import zh from "./zh/main";

const messages = {
  en,
  "en-US": en,
  zh,
  "zh-CN": zh,
};

const i18n = createI18n({
  legacy: false,
  locale: "zh",
  fallbackLocale: "zh",
  messages,
});

export default i18n;
