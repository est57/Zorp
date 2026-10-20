//! OpenAPI documents used by the importer tests.

pub const OAS3: &str = r##"
openapi: 3.0.3
info: { title: Petstore, version: 1.0.27 }
servers:
  - url: https://{region}.petstore.dev/api/v3/
    variables: { region: { default: eu } }
tags: [{ name: store }, { name: pet }]
security: [{ bearerAuth: [] }]
paths:
  /pet/{petId}:
    parameters:
      - { name: petId, in: path, required: true, schema: { type: integer, example: 7 } }
    get:
      tags: [pet]
      summary: Find pet by ID
      parameters:
        - { name: fields, in: query, schema: { type: string } }
        - { name: lang, in: query, required: true, schema: { type: string, enum: [id, en] } }
        - { name: X-Trace, in: header, schema: { type: string, format: uuid } }
      responses: { 200: { description: ok } }
    delete:
      tags: [pet]
      operationId: deletePet
      security: []
      responses: { 204: { description: gone } }
  /pet:
    post:
      tags: [pet]
      summary: Add a pet
      requestBody:
        content:
          application/json:
            schema: { $ref: "#/components/schemas/Pet" }
      responses: { 200: { description: ok } }
  /pet/{petId}/upload:
    post:
      tags: [pet]
      summary: Upload image
      requestBody:
        content:
          multipart/form-data:
            schema:
              type: object
              properties:
                file: { type: string, format: binary }
                note: { type: string, example: front }
      responses: { 200: { description: ok } }
  /store/inventory:
    get:
      tags: [store]
      summary: Inventory
      security: [{ apiKey: [] }]
      responses: { 200: { description: ok } }
  /health:
    get: { responses: { 200: { description: ok } } }
components:
  securitySchemes:
    bearerAuth: { type: http, scheme: bearer }
    apiKey: { type: apiKey, in: header, name: X-Api-Key }
  schemas:
    Pet:
      type: object
      required: [name]
      properties:
        id: { type: integer, readOnly: true }
        name: { type: string, example: Rex }
        tags: { type: array, items: { $ref: "#/components/schemas/Tag" } }
        parent: { $ref: "#/components/schemas/Pet" }
        status: { type: string, enum: [available, sold] }
    Tag:
      type: object
      properties: { label: { type: string } }
"##;

pub const SWAGGER2: &str = r##"{
  "swagger": "2.0",
  "info": { "title": "Legacy", "version": "1" },
  "host": "legacy.example.com", "basePath": "/v1/", "schemes": ["http"],
  "securityDefinitions": { "oauth": { "type": "oauth2", "flow": "application", "tokenUrl": "https://auth.example.com/token", "scopes": {} } },
  "paths": {
    "/users": {
      "post": {
        "summary": "Create user",
        "parameters": [{ "name": "body", "in": "body", "schema": { "$ref": "#/definitions/User" } }],
        "security": [{ "oauth": ["users:write"] }]
      }
    },
    "/login": {
      "post": {
        "summary": "Login",
        "consumes": ["application/x-www-form-urlencoded"],
        "parameters": [
          { "name": "email", "in": "formData", "type": "string", "format": "email", "required": true },
          { "name": "remember", "in": "formData", "type": "boolean" }
        ]
      }
    }
  },
  "definitions": { "User": { "type": "object", "properties": { "email": { "type": "string", "format": "email" }, "age": { "type": "integer" } } } }
}"##;
