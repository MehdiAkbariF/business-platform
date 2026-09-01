# Architecture Overview

The Business Discovery Platform follows Hexagonal / Clean Architecture principles:

```text
       [ HTTP Clients ]
              |
              v
        [ API Crate ] (Axum, Middleware, OpenAPI)
              |
              v
   [ Application Crate ] (Ports, Orchestration, Error types)
        /           \
       v             v
[ Domain Crate ]   [ Infrastructure Crate ]
(Pure Entities)    (PostgreSQL, Redis, MinIO)
#### `docs/development/local_setup.md`
```markdown
# Local Development Setup

1. Copy `.env.example` to `.env`:
   ```bash
   cp .env.example .env