<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { json } from "@codemirror/lang-json";
import { javascript } from "@codemirror/lang-javascript";
import { baseExtensions } from "./codemirror";

const props = withDefaults(
  defineProps<{ readonly?: boolean; language?: "json" | "text" | "javascript"; label: string }>(),
  {
    readonly: false,
    language: "text",
  },
);
const model = defineModel<string>({ required: true });

const host = ref<HTMLDivElement>();
const languageSlot = new Compartment();
let view: EditorView | null = null;

const languageExtension = () =>
  props.language === "json" ? json() : props.language === "javascript" ? javascript() : [];

onMounted(() => {
  view = new EditorView({
    parent: host.value,
    state: EditorState.create({
      doc: model.value,
      extensions: [
        baseExtensions,
        languageSlot.of(languageExtension()),
        EditorState.readOnly.of(props.readonly),
        EditorView.contentAttributes.of({ "aria-label": props.label }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) model.value = update.state.doc.toString();
        }),
      ],
    }),
  });
});

watch(model, (value) => {
  if (view && value !== view.state.doc.toString()) {
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } });
  }
});

watch(
  () => props.language,
  () => view?.dispatch({ effects: languageSlot.reconfigure(languageExtension()) }),
);

onBeforeUnmount(() => view?.destroy());
</script>

<template>
  <div ref="host" class="h-full min-h-0 overflow-hidden select-text" />
</template>
