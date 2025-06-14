import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import router from "./router";
import i18n from "./i18n";

// General Font
import "vfonts/Lato.css";
// Monospace Font
import "vfonts/FiraCode.css";

import "./style.css";

import App from "./App.vue";
const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);

createApp(App).use(i18n).use(router).use(pinia).mount("#app");
