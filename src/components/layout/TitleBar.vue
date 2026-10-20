<script setup lang="ts">
import { computed } from "vue";
import IconButton from "@/components/ui/IconButton.vue";
import { isMac } from "@/composables/usePlatform";
import { useAppStore } from "@/stores/app";
import { useLayoutStore } from "@/stores/layout";
import EnvSwitcher from "./EnvSwitcher.vue";
import WorkspaceSwitcher from "./WorkspaceSwitcher.vue";

const app = useAppStore();
const layout = useLayoutStore();

const themeIcon = computed(() =>
  layout.theme === "light" ? "sun" : layout.theme === "dark" ? "moon" : "monitor",
);
const themeLabel = computed(
  () =>
    ({ system: "Tema: ikuti sistem", light: "Tema: terang", dark: "Tema: gelap" })[layout.theme],
);
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-[38px] shrink-0 items-center gap-3 border-b border-line bg-surface pr-2"
    :class="isMac ? 'pl-[78px]' : 'pl-3'"
  >
    <IconButton
      icon="sidebar"
      label="Tampilkan atau sembunyikan sidebar"
      @click="layout.toggleSidebar()"
    />
    <span data-tauri-drag-region class="font-mono text-[13px] font-bold tracking-wide">zorp</span>
    <WorkspaceSwitcher v-if="app.info" />
    <span data-tauri-drag-region class="flex-1 self-stretch" />
    <EnvSwitcher />
    <IconButton :icon="themeIcon" :label="themeLabel" @click="layout.cycleTheme()" />
  </header>
</template>
