<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuTrigger,
} from "reka-ui";
import { importPostman } from "@/api/importers";
import AppIcon from "@/components/ui/AppIcon.vue";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";

const collections = useCollectionsStore();
const dialogs = useDialogsStore();
const toasts = useToastsStore();

async function fromPostman() {
  const path = await open({
    multiple: false,
    filters: [{ name: "Koleksi Postman", extensions: ["json"] }],
  });
  if (typeof path !== "string") return;
  const report = await toasts.guard(() => importPostman(path));
  if (!report) return;
  collections.toggle(report.collectionId, true);
  await collections.load();
  toasts.push(
    `"${report.collectionName}" diimpor: ${report.requests} request, ${report.folders} folder`,
    "success",
  );
  if (report.warnings.length) {
    const more = report.warnings.length > 1 ? ` (dan ${report.warnings.length - 1} lainnya)` : "";
    toasts.push(`${report.warnings[0]}${more}`, "danger");
  }
}

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
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
