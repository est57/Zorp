import { watchEffect } from "vue";
import { useLayoutStore } from "@/stores/layout";

export function useTheme() {
  const layout = useLayoutStore();
  watchEffect(() => {
    const root = document.documentElement;
    if (layout.theme === "system") root.removeAttribute("data-theme");
    else root.dataset.theme = layout.theme;
  });
}
