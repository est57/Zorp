<script setup lang="ts">
import AppButton from "@/components/ui/AppButton.vue";
import { modKey } from "@/composables/usePlatform";
import { useTabsStore } from "@/stores/tabs";

const tabs = useTabsStore();

const shortcuts = [
  { label: "Kirim request", keys: [modKey, "Enter"] },
  { label: "Request baru", keys: [modKey, "T"] },
  { label: "Cari", keys: [modKey, "K"] },
  { label: "Sidebar", keys: [modKey, "B"] },
];
</script>

<template>
  <section class="grid h-full place-items-center overflow-auto bg-surface p-6">
    <div class="grid w-full max-w-sm gap-5 text-center">
      <div class="grid gap-1">
        <h1 class="text-xl font-medium text-balance">Belum ada request terbuka</h1>
        <p class="text-muted">Buat request baru atau pilih dari koleksi untuk memulai.</p>
      </div>
      <div>
        <AppButton variant="primary" icon="plus" @click="tabs.openNew()">Request baru</AppButton>
      </div>
      <dl class="grid gap-2 text-left text-xs">
        <div
          v-for="item in shortcuts"
          :key="item.label"
          class="flex items-center justify-between gap-4 border-b border-line pb-2"
        >
          <dt class="text-muted">{{ item.label }}</dt>
          <dd class="flex gap-1">
            <kbd
              v-for="key in item.keys"
              :key="key"
              class="min-w-6 rounded border border-b-2 border-line bg-bg px-1.5 text-center font-mono text-[11px] text-fg"
            >
              {{ key }}
            </kbd>
          </dd>
        </div>
      </dl>
    </div>
  </section>
</template>
