import { createApp } from "vue";
import { createPinia } from "pinia";
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import "@fontsource-variable/jetbrains-mono";
import "./styles/main.css";
import App from "./App.vue";

async function start() {
  if (import.meta.env.DEV && new URLSearchParams(location.search).has("mock")) {
    (await import("./dev/mockIpc")).installMockBackend();
  }
  createApp(App).use(createPinia()).mount("#app");
}

void start();
