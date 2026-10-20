import type { Auth } from "@/bindings/Auth";
import type { Collection } from "@/bindings/Collection";
import type { CollectionTree } from "@/bindings/CollectionTree";
import type { Folder } from "@/bindings/Folder";
import type { MoveTarget } from "@/bindings/MoveTarget";
import type { NodeKind } from "@/bindings/NodeKind";
import type { Scripts } from "@/bindings/Scripts";
import type { RequestSpec } from "@/bindings/RequestSpec";
import type { SavedRequest } from "@/bindings/SavedRequest";
import { invoke } from "./invoke";

export const listTree = () => invoke<CollectionTree>("list_tree");

export const createCollection = (name: string) => invoke<Collection>("create_collection", { name });

export const createFolder = (collectionId: string, parentId: string | null, name: string) =>
  invoke<Folder>("create_folder", { collectionId, parentId, name });

export const saveRequest = (spec: RequestSpec, collectionId: string, folderId: string | null) =>
  invoke<SavedRequest>("save_request", { spec, collectionId, folderId });

export const renameNode = (kind: NodeKind, id: string, name: string) =>
  invoke<void>("rename_node", { kind, id, name });

export const deleteNode = (kind: NodeKind, id: string) => invoke<void>("delete_node", { kind, id });

export const moveNode = (kind: NodeKind, id: string, target: MoveTarget) =>
  invoke<void>("move_node", { kind, id, target });

export const setNodeAuth = (kind: NodeKind, id: string, auth: Auth) =>
  invoke<void>("set_node_auth", { kind, id, auth });

export const setNodeSettings = (kind: NodeKind, id: string, scripts: Scripts, docs: string) =>
  invoke<void>("set_node_settings", { kind, id, scripts, docs });

export const getCollectionVariables = (id: string) =>
  invoke<[string, string][]>("get_collection_variables", { id });

export const setCollectionVariables = (id: string, variables: [string, string][]) =>
  invoke<void>("set_collection_variables", { id, variables });
