<!--
=== SYNC IMPACT REPORT ===
Version change: N/A (initial population) → 1.0.0
Added sections:
  - Core Principles (I–VII, fully populated from template stubs)
  - Architecture Constraints
  - Development Workflow & Quality Gates
  - Governance
Modified principles: All placeholders replaced; no prior content to diff.
Removed sections: None (template had no prior values).
Follow-up TODOs:
  - TODO(RATIFICATION_DATE): Adopt the date 2026-10-30 as the initial ratification date.
    Confirm or correct if the project pre-dates this conversation.
=== END SYNC IMPACT REPORT ===
-->

# Auth Plus Billing Constitution

## Core Principles

### I. Hexagonal Architecture (NON-NEGOTIABLE)

The service MUST strictly follow the Ports & Adapters (Clean Architecture) pattern:

- **Presentation layer** (`src/presentation/`) — driving adapters only: HTTP routes, Kafka consumer,
  CLI cronjob. No business logic here.
- **Use-case layer** (`src/core/usecase/`) — pure domain logic. MUST NOT instantiate SQL
  connections, HTTP clients, or any infrastructure concerns directly. All external I/O MUST be
  accessed through trait objects (`Box<dyn DrivenTrait>`) defined in `src/core/usecase/driven/`.
- **Infrastructure adapters** (`src/core/repository/`, `src/core/gateway/`) — concrete
  implementations of driven ports. SQLx DAOs and the Stripe client live here exclusively.
- **DTOs & Value Objects** (`src/core/dto/`) — domain entities shared across layers.

**Rationale:** Enforcing the boundary keeps business rules testable in isolation (using `mockall`
mocks), decouples the service from infrastructure details, and allows swapping payment gateways or
databases without touching domain logic.

### II. Financial Precision (NON-NEGOTIABLE)

All monetary amounts MUST be represented and processed using `rust_decimal::Decimal`. The use of
`f32` or `f64` for any financial computation, persistence, or API contract is strictly forbidden.

- Database columns for amounts MUST use `DECIMAL` / `NUMERIC` types.
- When interfacing with external gateways that require integer cents, the conversion
  (`Decimal → cents`) MUST happen at the adapter boundary, not in the use case.

**Rationale:** Floating-point arithmetic introduces rounding errors that are unacceptable in
billing systems. `rust_decimal` provides exact decimal arithmetic matching standard accounting
expectations.

### III. Two-Tier Idempotency (NON-NEGOTIABLE)

Every billing-mutating operation that can be triggered by an external caller MUST implement
two-tier idempotency:

1. **Cache tier (Redis/Valkey):** Cache the serialised response under a structured key
   (e.g., `create_invoice:<idempotency_key>`) with an appropriate TTL.
2. **Database tier (PostgreSQL):** The idempotency key MUST be stored with a `UNIQUE` constraint
   in the relevant table (e.g., `invoice.idempotency_key`).

New endpoints or cronjob routines that perform charge or invoice mutations MUST follow this pattern
before being merged.

**Rationale:** Distributed systems and payment retries require safe replay semantics. Duplicate
charges caused by network retries or at-least-once Kafka delivery represent direct financial harm.

### IV. Invoice State Machine Compliance (NON-NEGOTIABLE)

All transitions of `InvoiceStatus` MUST comply with the validated state machine implemented in
`InvoiceUpdateUsecase::validate_update_status_change`. The permitted transitions are:

- `Draft` → `Pending`
- `Pending` → `Paid` | `ChargedWithError`
- `ChargedWithError` → `Paid` | `Uncollectible`
- `Paid` → `Cancelled` | `Refunded` | `InProtest`
- `InProtest` → `Chargeback`

Any attempt to transition to an unlisted target state MUST return a validation error. New status
values MUST be added to the state machine with an explicit transition matrix update, reviewed and
documented.

**Rationale:** Invoice status is a financial audit trail. Invalid transitions can hide fraud,
cause double-charges, or corrupt reconciliation reports.

### V. Bounded Context Isolation (NON-NEGOTIABLE)

This service MUST access exclusively the `billing` PostgreSQL database. Direct queries or
connections to `auth-plus-authentication` or any other bounded context's datastore are strictly
forbidden.

- Cross-context data ingestion MUST occur via asynchronous events (Kafka topics).
- Synchronous inter-service communication, if introduced, MUST go through documented HTTP APIs —
  never through shared database schemas.

**Rationale:** Bounded context isolation guarantees independent deployability and prevents implicit
coupling that would make the invoice/payment data model hostage to another team's schema changes.

### VI. Observability First

