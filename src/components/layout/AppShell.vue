<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import CommandPalette from "@/components/palette/CommandPalette.vue";
import RunnerDialog from "@/components/runner/RunnerDialog.vue";
import SettingsDialog from "@/components/settings/SettingsDialog.vue";
import CodeSnippetDialog from "@/components/request/CodeSnippetDialog.vue";
import SaveRequestDialog from "@/components/request/SaveRequestDialog.vue";
import CookieManagerDialog from "@/components/sidebar/CookieManagerDialog.vue";
import ImportCurlDialog from "@/components/sidebar/ImportCurlDialog.vue";
import NodeAuthDialog from "@/components/sidebar/NodeAuthDialog.vue";
import DialogHost from "@/components/ui/DialogHost.vue";
import ToastHost from "@/components/ui/ToastHost.vue";
import { useMediaQuery } from "@/composables/useMediaQuery";
import { useRequestActions } from "@/composables/useRequestActions";
import { useShortcuts } from "@/composables/useShortcuts";
import { useTheme } from "@/composables/useTheme";
import { useAppStore } from "@/stores/app";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useEnvironmentsStore } from "@/stores/environments";
import { useHistoryStore } from "@/stores/history";
import { useLayoutStore } from "@/stores/layout";
import { useTabsStore } from "@/stores/tabs";
import { useUpdaterStore } from "@/stores/updater";
import { useWorkspacesStore } from "@/stores/workspaces";
import ActivityRail from "./ActivityRail.vue";
import SideBar from "./SideBar.vue";
import StatusBar from "./StatusBar.vue";
import TabBar from "./TabBar.vue";
import TitleBar from "./TitleBar.vue";
import WorkspaceArea from "./WorkspaceArea.vue";

const app = useAppStore();
const layout = useLayoutStore();
const tabs = useTabsStore();
const actions = useRequestActions();
const dialogs = useDialogsStore();
const compact = useMediaQuery("(max-width: 1099px)");
const workspace = ref<InstanceType<typeof WorkspaceArea>>();
const sidebar = ref<InstanceType<typeof SideBar>>();

useTheme();
useShortcuts({
  "mod+b": () => layout.toggleSidebar(),
  "mod+t": () => tabs.openNew(),
  "mod+enter": () => tabs.active && actions.send(tabs.active),
  "mod+s": () => tabs.active && actions.save(tabs.active),
  "mod+w": () => tabs.active && actions.close(tabs.active),
  "mod+l": () => workspace.value?.focusUrl(),
  "mod+k": () => (dialogs.palette = dialogs.palette ? null : "all"),
  "mod+e": () => (dialogs.palette = "environments"),
  "mod+shift+f": () => {
    layout.toggleSidebar(true);
    requestAnimationFrame(() => sidebar.value?.focusSearch());
  },
});

// Close the sidebar on narrow windows and restore it when the window is widened again.
let autoClosed = false;
watch(
  compact,
  (isCompact) => {
    if (isCompact && layout.sidebarOpen) {
      autoClosed = true;
      layout.toggleSidebar(false);
    } else if (!isCompact && autoClosed) {
      autoClosed = false;
      layout.toggleSidebar(true);
    }
  },
  { immediate: true },
);

onMounted(async () => {
  await app.load();
  if (app.status !== "ready") return;
  await Promise.all([
    useCollectionsStore().load(),
    useEnvironmentsStore().load(),
    useHistoryStore().load(),
    useWorkspacesStore().load(),
  ]);
  // Check quietly after startup so it never delays the first render.
  setTimeout(() => void useUpdaterStore().checkForUpdate(true), 5000);
});
</script>

<template>
  <div class="flex h-full flex-col">
    <TitleBar />
    <div class="relative flex min-h-0 flex-1">
      <ActivityRail />
      <div class="relative flex min-w-0 flex-1">
        <SideBar v-if="layout.sidebarOpen" ref="sidebar" :overlay="compact" />
        <div
          v-if="compact && layout.sidebarOpen"
          class="absolute inset-0 z-10 bg-black/20"
          aria-hidden="true"
          @click="layout.toggleSidebar(false)"
        />
        <main class="flex min-w-0 flex-1 flex-col">
          <TabBar />
          <div class="min-h-0 flex-1">
            <WorkspaceArea ref="workspace" />
          </div>
        </main>
      </div>
    </div>
    <StatusBar />
    <DialogHost />
    <SaveRequestDialog />
    <NodeAuthDialog />
    <CodeSnippetDialog />
    <ImportCurlDialog />
    <CommandPalette />
    <CookieManagerDialog />
    <SettingsDialog />
    <RunnerDialog />
    <ToastHost />
  </div>
</template>
