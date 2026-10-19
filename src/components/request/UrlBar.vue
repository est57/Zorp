<script setup lang="ts">
import { ref } from "vue";
import type { RequestSpec } from "@/bindings/RequestSpec";
import AppButton from "@/components/ui/AppButton.vue";
import VariableInput from "@/components/ui/VariableInput.vue";
import { METHODS, methodColor } from "@/utils/request";
import { paramsFromUrl } from "@/utils/url";

const props = defineProps<{ spec: RequestSpec; loading: boolean; dirty: boolean }>();
const emit = defineEmits<{ send: []; cancel: []; save: [] }>();

const urlInput = ref<InstanceType<typeof VariableInput>>();

function onUrl(url: string) {
  props.spec.url = url;
  props.spec.params = paramsFromUrl(url, props.spec.params);
}

defineExpose({ focus: () => urlInput.value?.focus() });
</script>

<template>
  <div class="flex min-w-0 gap-2 px-3 py-2.5">
    <div
      class="flex h-9 min-w-0 flex-1 items-stretch overflow-hidden rounded-md border border-line bg-bg focus-within:border-accent"
    >
      <select
        v-model="spec.method"
        aria-label="Method"
        class="w-[92px] shrink-0 cursor-pointer border-r border-line bg-transparent px-2.5 font-mono text-xs font-bold outline-none max-sm:w-[76px]"
        :class="methodColor(spec.method)"
      >
        <option v-for="method in METHODS" :key="method" :value="method" class="text-fg">
          {{ method }}
        </option>
      </select>
      <VariableInput
        ref="urlInput"
        input-id="request-url"
        :model-value="spec.url"
        label="URL"
        size="md"
        placeholder="https://api.contoh.com/v1/users atau {{base_url}}/users"
        @update:model-value="onUrl"
        @enter="emit('send')"
      />
    </div>
    <AppButton
      v-if="loading"
      variant="danger"
      class="!h-9 w-[88px] max-sm:w-16"
      @click="emit('cancel')"
    >
      <span class="size-3 animate-spin rounded-full border-2 border-current border-r-transparent" />
      Batal
    </AppButton>
    <AppButton
      v-else
      variant="primary"
      class="!h-9 w-[88px] text-[13px] max-sm:w-16"
      @click="emit('send')"
    >
      Kirim
    </AppButton>
    <AppButton
      icon="save"
      class="!h-9 max-md:!w-9 max-md:!px-0"
      :aria-label="dirty ? 'Simpan perubahan' : 'Simpan'"
      @click="emit('save')"
    >
      <span class="max-md:hidden">Simpan</span>
    </AppButton>
  </div>
</template>
