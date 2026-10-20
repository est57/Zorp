<script setup lang="ts">
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuSeparator,
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
        class="z-50 min-w-56 rounded-md border border-line bg-surface p-1 text-xs text-fg shadow-float"
      >
        <DropdownMenuLabel class="label-caps px-2 py-1">Impor koleksi</DropdownMenuLabel>
        <DropdownMenuItem :class="item" @select="fromPostman">
          <AppIcon name="file" :size="14" />
          Postman (v2.1)
        </DropdownMenuItem>
        <DropdownMenuItem :class="item" @select="dialogs.openApiOpen = true">
          <AppIcon name="file" :size="14" />
          OpenAPI / Swagger
          <span
            class="ml-auto rounded bg-accent-soft px-1.5 text-[10px] leading-4 font-semibold text-accent"
            >Baru</span
          >
        </DropdownMenuItem>
        <DropdownMenuItem :class="item" @select="dialogs.curlImportOpen = true">
          <AppIcon name="code" :size="14" />
          Perintah cURL
        </DropdownMenuItem>
        <DropdownMenuSeparator class="my-1 h-px bg-line" />
        <DropdownMenuLabel class="label-caps px-2 py-1">Impor environment</DropdownMenuLabel>
        <DropdownMenuItem :class="item" @select="environmentFromPostman">
          <AppIcon name="layers" :size="14" />
          Environment Postman
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
