Self-signed certificates used only by unit tests. They carry no trust and must never be used outside tests.

- `ca-cert.pem`, `client-cert.pem`, `client-key.pem`: certificate loading tests.
- `ws-ca.pem` signs `ws-server.pem` (localhost) and `ws-client.pem`; used by the WebSocket TLS and mTLS tests. The CA private key was discarded.
