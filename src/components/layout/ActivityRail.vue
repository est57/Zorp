<script setup lang="ts">
import IconButton from "@/components/ui/IconButton.vue";
import type { IconName } from "@/components/ui/icons";
import { useDialogsStore } from "@/stores/dialogs";
import { useLayoutStore, type SidebarView } from "@/stores/layout";

const layout = useLayoutStore();
const dialogs = useDialogsStore();

const items: { view: SidebarView; icon: IconName; label: string }[] = [
  { view: "collections", icon: "folder", label: "Koleksi" },
  { view: "history", icon: "history", label: "Riwayat" },
  { view: "environments", icon: "layers", label: "Environment" },
];
</script>

<template>
  <nav
    aria-label="Navigasi utama"
    class="flex w-12 shrink-0 flex-col items-center gap-1 border-r border-line bg-surface py-2"
  >
    <IconButton
      v-for="item in items"
      :key="item.view"
      :icon="item.icon"
      :label="item.label"
      size="md"
      :active="layout.sidebarOpen && layout.view === item.view"
      @click="layout.selectView(item.view)"
    />
    <IconButton
      class="mt-auto"
      icon="settings"
      label="Pengaturan"
      size="md"
      @click="dialogs.settingsOpen = true"
    />
  </nav>
</template>
