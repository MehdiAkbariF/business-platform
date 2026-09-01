# ADR 0001: Modular Monolith Architecture

## Status
Accepted

## Context
The platform requires strong domain boundaries, clear dependency control, and high performance without the operational complexity of distributed microservices.

## Decision
Adopt a Modular Monolith structure split into explicit crates:
- `domain`: Pure business rules, zero I/O or framework dependencies.
- `application`: Use cases, service orchestrators, and port definitions.
- `infrastructure`: Concrete adapters (SQLx/Postgres, Redis, S3).
- `api`: Axum HTTP routing, middleware, and request/response DTOs.
- `shared`: Common immutable kernel types.

## Consequences
Enforces compile-time separation of concerns and prevents domain logic pollution.