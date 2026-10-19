export const isMac = typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
export const modKey = isMac ? "⌘" : "Ctrl";
