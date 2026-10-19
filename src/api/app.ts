import type { AppInfo } from "@/bindings/AppInfo";
import { invoke } from "./invoke";

export const getAppInfo = () => invoke<AppInfo>("app_info");
