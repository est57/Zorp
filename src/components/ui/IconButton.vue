<script setup lang="ts">
import AppIcon from "./AppIcon.vue";
import type { IconName } from "./icons";

withDefaults(
  defineProps<{ icon: IconName; label: string; active?: boolean; size?: "sm" | "md" }>(),
  {
    active: false,
    size: "sm",
  },
);
</script>

<template>
  <button
    type="button"
    :aria-label="label"
    :title="label"
    :aria-pressed="active || undefined"
    class="grid shrink-0 place-items-center rounded-md transition-colors duration-100"
    :class="[
      size === 'md' ? 'size-9' : 'size-7',
      active ? 'bg-accent-soft text-accent' : 'text-muted hover:bg-surface-2 hover:text-fg',
    ]"
  >
    <AppIcon :name="icon" />
  </button>
</template>
