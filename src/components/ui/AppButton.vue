<script setup lang="ts">
import AppIcon from "./AppIcon.vue";
import type { IconName } from "./icons";

withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "ghost" | "danger";
    icon?: IconName;
    type?: "button" | "submit";
  }>(),
  { variant: "secondary", icon: undefined, type: "button" },
);
</script>

<template>
  <button
    :type="type"
    class="inline-flex h-7 shrink-0 items-center justify-center gap-1.5 rounded-md px-3 text-xs font-medium whitespace-nowrap transition-[filter,background-color] duration-100 disabled:cursor-not-allowed disabled:opacity-50"
    :class="{
      'bg-accent text-accent-fg hover:brightness-110': variant === 'primary',
      'border border-line bg-surface-2 text-fg hover:bg-line': variant === 'secondary',
      'text-muted hover:bg-surface-2 hover:text-fg': variant === 'ghost',
      'border border-danger text-danger hover:bg-danger/10': variant === 'danger',
    }"
  >
    <AppIcon v-if="icon" :name="icon" :size="14" />
    <slot />
  </button>
</template>
