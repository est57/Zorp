import type { OAuth2Config } from "@/bindings/OAuth2Config";
import type { TokenStatus } from "@/bindings/TokenStatus";
import { invoke } from "./invoke";

export const getOAuthToken = (config: OAuth2Config, collectionId: string | null) =>
  invoke<TokenStatus>("oauth_get_token", { config, collectionId });

export const cancelOAuth = () => invoke<void>("oauth_cancel");

export const oauthTokenStatus = (tokenKey: string) =>
  invoke<TokenStatus>("oauth_token_status", { tokenKey });

export const clearOAuthToken = (tokenKey: string) =>
  invoke<void>("oauth_clear_token", { tokenKey });
