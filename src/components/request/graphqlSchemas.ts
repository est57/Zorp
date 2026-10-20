import type { GraphQLSchema } from "graphql";

/** Introspected schemas per request, kept for the session only. */
export const schemaCache = new Map<string, GraphQLSchema>();
