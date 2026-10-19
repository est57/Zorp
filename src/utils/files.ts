import { open } from "@tauri-apps/plugin-dialog";

export async function pickFile(
  filters?: { name: string; extensions: string[] }[],
): Promise<string | null> {
  const selected = await open({ multiple: false, directory: false, filters });
  return typeof selected === "string" ? selected : null;
}

export const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;
