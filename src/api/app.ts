import type { AppInfo } from "@/bindings/AppInfo";
import { invoke } from "./invoke";

export const getAppInfo = () => invoke<AppInfo>("app_info");

export const openLink = (url: string) => invoke<void>("open_link", { url });
