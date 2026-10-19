import { open } from "@tauri-apps/plugin-dialog";

export async function pickFile(): Promise<string | null> {
  const selected = await open({ multiple: false, directory: false });
  return typeof selected === "string" ? selected : null;
}

export const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;
