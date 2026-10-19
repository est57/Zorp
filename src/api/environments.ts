import type { Environment } from "@/bindings/Environment";
import { invoke } from "./invoke";

export const listEnvironments = () => invoke<Environment[]>("list_environments");

export const saveEnvironment = (environment: Environment) =>
  invoke<Environment>("save_environment", { environment });

export const deleteEnvironment = (id: string) => invoke<void>("delete_environment", { id });

export const setActiveEnvironment = (id: string | null) =>
  invoke<void>("set_active_environment", { id });
