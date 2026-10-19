<script setup lang="ts">
import { computed, ref } from "vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import IconButton from "@/components/ui/IconButton.vue";
import CollectionTree from "@/components/sidebar/CollectionTree.vue";
import EnvironmentList from "@/components/sidebar/EnvironmentList.vue";
import HistoryList from "@/components/sidebar/HistoryList.vue";
import ImportMenu from "@/components/sidebar/ImportMenu.vue";
import SearchResults from "@/components/sidebar/SearchResults.vue";
import { useTreeActions } from "@/components/sidebar/useTreeActions";
import { modKey } from "@/composables/usePlatform";
import { useLayoutStore } from "@/stores/layout";
import ResizeHandle from "./ResizeHandle.vue";

defineProps<{ overlay: boolean }>();

const layout = useLayoutStore();
const actions = useTreeActions();
const query = ref("");
const search = ref<HTMLInputElement>();

const title = computed(
  () => ({ collections: "Koleksi", history: "Riwayat", environments: "Environment" })[layout.view],
);

defineExpose({ focusSearch: () => search.value?.focus() });
</script>

<template>
  <aside
    :aria-label="title"
    class="relative flex shrink-0 flex-col border-r border-line bg-surface"
    :class="overlay && 'absolute inset-y-0 left-0 z-20 shadow-float'"
    :style="{ width: `${layout.sidebarWidth}px` }"
  >
    <div class="grid gap-2 px-2.5 pt-2.5 pb-1.5">
      <div class="flex h-7 items-center justify-between">
        <h2 class="label-caps">{{ title }}</h2>
        <div v-if="layout.view === 'collections'" class="flex items-center gap-0.5">
          <ImportMenu />
          <IconButton icon="plus" label="Koleksi baru" @click="actions.newCollection" />
        </div>
      </div>
      <label
        v-if="layout.view !== 'environments'"
        class="flex h-7 items-center gap-1.5 rounded border border-line bg-bg px-2 text-xs text-subtle focus-within:border-accent"
      >
        <AppIcon name="search" :size="13" />
        <input
          id="sidebar-search"
          ref="search"
          v-model="query"
          :placeholder="layout.view === 'history' ? 'Cari URL' : 'Cari request'"
          aria-label="Cari"
          class="min-w-0 flex-1 bg-transparent text-fg outline-none placeholder:text-subtle"
        />
        <kbd class="font-mono text-[11px]">{{ modKey }}K</kbd>
      </label>
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto">
      <template v-if="layout.view === 'collections'">
        <SearchResults v-if="query" :query="query" />
        <CollectionTree v-else />
      </template>
      <HistoryList v-else-if="layout.view === 'history'" :filter="query" />
      <EnvironmentList v-else />
    </div>
    <ResizeHandle :width="layout.sidebarWidth" @resize="layout.setSidebarWidth" />
  </aside>
</template>
