import { defineStore } from "pinia";
import { reactive, toRefs, watch } from "vue";
import { readJson, writeJson } from "@/utils/storage";

export type SidebarView = "collections" | "history" | "environments";
export type ThemePreference = "system" | "light" | "dark";

export const SIDEBAR_MIN = 200;
export const SIDEBAR_MAX = 420;
export const SIDEBAR_DEFAULT = 260;
const STORAGE_KEY = "zorp.layout";
const THEMES: ThemePreference[] = ["system", "light", "dark"];

interface LayoutState {
  sidebarWidth: number;
  sidebarOpen: boolean;
  view: SidebarView;
  theme: ThemePreference;
  splitRatio: number;
}

const defaults: LayoutState = {
  sidebarWidth: SIDEBAR_DEFAULT,
  sidebarOpen: true,
  view: "collections",
  theme: "system",
  splitRatio: 0.45,
};

export const clampSidebar = (width: number) =>
  Math.round(Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, width)));

export const useLayoutStore = defineStore("layout", () => {
  const state = reactive<LayoutState>(readJson(STORAGE_KEY, defaults));
  state.sidebarWidth = clampSidebar(state.sidebarWidth);
  if (!THEMES.includes(state.theme)) state.theme = "system";

  watch(state, (value) => writeJson(STORAGE_KEY, value), { deep: true });

  function setSidebarWidth(width: number) {
    state.sidebarWidth = clampSidebar(width);
  }

  function toggleSidebar(open = !state.sidebarOpen) {
    state.sidebarOpen = open;
  }

  function selectView(view: SidebarView) {
    if (state.view === view && state.sidebarOpen) {
      state.sidebarOpen = false;
      return;
    }
    state.view = view;
    state.sidebarOpen = true;
  }

  function setSplitRatio(ratio: number) {
    state.splitRatio = Math.min(0.8, Math.max(0.2, ratio));
  }

  function cycleTheme() {
    state.theme = THEMES[(THEMES.indexOf(state.theme) + 1) % THEMES.length];
  }

  return {
    ...toRefs(state),
    setSidebarWidth,
    toggleSidebar,
    selectView,
    setSplitRatio,
    cycleTheme,
  };
});
