<script setup lang="ts">
import { computed, ref } from "vue";
import AppButton from "@/components/ui/AppButton.vue";
import SegmentedTabs, { type TabItem } from "@/components/ui/SegmentedTabs.vue";
import type { ResponseData } from "@/bindings/ResponseData";
import type { RunState } from "@/stores/runner";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";
import { parseSetCookies } from "@/utils/cookies";
import HeaderTable from "./HeaderTable.vue";
import ResponseBodyView from "./ResponseBodyView.vue";
import ResponseMeta from "./ResponseMeta.vue";
import ResponseState from "./ResponseState.vue";
import ResponseTimeline from "./ResponseTimeline.vue";
import ScriptResults from "./ScriptResults.vue";

const props = defineProps<{ run: RunState; example?: boolean }>();
const emit = defineEmits<{ saveExample: [response: ResponseData] }>();
const toasts = useToastsStore();
const dialogs = useDialogsStore();

const section = ref("body");
const mode = ref<"pretty" | "raw">("pretty");
const response = computed(() => (props.run.status === "done" ? props.run.response : undefined));
const cookies = computed(() => parseSetCookies(response.value?.headers ?? []));

const testBadge = computed(() => {
  const tests = response.value?.script.tests ?? [];
  return tests.length ? `${tests.filter((t) => t.passed).length}/${tests.length}` : undefined;
});
const consoleCount = computed(
  () => (response.value?.script.logs.length ?? 0) + (response.value?.script.errors.length ?? 0),
);

const sections = computed<TabItem[]>(() => {
  const base: TabItem[] = [
    { id: "body", label: "Body" },
    { id: "headers", label: "Headers", badge: response.value?.headers.length },
  ];
  // A saved example has no timing or script results.
  if (props.example) return base;
  return [
    ...base,
    { id: "cookies", label: "Cookies", badge: cookies.value.length || undefined },
    { id: "timeline", label: "Timeline" },
    { id: "tests", label: "Tes", badge: testBadge.value },
    { id: "console", label: "Konsol", badge: consoleCount.value || undefined },
  ];
});

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
      <ResponseMeta v-if="response" :response="response" :example="example" />
      <AppButton
        v-if="response && !example"
        icon="save"
        class="ml-auto"
        @click="emit('saveExample', response)"
        >Simpan sebagai contoh</AppButton
      >
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
          <AppButton
            v-else-if="section === 'cookies'"
            variant="ghost"
            @click="dialogs.cookiesOpen = true"
          >
            Kelola cookie
          </AppButton>
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
        <ResponseTimeline v-else-if="section === 'timeline'" :timing="response.timing" />
        <ScriptResults
          v-else-if="section === 'tests' || section === 'console'"
          :report="response.script"
          :view="section === 'tests' ? 'tests' : 'console'"
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
