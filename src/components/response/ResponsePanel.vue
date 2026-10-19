<script setup lang="ts">
import { computed, ref } from "vue";
import AppButton from "@/components/ui/AppButton.vue";
import SegmentedTabs, { type TabItem } from "@/components/ui/SegmentedTabs.vue";
import type { RunState } from "@/stores/runner";
import { useToastsStore } from "@/stores/toasts";
import { parseSetCookies } from "@/utils/cookies";
import HeaderTable from "./HeaderTable.vue";
import ResponseBodyView from "./ResponseBodyView.vue";
import ResponseMeta from "./ResponseMeta.vue";
import ResponseState from "./ResponseState.vue";

const props = defineProps<{ run: RunState }>();
const toasts = useToastsStore();

const section = ref("body");
const mode = ref<"pretty" | "raw">("pretty");
const response = computed(() => (props.run.status === "done" ? props.run.response : undefined));
const cookies = computed(() => parseSetCookies(response.value?.headers ?? []));

const sections = computed<TabItem[]>(() => [
  { id: "body", label: "Body" },
  { id: "headers", label: "Headers", badge: response.value?.headers.length },
  { id: "cookies", label: "Cookies", badge: cookies.value.length || undefined },
]);

async function copyBody() {
  const body = response.value?.body;
  const text = body?.type === "text" ? body.text : body?.type === "too_large" ? body.preview : "";
  try {
    await navigator.clipboard.writeText(text);
    toasts.push("Body disalin ke clipboard", "success");
  } catch {
    toasts.push("Gagal menyalin ke clipboard", "danger");
  }
}
</script>

<template>
  <section aria-label="Response" class="flex h-full min-h-0 flex-col bg-surface">
    <div class="flex min-h-10 items-center gap-3 px-3 py-1.5">
      <span class="label-caps">Response</span>
      <ResponseMeta v-if="response" :response="response" />
    </div>
    <template v-if="response">
      <SegmentedTabs v-model="section" :items="sections" label="Bagian response">
        <template #actions>
          <template v-if="section === 'body'">
            <AppButton
              variant="ghost"
              :class="mode === 'pretty' && '!text-fg'"
              @click="mode = 'pretty'"
              >Pretty</AppButton
            >
            <AppButton variant="ghost" :class="mode === 'raw' && '!text-fg'" @click="mode = 'raw'"
              >Raw</AppButton
            >
            <AppButton variant="ghost" icon="copy" aria-label="Salin body" @click="copyBody" />
          </template>
        </template>
      </SegmentedTabs>
      <div class="min-h-0 flex-1 overflow-auto">
        <ResponseBodyView v-if="section === 'body'" :response="response" :mode="mode" />
        <HeaderTable
          v-else-if="section === 'headers'"
          :rows="response.headers"
          :columns="['Header', 'Value']"
          empty="Response tidak memiliki header."
        />
        <HeaderTable
          v-else
          :rows="cookies.map((c) => [c.name, c.value, c.attributes])"
          :columns="['Nama', 'Value', 'Atribut']"
          empty="Response ini tidak mengirim cookie."
        />
      </div>
    </template>
    <div v-else class="min-h-0 flex-1 overflow-auto border-t border-line">
      <ResponseState :run="run" />
    </div>
  </section>
</template>
