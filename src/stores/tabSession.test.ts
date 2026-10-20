import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { newRequest } from "@/utils/request";
import { leaveTabSession, restoreTabSession, saveTabSession } from "./tabSession";
import { useTabsStore } from "./tabs";

describe("tab session", () => {
  beforeEach(() => {
    localStorage.clear();
    setActivePinia(createPinia());
  });

  it("keeps open tabs per workspace", () => {
    const tabs = useTabsStore();
    restoreTabSession("w1");
    tabs.openNew(newRequest({ name: "Login" }));
    tabs.openNew(newRequest({ name: "Profil" }));
    tabs.tabs[0].draft.url = "https://api.contoh.id/login";
    leaveTabSession();
    tabs.reset();

    restoreTabSession("w2");
    expect(tabs.tabs).toHaveLength(0);
    tabs.openNew(newRequest({ name: "Lain" }));
    saveTabSession();

    restoreTabSession("w1");
    expect(tabs.tabs.map((t) => t.draft.name)).toEqual(["Login", "Profil"]);
    expect(tabs.tabs[0].draft.url).toBe("https://api.contoh.id/login");
    expect(tabs.active?.draft.name).toBe("Profil");
  });

  it("ignores damaged saved data", () => {
    localStorage.setItem("zorp.tabs.w1", '{"tabs":[{"id":1}],"activeId":5}');
    restoreTabSession("w1");
    expect(useTabsStore().tabs).toEqual([]);
    localStorage.setItem("zorp.tabs.w1", "bukan json");
    restoreTabSession("w1");
    expect(useTabsStore().activeId).toBeNull();
  });
});
