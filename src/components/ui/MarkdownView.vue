<script setup lang="ts">
import { computed } from "vue";
import { openLink } from "@/api/app";
import { useToastsStore } from "@/stores/toasts";
import { isWebLink, renderMarkdown } from "@/utils/markdown";

const props = defineProps<{ source: string }>();
const toasts = useToastsStore();
const content = computed(() => renderMarkdown(props.source));

// Links open in the system browser; the app window itself never navigates away.
function onClick(event: MouseEvent) {
  const link = (event.target as HTMLElement).closest<HTMLElement>("[data-link]");
  if (!link) return;
  event.preventDefault();
  const href = link.dataset.link ?? "";
  if (isWebLink(href)) void toasts.guard(() => openLink(href));
}
</script>

<template>
  <div class="md-view select-text" @click="onClick">
    <component :is="content" />
  </div>
</template>

<style scoped>
.md-view :deep(.md) {
  display: grid;
  gap: 8px;
  font-size: 13px;
  line-height: 21px;
  max-width: 72ch;
}
.md-view :deep(h1),
.md-view :deep(h2),
.md-view :deep(h3) {
  margin: 6px 0 0;
  font-weight: 600;
  text-wrap: balance;
}
.md-view :deep(h1) {
  font-size: 17px;
  line-height: 24px;
}
.md-view :deep(h2) {
  font-size: 15px;
  line-height: 22px;
}
.md-view :deep(h3),
.md-view :deep(h4) {
  font-size: 13px;
}
.md-view :deep(p),
.md-view :deep(ul),
.md-view :deep(ol) {
  margin: 0;
  color: var(--fg-muted);
}
.md-view :deep(ul),
.md-view :deep(ol) {
  padding-left: 20px;
}
.md-view :deep(a) {
  color: var(--accent);
  text-decoration: underline;
  text-underline-offset: 2px;
  cursor: pointer;
}
.md-view :deep(code) {
  font: 12px var(--font-code);
  background: var(--surface-2);
  border-radius: 4px;
  padding: 1px 4px;
}
.md-view :deep(pre) {
  margin: 0;
  overflow-x: auto;
  background: var(--surface-2);
  border-radius: 6px;
  padding: 10px 12px;
}
.md-view :deep(pre code) {
  background: none;
  padding: 0;
}
.md-view :deep(blockquote) {
  margin: 0;
  padding-left: 10px;
  border-left: 2px solid var(--border);
  color: var(--fg-muted);
}
.md-view :deep(table) {
  border-collapse: collapse;
  font-size: 12px;
}
.md-view :deep(th),
.md-view :deep(td) {
  border: 1px solid var(--border);
  padding: 3px 8px;
  text-align: left;
}
.md-view :deep(th) {
  background: var(--surface-2);
  font-weight: 500;
}
.md-view :deep(hr) {
  border: 0;
  border-top: 1px solid var(--border);
  width: 100%;
}
.md-view :deep(.md-image) {
  color: var(--fg-subtle);
  font-style: italic;
}
</style>
