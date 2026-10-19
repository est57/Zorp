<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { CookieInfo } from "@/bindings/CookieInfo";
import { clearCookies, deleteCookie, listCookies } from "@/api/cookies";
import AppButton from "@/components/ui/AppButton.vue";
import IconButton from "@/components/ui/IconButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";

const dialogs = useDialogsStore();
const toasts = useToastsStore();
const cookies = ref<CookieInfo[]>([]);

const open = computed({ get: () => dialogs.cookiesOpen, set: (v) => (dialogs.cookiesOpen = v) });
const groups = computed(() => {
  const map = new Map<string, CookieInfo[]>();
  for (const c of cookies.value) map.set(c.domain, [...(map.get(c.domain) ?? []), c]);
  return [...map.entries()];
});

async function load() {
  cookies.value = (await toasts.guard(listCookies)) ?? [];
}
watch(open, (isOpen) => isOpen && load());

async function remove(cookie: CookieInfo) {
  await toasts.guard(() => deleteCookie(cookie));
  await load();
}

async function clear(domain: string | null) {
  const what = domain ? `semua cookie untuk ${domain}` : "semua cookie di workspace ini";
  const ok = await dialogs.confirm(
    "Hapus cookie?",
    `Hapus ${what}? Sesi login yang memakai cookie tersebut akan berakhir.`,
  );
  if (!ok) return;
  dialogs.cookiesOpen = true;
  await toasts.guard(() => clearCookies(domain));
  await load();
}

const expiry = (c: CookieInfo) =>
  c.expiresAt === null
    ? "Sesi"
    : new Date(c.expiresAt).toLocaleString("id-ID", { dateStyle: "medium", timeStyle: "short" });
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    title="Cookie"
    description="Cookie dari response disimpan per workspace dan dikirim otomatis ke domain yang cocok."
  >
    <div class="grid gap-4 text-xs">
      <p v-if="!cookies.length" class="text-muted">Belum ada cookie tersimpan.</p>
      <section v-for="[domain, items] in groups" :key="domain" class="grid gap-1">
        <header class="flex items-center justify-between gap-2">
          <h3 class="truncate font-mono font-semibold">{{ domain }}</h3>
          <AppButton variant="ghost" class="!text-danger" @click="clear(domain)"
            >Hapus domain</AppButton
          >
        </header>
        <div class="overflow-x-auto rounded-md border border-line">
          <table class="w-full border-collapse select-text">
            <thead>
              <tr class="bg-surface-2 text-left">
                <th class="label-caps h-7 px-2.5">Nama</th>
                <th class="label-caps px-2.5">Value</th>
                <th class="label-caps px-2.5 max-sm:hidden">Path</th>
                <th class="label-caps px-2.5 max-sm:hidden">Berlaku</th>
                <th class="w-8" />
              </tr>
            </thead>
            <tbody>
              <tr v-for="c in items" :key="`${c.path}|${c.name}`" class="border-t border-line">
                <td class="px-2.5 py-1.5 font-mono">
                  {{ c.name }}
                  <span v-if="c.secure" class="ml-1 text-[10px] text-subtle">Secure</span>
                  <span v-if="c.httpOnly" class="ml-1 text-[10px] text-subtle">HttpOnly</span>
                </td>
                <td class="max-w-56 truncate px-2.5 font-mono text-muted" :title="c.value">
                  {{ c.value }}
                </td>
                <td class="px-2.5 font-mono max-sm:hidden">{{ c.path }}</td>
                <td class="px-2.5 whitespace-nowrap max-sm:hidden">{{ expiry(c) }}</td>
                <td>
                  <IconButton icon="trash" :label="`Hapus cookie ${c.name}`" @click="remove(c)" />
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
      <div class="flex justify-between gap-2">
        <AppButton v-if="cookies.length" variant="danger" icon="trash" @click="clear(null)"
          >Hapus semua</AppButton
        >
        <AppButton class="ml-auto" @click="open = false">Tutup</AppButton>
      </div>
    </div>
  </ModalDialog>
</template>
