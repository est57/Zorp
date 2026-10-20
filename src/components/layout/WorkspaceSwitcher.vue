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
import { useWorkspaceActions } from "@/composables/useWorkspaceActions";
import { revealFolder } from "@/api/workspaces";
import { isMac } from "@/composables/usePlatform";
import { useAppStore } from "@/stores/app";
import { useToastsStore } from "@/stores/toasts";
import { useWorkspacesStore } from "@/stores/workspaces";

const app = useAppStore();
const workspaces = useWorkspacesStore();
const actions = useWorkspaceActions();
const toasts = useToastsStore();

const folderName = (path: string) => path.split(/[\\/]/).pop() ?? path;

const item =
  "flex h-7 cursor-default items-center gap-2 rounded px-2 outline-none data-highlighted:bg-surface-2 data-disabled:opacity-50";
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger
      class="flex h-[26px] max-w-56 min-w-0 items-center gap-1.5 rounded-md px-2 text-muted hover:bg-surface-2 hover:text-fg"
      aria-label="Pilih workspace"
    >
      <AppIcon
        :name="workspaces.active?.folder ? 'folder' : 'layers'"
        :size="13"
        class="shrink-0"
      />
      <span class="truncate">{{ workspaces.active?.name ?? app.info?.workspace.name }}</span>
      <AppIcon name="chevronDown" :size="12" class="shrink-0 text-subtle" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        align="start"
        :side-offset="4"
        class="z-50 max-w-80 min-w-60 rounded-md border border-line bg-surface p-1 text-xs text-fg shadow-float"
      >
        <DropdownMenuLabel class="label-caps px-2 py-1.5">Workspace</DropdownMenuLabel>
        <DropdownMenuItem
          v-for="w in workspaces.items"
          :key="w.id"
          :class="[item, w.folder && 'h-auto py-1.5']"
          :title="w.folder ?? undefined"
          @select="actions.switchTo(w.id)"
        >
          <span
            class="size-2 shrink-0 rounded-full"
            :class="w.id === workspaces.activeId ? 'bg-success' : 'bg-line'"
          />
          <span class="grid min-w-0 flex-1">
            <span class="truncate">{{ w.name }}</span>
            <span v-if="w.folder" class="flex min-w-0 items-center gap-1 text-[11px] text-subtle">
              <AppIcon name="folder" :size="11" class="shrink-0" />
              <span class="truncate">{{ folderName(w.folder) }}</span>
            </span>
          </span>
        </DropdownMenuItem>
        <DropdownMenuSeparator class="my-1 h-px bg-line" />
        <DropdownMenuItem :class="item" @select="actions.create">
          <AppIcon name="plus" :size="14" />
          Workspace baru
        </DropdownMenuItem>
        <DropdownMenuItem :class="item" @select="actions.openFolder">
          <AppIcon name="folder" :size="14" />
          Buka folder
        </DropdownMenuItem>
        <DropdownMenuSeparator class="my-1 h-px bg-line" />
        <DropdownMenuItem
          v-if="!workspaces.active?.folder"
          :class="item"
          @select="actions.linkFolder"
        >
          <AppIcon name="save" :size="14" />
          Simpan ke folder
        </DropdownMenuItem>
        <DropdownMenuItem
          v-if="workspaces.active?.folder"
          :class="item"
          @select="toasts.guard(revealFolder)"
        >
          <AppIcon name="folder" :size="14" />
          {{ isMac ? "Tampilkan di Finder" : "Tampilkan folder" }}
        </DropdownMenuItem>
        <DropdownMenuItem
          v-if="workspaces.active?.folder"
          :class="item"
          @select="actions.unlinkFolder"
        >
          <AppIcon name="x" :size="14" />
          Berhenti sinkron folder
        </DropdownMenuItem>
        <DropdownMenuItem :class="item" @select="actions.rename">
          <AppIcon name="pencil" :size="14" />
          Ganti nama
        </DropdownMenuItem>
        <DropdownMenuItem
          :class="[item, 'text-danger']"
          :disabled="workspaces.items.length < 2"
          @select="actions.remove"
        >
          <AppIcon name="trash" :size="14" />
          Hapus workspace
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
