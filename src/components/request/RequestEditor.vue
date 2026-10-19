<script setup lang="ts">
import { computed, ref } from "vue";
import type { RequestSpec } from "@/bindings/RequestSpec";
import KeyValueTable from "@/components/ui/KeyValueTable.vue";
import SegmentedTabs, { type TabItem } from "@/components/ui/SegmentedTabs.vue";
import { urlFromParams } from "@/utils/url";
import AuthForm from "./AuthForm.vue";
import BodyEditor from "./BodyEditor.vue";
import SettingsPanel from "./SettingsPanel.vue";

const props = defineProps<{ spec: RequestSpec; collectionId: string | null }>();

const section = ref(props.spec.body.type !== "none" ? "body" : "params");
const count = (rows: { enabled: boolean; key: string }[]) =>
  rows.filter((r) => r.enabled && r.key).length || undefined;

const sections = computed<TabItem[]>(() => [
  { id: "params", label: "Params", badge: count(props.spec.params) },
  { id: "headers", label: "Headers", badge: count(props.spec.headers) },
  { id: "body", label: "Body", badge: props.spec.body.type === "none" ? undefined : "1" },
  { id: "auth", label: "Auth" },
  { id: "settings", label: "Pengaturan" },
]);

function setParams(rows: RequestSpec["params"]) {
  props.spec.params = rows;
  props.spec.url = urlFromParams(props.spec.url, rows);
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <SegmentedTabs v-model="section" :items="sections" label="Bagian request" />
    <div class="min-h-0 flex-1 overflow-auto">
      <KeyValueTable
        v-if="section === 'params'"
        :model-value="spec.params"
        name="Query params"
        describe
        @update:model-value="setParams"
      />
      <KeyValueTable
        v-else-if="section === 'headers'"
        v-model="spec.headers"
        name="Headers"
        describe
      />
      <BodyEditor v-else-if="section === 'body'" :spec="spec" />
      <AuthForm
        v-else-if="section === 'auth'"
        v-model="spec.auth"
        inherit-label="Ikuti folder atau koleksi"
        :collection-id="collectionId"
      />
      <SettingsPanel v-else :spec="spec" />
    </div>
  </div>
</template>
