<script setup lang="ts">
export interface TabItem {
  id: string;
  label: string;
  badge?: string | number;
}

defineProps<{ items: TabItem[]; label: string }>();
const model = defineModel<string>({ required: true });
</script>

<template>
  <div class="flex min-w-0 items-center border-b border-line px-2">
    <div
      role="tablist"
      :aria-label="label"
      class="flex min-w-0 gap-0.5 overflow-x-auto [scrollbar-width:none]"
    >
      <button
        v-for="item in items"
        :key="item.id"
        type="button"
        role="tab"
        :aria-selected="model === item.id"
        class="relative flex h-8 shrink-0 items-center gap-1.5 px-2.5 text-xs transition-colors"
        :class="model === item.id ? 'font-medium text-fg' : 'text-muted hover:text-fg'"
        @click="model = item.id"
      >
        {{ item.label }}
        <span
          v-if="item.badge"
          class="rounded-full bg-surface-2 px-1.5 font-mono text-[10px] leading-4 text-subtle"
          >{{ item.badge }}</span
        >
        <span
          v-if="model === item.id"
          class="absolute inset-x-2 -bottom-px h-0.5 rounded-full bg-accent"
        />
      </button>
    </div>
    <div class="ml-auto flex shrink-0 items-center gap-1 pl-2">
      <slot name="actions" />
    </div>
  </div>
</template>
