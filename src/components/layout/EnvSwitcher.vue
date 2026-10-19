<script setup lang="ts">
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "reka-ui";
import AppIcon from "@/components/ui/AppIcon.vue";
import { useEnvironmentsStore } from "@/stores/environments";
import { useLayoutStore } from "@/stores/layout";
import { useToastsStore } from "@/stores/toasts";

const envs = useEnvironmentsStore();
const layout = useLayoutStore();
const toasts = useToastsStore();

const activate = (id: string | null) => toasts.guard(() => envs.activate(id));
const item =
  "flex h-7 cursor-default items-center gap-2 rounded px-2 outline-none data-highlighted:bg-surface-2";
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger
      class="flex h-[26px] max-w-48 items-center gap-2 rounded-md border border-line bg-bg px-2.5 text-xs"
      aria-label="Pilih environment"
    >
      <span
        class="size-2 shrink-0 rounded-full"
        :class="envs.active ? 'bg-success' : 'bg-subtle'"
      />
      <span class="truncate">{{ envs.active?.name ?? "Tanpa environment" }}</span>
      <AppIcon name="chevronDown" :size="12" class="shrink-0 text-subtle" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        align="end"
        :side-offset="4"
        class="z-50 min-w-52 rounded-md border border-line bg-surface p-1 text-xs text-fg shadow-float"
      >
        <DropdownMenuItem :class="item" @select="activate(null)">
          <span class="size-2 rounded-full" :class="envs.active ? 'bg-line' : 'bg-success'" />
          Tanpa environment
        </DropdownMenuItem>
        <DropdownMenuItem
          v-for="env in envs.items"
          :key="env.id"
          :class="item"
          @select="activate(env.id)"
        >
          <span class="size-2 rounded-full" :class="env.isActive ? 'bg-success' : 'bg-line'" />
          <span class="truncate">{{ env.name }}</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator class="my-1 h-px bg-line" />
        <DropdownMenuItem
          :class="item"
          @select="
            layout.selectView('environments');
            layout.toggleSidebar(true);
          "
        >
          <AppIcon name="layers" :size="14" />
          Kelola environment
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
