<script setup lang="ts">
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuTrigger,
} from "reka-ui";
import AppIcon from "@/components/ui/AppIcon.vue";
import { useImport } from "@/composables/useImport";
import { useDialogsStore } from "@/stores/dialogs";

const dialogs = useDialogsStore();
const { fromPostman, environmentFromPostman } = useImport();

const item =
  "flex h-7 cursor-default items-center gap-2 rounded px-2 outline-none data-highlighted:bg-surface-2";
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger
      class="grid size-7 place-items-center rounded-md text-muted hover:bg-surface-2 hover:text-fg"
      aria-label="Impor"
      title="Impor"
    >
      <AppIcon name="download" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        align="end"
        :side-offset="4"
        class="z-50 min-w-52 rounded-md border border-line bg-surface p-1 text-xs text-fg shadow-float"
      >
        <DropdownMenuItem :class="item" @select="dialogs.curlImportOpen = true">
          <AppIcon name="code" :size="14" />
          Dari perintah cURL
        </DropdownMenuItem>
        <DropdownMenuItem :class="item" @select="fromPostman">
          <AppIcon name="file" :size="14" />
          Dari koleksi Postman (v2.1)
        </DropdownMenuItem>
        <DropdownMenuItem :class="item" @select="environmentFromPostman">
          <AppIcon name="layers" :size="14" />
          Environment Postman
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
