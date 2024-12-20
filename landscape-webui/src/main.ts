import { createApp } from "vue";
import { createPinia } from "pinia";
// General Font
import "vfonts/Lato.css";
// Monospace Font
import "vfonts/FiraCode.css";

import "./style.css";

import App from "./App.vue";
const pinia = createPinia();

createApp(App).use(pinia).mount("#app");
