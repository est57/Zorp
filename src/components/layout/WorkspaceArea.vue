<script setup lang="ts">
import { ref } from "vue";
import RequestEditor from "@/components/request/RequestEditor.vue";
import UrlBar from "@/components/request/UrlBar.vue";
import RealtimePanel from "@/components/realtime/RealtimePanel.vue";
import ExampleView from "@/components/response/ExampleView.vue";
import ResponsePanel from "@/components/response/ResponsePanel.vue";
import { useExampleActions } from "@/composables/useExampleActions";
import { useRequestActions } from "@/composables/useRequestActions";
import { useDialogsStore } from "@/stores/dialogs";
import { useRealtimeStore } from "@/stores/realtime";
import { useRunnerStore } from "@/stores/runner";
import { type Tab, useTabsStore } from "@/stores/tabs";
import EmptyWorkspace from "./EmptyWorkspace.vue";
import SplitPane from "./SplitPane.vue";

const tabs = useTabsStore();
const runner = useRunnerStore();
const realtime = useRealtimeStore();
const actions = useRequestActions();
const dialogs = useDialogsStore();
const examples = useExampleActions();
const urlBar = ref<InstanceType<typeof UrlBar>>();

defineExpose({ focusUrl: () => urlBar.value?.focus() });
</script>

<template>
  <ExampleView
    v-if="tabs.active?.example"
    :key="tabs.active.id"
    :tab="tabs.active as Tab & { example: NonNullable<Tab['example']> }"
  />
  <div
    v-else-if="tabs.active"
    :key="tabs.active.id"
    class="flex h-full min-h-0 flex-col bg-surface"
  >
    <UrlBar
      ref="urlBar"
      :spec="tabs.active.draft"
      :loading="runner.get(tabs.active.id).status === 'loading'"
      :connection="realtime.get(tabs.active.id)?.status ?? 'idle'"
      :dirty="tabs.isDirty(tabs.active)"
      @send="actions.send(tabs.active)"
      @cancel="actions.cancel(tabs.active)"
      @save="actions.save(tabs.active)"
      @code="dialogs.snippetTabId = tabs.active.id"
    />
    <div class="border-t border-line" />
    <SplitPane>
      <template #top
        ><RequestEditor :spec="tabs.active.draft" :collection-id="tabs.active.collectionId"
      /></template>
      <template #bottom
        ><RealtimePanel v-if="tabs.active.draft.mode !== 'http'" :tab="tabs.active" />
        <ResponsePanel
          v-else
          :run="runner.get(tabs.active.id)"
          @save-example="(response) => tabs.active && examples.save(tabs.active, response)"
      /></template>
    </SplitPane>
  </div>
  <EmptyWorkspace v-else />
</template>
