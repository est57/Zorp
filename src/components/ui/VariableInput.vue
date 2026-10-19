<script setup lang="ts">
import { computed, ref } from "vue";
import { useEnvironmentsStore } from "@/stores/environments";
import { isDynamic, tokenize } from "@/utils/variables";

const model = defineModel<string>({ required: true });
withDefaults(
  defineProps<{ placeholder?: string; label: string; size?: "sm" | "md"; inputId?: string }>(),
  {
    placeholder: "",
    size: "sm",
    inputId: undefined,
  },
);
defineEmits<{ enter: []; paste: [event: ClipboardEvent] }>();

const envs = useEnvironmentsStore();
const input = ref<HTMLInputElement>();
const mirror = ref<HTMLDivElement>();

const segments = computed(() => tokenize(model.value));
const known = (name: string) => isDynamic(name) || envs.names.has(name);

function syncScroll() {
  if (mirror.value && input.value) mirror.value.scrollLeft = input.value.scrollLeft;
}

defineExpose({ focus: () => input.value?.focus() });
</script>

<template>
  <div class="relative min-w-0 flex-1 font-mono" :class="size === 'md' ? 'text-[13px]' : 'text-xs'">
    <div
      ref="mirror"
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 flex items-center overflow-hidden px-2 whitespace-pre"
    >
      <template v-for="(segment, index) in segments" :key="index">
        <span
          v-if="segment.variable"
          :class="
            known(segment.variable)
              ? 'rounded-sm bg-accent-soft text-accent'
              : 'text-danger underline decoration-danger decoration-wavy underline-offset-3'
          "
          >{{ segment.text }}</span
        >
        <span v-else>{{ segment.text }}</span>
      </template>
    </div>
    <input
      :id="inputId"
      ref="input"
      v-model="model"
      :aria-label="label"
      :placeholder="placeholder"
      spellcheck="false"
      autocomplete="off"
      class="relative h-full w-full bg-transparent px-2 text-transparent caret-[var(--fg)] outline-none placeholder:text-subtle selection:bg-accent/30 selection:text-transparent"
      @scroll="syncScroll"
      @input="syncScroll"
      @keyup="syncScroll"
      @keydown.enter="$emit('enter')"
      @paste="$emit('paste', $event)"
    />
  </div>
</template>
