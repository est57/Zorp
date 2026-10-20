<script setup lang="ts">
import IconButton from "@/components/ui/IconButton.vue";
import { useRequestActions } from "@/composables/useRequestActions";
import { useTabsStore } from "@/stores/tabs";
import { specColor, specLabel } from "@/utils/request";

const tabs = useTabsStore();
const actions = useRequestActions();
</script>

<template>
  <div class="flex h-9 shrink-0 border-b border-line bg-bg">
    <div
      role="tablist"
      aria-label="Request terbuka"
      class="flex min-w-0 overflow-x-auto [scrollbar-width:none]"
    >
      <div
        v-for="tab in tabs.tabs"
        :key="tab.id"
        class="group relative flex h-9 max-w-56 shrink-0 items-center border-r border-line text-xs"
        :class="
          tab.id === tabs.activeId ? 'bg-surface font-medium text-fg' : 'text-muted hover:text-fg'
        "
        @auxclick.middle="actions.close(tab)"
      >
        <button
          type="button"
          role="tab"
          :aria-selected="tab.id === tabs.activeId"
          class="flex h-full min-w-0 items-center gap-2 pr-1 pl-3"
          @click="tabs.activeId = tab.id"
        >
          <span class="font-mono text-[10px] font-semibold" :class="specColor(tab.draft)">
            {{ specLabel(tab.draft) }}
          </span>
          <span class="truncate">{{ tab.draft.name }}</span>
        </button>
        <span class="relative mr-1.5 grid size-5 place-items-center">
          <span
            v-if="tabs.isDirty(tab)"
            class="size-[7px] rounded-full bg-muted group-hover:hidden"
            aria-label="Belum disimpan"
          />
          <button
            type="button"
            class="grid size-5 place-items-center rounded text-subtle hover:bg-surface-2 hover:text-fg"
            :class="tabs.isDirty(tab) && 'hidden group-hover:grid'"
            :aria-label="`Tutup ${tab.draft.name}`"
            @click="actions.close(tab)"
          >
            &times;
          </button>
        </span>
        <span
          v-if="tab.id === tabs.activeId"
          class="absolute inset-x-0 -bottom-px h-0.5 bg-accent"
        />
      </div>
    </div>
    <div class="flex shrink-0 items-center px-1">
      <IconButton icon="plus" label="Request baru" @click="tabs.openNew()" />
    </div>
  </div>
</template>
