import type { NewExample } from "@/bindings/NewExample";
import type { ResponseExample } from "@/bindings/ResponseExample";
import { invoke } from "./invoke";

export const saveExample = (example: NewExample) =>
  invoke<ResponseExample>("save_example", { example });

export const getExample = (id: string) => invoke<ResponseExample>("get_example", { id });

export const renameExample = (id: string, name: string) =>
  invoke<void>("rename_example", { id, name });

export const deleteExample = (id: string) => invoke<void>("delete_example", { id });
