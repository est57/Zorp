<script setup lang="ts">
import { ref } from "vue";
import RequestEditor from "@/components/request/RequestEditor.vue";
import UrlBar from "@/components/request/UrlBar.vue";
import ResponsePanel from "@/components/response/ResponsePanel.vue";
import { useRequestActions } from "@/composables/useRequestActions";
import { useRunnerStore } from "@/stores/runner";
import { useTabsStore } from "@/stores/tabs";
import EmptyWorkspace from "./EmptyWorkspace.vue";
import SplitPane from "./SplitPane.vue";

const tabs = useTabsStore();
const runner = useRunnerStore();
const actions = useRequestActions();
const urlBar = ref<InstanceType<typeof UrlBar>>();

defineExpose({ focusUrl: () => urlBar.value?.focus() });
</script>

<template>
  <div v-if="tabs.active" :key="tabs.active.id" class="flex h-full min-h-0 flex-col bg-surface">
    <UrlBar
      ref="urlBar"
      :spec="tabs.active.draft"
      :loading="runner.get(tabs.active.id).status === 'loading'"
      :dirty="tabs.isDirty(tabs.active)"
      @send="actions.send(tabs.active)"
      @cancel="actions.cancel(tabs.active)"
      @save="actions.save(tabs.active)"
    />
    <div class="border-t border-line" />
    <SplitPane>
      <template #top><RequestEditor :spec="tabs.active.draft" /></template>
      <template #bottom><ResponsePanel :run="runner.get(tabs.active.id)" /></template>
    </SplitPane>
  </div>
  <EmptyWorkspace v-else />
</template>
