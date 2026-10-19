import type { CookieInfo } from "@/bindings/CookieInfo";
import { invoke } from "./invoke";

export const listCookies = () => invoke<CookieInfo[]>("list_cookies");

export const deleteCookie = (cookie: Pick<CookieInfo, "domain" | "path" | "name">) =>
  invoke<void>("delete_cookie", { domain: cookie.domain, path: cookie.path, name: cookie.name });

export const clearCookies = (domain: string | null) => invoke<void>("clear_cookies", { domain });
