import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { SIDEBAR_MAX, SIDEBAR_MIN, clampSidebar, useLayoutStore } from "./layout";

describe("layout store", () => {
  beforeEach(() => {
    localStorage.clear();
    setActivePinia(createPinia());
  });

  it("clamps sidebar width to the allowed range", () => {
    expect(clampSidebar(10)).toBe(SIDEBAR_MIN);
    expect(clampSidebar(9999)).toBe(SIDEBAR_MAX);
    expect(clampSidebar(300.4)).toBe(300);
  });

  it("collapses the sidebar when the active view is selected again", () => {
    const layout = useLayoutStore();
    layout.selectView("collections");
    expect(layout.sidebarOpen).toBe(false);
    layout.selectView("history");
    expect(layout.sidebarOpen).toBe(true);
    expect(layout.view).toBe("history");
  });

  it("cycles theme preference", () => {
    const layout = useLayoutStore();
    layout.cycleTheme();
    expect(layout.theme).toBe("light");
    layout.cycleTheme();
    expect(layout.theme).toBe("dark");
    layout.cycleTheme();
    expect(layout.theme).toBe("system");
  });

  it("ignores corrupted stored preferences", () => {
    localStorage.setItem("zorp.layout", "{not json");
    setActivePinia(createPinia());
    expect(useLayoutStore().sidebarWidth).toBe(260);
  });
});
