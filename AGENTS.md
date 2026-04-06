# AGENTS.md — Backend

## Project

Pockerra backend: Rust backend using Axum + Socketioxide for a planning poker app.

- Treat backend and frontend contracts as one surface
- When changing events, update both backend payloads and frontend types/stores
- When changing room/game models, verify Rust serde output still matches frontend expectations
- Keep deck labels and values synchronized between backend and frontend
- Prefer minimal changes that match the current implementation, not the unimplemented future plan
- There is a localStorage state on the frontend — be careful that backend state doesn't conflict with it
