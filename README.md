# auth-plus-billing

[![codecov](https://codecov.io/gh/auth-plus/auth-plus-billing/branch/main/graph/badge.svg?token=PO6CQJDQJH)](https://codecov.io/gh/auth-plus/auth-plus-billing)

[![Known Vulnerabilities](https://snyk.io/test/github/auth-plus/auth-plus-billing/badge.svg)](https://snyk.io/test/github/auth-plus/auth-plus-billing)

A billing system should be able to

- Create an invoice with items that must have a description, a quantity, and an amount
- Pay an invoice by charging with a payment method previously registered
- Contest/Cancel/Refund an invoice
- Change Payment Gateway dynamically
- Register a payment method like credit card or pix and make one of them default
- List all transactions made by a user with filter as period, amount, items, and so on

## Development Requirements

- Docker
- Docker Compose

### Commands (Docker)

```bash
make dev # create database and run migrations and attach a bash on container
cargo run # download all dependencies and start http server (only works on make dev)
cargo run --bin kafka # download all dependencies and start kafka consumer server (only works on make dev)
cargo run --bin cronjob # download all dependencies and start cronjob (only works on make dev)
cargo test # run integration and unit tests (only works on make dev)
cargo clippy # run lint (don't work on make dev)
cargo fmt # run formatter (don't work on make dev)
cargo audit # run audit on packages (don't work on make dev), see https://crates.io/crates/cargo-audit
```

### Commands (nix)

```bash
nix build
nix develop # enter dev shell
```

## Documentation

### Model Entity Relation

![diagram made by DBeaver](/db/MER.png "Database Diagram")

### Machine state of invoice

1. Creating an invoice: `POST /invoice` -> **draft**
2. Finish building invoice and try to charge: `POST /charge` -> **pending** -> Go to Kafka
3. Receive a TOPIC on Kafka to try to charge:
    - `TOPIC charge_invoice` -> Gateway success -> **paid**
    - `TOPIC charge_invoice` -> Gateway fail -> **charged_with_error**
4. A cronjob eventually gets all invoices in **charged_with_error** status and tries to charge again
5. A user can:
    - Cancel: `PATCH /invoice` -> Gateway success -> **canceled**
    - Refund before 7 days: `PATCH /invoice` -> Gateway success -> **refunded**
    - Contest by fraud: `PATCH /invoice` -> Gateway success -> **in_protest** -> **chargeback**

All flows that involve Gateway that change of status is made by a webhook.

### Difference between Charge/Invoice/PaymentMethod

An invoice is a list of items containing a description, amount, and quantity. A charge is the acting of register a invoice in a payment gateway. We currently track a lifetime of an invoice by using the column status. A payment method is like credit-card or pix key.

### Why Payment Gateway should be dynamic?

There's no precise answer to this, but could be: a cost for each transaction, temporary unavailability, payment method issue (pix only in brazil), batch reconciliation distribution,  etc.

## Dev Tips
```bash
cargo build # update all crates
cargo clean # clean the target folder
cargo audit # verify vulnerabilities of crates (may need this: https://crates.io/crates/cargo-audit)
```

## TODO

### Development

- Add [GraphQL](https://github.com/graphql-rust/juniper)
- Add [Mutating Test](https://github.com/llogiq/mutagen)
- Add load testing with [k6](https://k6.io/docs/)

### Business

- Add preferable system for payment gateway as Vindi, Iugu, PagSeguro, or Paypal
- Add a webhook for each payment-gateway

### Security

- Add [grype](https://github.com/anchore/grype) for security scan (SAST)
- Add [OWASP ZAP](https://owasp.org/www-project-zap/) to scan vulnerabilities (DAST)
