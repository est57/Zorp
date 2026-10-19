<script setup lang="ts">
import type { ProxySettings } from "@/bindings/ProxySettings";
import { checkboxClass, inputClass } from "@/components/ui/styles";

const proxy = defineModel<ProxySettings>({ required: true });
const field = [inputClass, "!h-8 font-mono"];
</script>

<template>
  <div class="grid gap-3 text-xs">
    <label class="flex items-center gap-2.5">
      <input v-model="proxy.enabled" type="checkbox" :class="checkboxClass" />
      Kirim request lewat proxy
    </label>
    <fieldset :disabled="!proxy.enabled" class="grid gap-3 disabled:opacity-50">
      <label class="grid gap-1.5">
        <span class="text-muted">URL proxy</span>
        <input
          v-model="proxy.url"
          placeholder="http://127.0.0.1:8080 atau socks5://127.0.0.1:1080"
          :class="field"
        />
      </label>
      <div class="grid grid-cols-2 gap-3 max-sm:grid-cols-1">
        <label class="grid gap-1.5">
          <span class="text-muted">Username (opsional)</span>
          <input v-model="proxy.username" autocomplete="off" :class="field" />
        </label>
        <label class="grid gap-1.5">
          <span class="text-muted">Password</span>
          <input
            v-model="proxy.password"
            type="password"
            autocomplete="off"
            :placeholder="proxy.hasPassword ? 'Tersimpan di keychain, isi untuk mengganti' : ''"
            :class="field"
          />
        </label>
      </div>
      <label class="grid gap-1.5">
        <span class="text-muted">Host tanpa proxy</span>
        <input
          v-model="proxy.bypass"
          placeholder="localhost, 127.0.0.1, .internal"
          :class="field"
        />
        <span class="text-subtle"
          >Pisahkan dengan koma. Awali dengan titik untuk mencakup semua subdomain.</span
        >
      </label>
    </fieldset>
  </div>
</template>