All asynchronous and I/O-bound code paths MUST be instrumented:

- Use `#[tracing::instrument(skip(self))]` on use-case public methods.
- Emit `tracing::info!` at the start of each logical step and `tracing::error!` on every error
  branch.
- Preserve OpenTelemetry metric annotations (`C_HTTP_SUCCESS`, `C_HTTP_FAIL`) on all HTTP handlers.
- The Uptrace DSN integration (`opentelemetry-otlp`) MUST remain initialised in the `http` binary.

**Rationale:** Distributed tracing is essential for correlating payment failures across async
boundaries (Kafka, HTTP, cronjob). Removing or bypassing these annotations silently degrades the
on-call operator's visibility.

### VII. Test Isolation via Mocks

Use-case logic MUST be covered by unit tests using `mockall` mock implementations of all driven
ports. Tests MUST NOT depend on a live database, Redis instance, Kafka broker, or Stripe API.

- Integration tests (those requiring infrastructure) MUST be confined to `tests/` integration
  directories and MUST be runnable with `cargo test -- --test-threads=1`.
- `cargo clippy --all-targets --all-features -- -D warnings` MUST pass with zero warnings.
- `cargo fmt --check` MUST pass before any merge.

**Rationale:** Fast, deterministic unit tests catch regressions immediately. Infrastructure
dependencies in unit tests introduce flakiness and slow down the development feedback loop.

## Architecture Constraints

- **Language & Runtime:** Rust (Edition 2021), Tokio async runtime, Actix-web 4. No synchronous
  blocking calls are permitted inside async tasks unless explicitly wrapped in `tokio::task::spawn_blocking`.
- **Three Binary Entry Points:** `http` (REST API on port 5002), `kafka` (event consumer), and
  `cronjob` (CLI via `clap`). New operational entry points MUST follow the same pattern.
- **Gateway Abstraction:** The `GatewayAPI` trait (in `src/core/usecase/driven/gateway.rs`) MUST
  be the only way use cases interact with external payment processors. The `StripeGateway`
  implementation is the current concrete adapter. Additional gateways MUST implement this trait.
- **Async Trait Pattern:** External-facing traits that perform I/O MUST be decorated with
  `#[async_trait]`. Implementations MUST use `#[async_trait]` as well.
- **Error Handling:** Domain errors MUST be represented as typed enums (e.g.,
  `ReadingInvoiceError`, `CreatingChargeError`). Use of `anyhow::Error` is acceptable only at
  adapter boundaries, not inside use cases.

## Development Workflow & Quality Gates

The following gates MUST pass before a feature branch is merged:

1. `cargo check` — no compilation errors.
2. `cargo fmt --check` — code is properly formatted.
3. `cargo clippy --all-targets --all-features -- -D warnings` — zero lint warnings.
4. `cargo test -- --test-threads=1` — all unit and integration tests pass.
5. Any new charging or invoice endpoint MUST include:
   - At least one happy-path unit test using mocks.
   - At least one error-path unit test per distinct failure mode.
6. New Kafka topics consumed MUST be registered in the consumer dispatcher, even as stubs, to
   maintain discoverability.

**Dunning / RCI Cronjob:** The `cargo run --bin cronjob -- rci` routine retries all invoices in
`ChargedWithError` or `Pending` status. Changes to charge eligibility rules MUST update this
routine accordingly.

**Deployment Hygiene:** Environment variables listed in `AGENTS.md § 1.2` MUST be documented for
every new configurable parameter. The `.env` or Docker Compose override is the single source of
truth for local development defaults.

## Governance

This constitution supersedes all other project conventions and practices. It applies to all
contributors — human and AI agents alike.

**Amendment Procedure:**
1. Propose changes by opening a PR that modifies `.specify/memory/constitution.md` directly.
2. Amendments MUST include a rationale explaining why the existing principle is insufficient.
3. Principle removals or incompatible redefinitions require a MAJOR version bump and a migration
   plan describing how existing code will be brought into compliance.
4. Additions or material expansions require a MINOR version bump.
5. Clarifications, wording fixes, and non-semantic refinements require a PATCH bump.

**Compliance Review:**
- All PRs and code-review sessions MUST verify compliance with these principles before approval.
- AI agent sessions operating on this codebase MUST read this constitution and the `AGENTS.md`
  file before generating or modifying any code.
- Complexity additions MUST be justified against the principles above; "it works" is not sufficient
  justification for violating Hexagonal Architecture or the idempotency mandate.

**Version**: 1.0.0 | **Ratified**: 2026-10-30 | **Last Amended**: 2026-10-30
