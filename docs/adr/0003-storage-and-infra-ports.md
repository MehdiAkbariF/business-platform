# ADR 0003: Storage & Infrastructure Inversion of Control

## Status
Accepted

## Context
Business domain entities should not be coupled to MinIO or AWS SDK types directly.

## Decision
Expose the `ObjectStoragePort` async trait in `application::ports` and implement it in `infrastructure::storage::S3ObjectStorage`.

## Consequences
MinIO is used for local development and CI, while AWS S3 / Cloudflare R2 can be used in production without any domain code changes.