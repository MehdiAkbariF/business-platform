# ADR 0002: Uniform Safe Error Handling

## Status
Accepted

## Context
Exposing database errors, stack traces, or secrets to end-users creates security vulnerabilities.

## Decision
Implement a central `AppError` mapped via `IntoResponse` in the API layer. Internal errors are masked to a generic safe message and logged with structured tracing at `ERROR` level.

## Consequences
API responses are predictable and consistent; sensitive diagnostic logs never leave server boundaries.