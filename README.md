# auth-plus-billing

[![codecov](https://codecov.io/gh/auth-plus/auth-plus-billing/branch/main/graph/badge.svg?token=PO6CQJDQJH)](https://codecov.io/gh/auth-plus/auth-plus-billing)

A billing system, it should be able to

- Create a invoice with item that musta have a description and an amount
- Pay a invoice by charging with a payment method preivou registered
- Register a payment method like credit-card
- List a transaction made by a user like payments and funding

## Development

Requirements:

- Docker
- Docker Compose

### Commands

```bash
make dev # create database and run migrations
cargon run # download all dependencies and start http server
```

## Document

Some guides of business model

### Machine state of invoice

1. Creating a invoice: `POST /invoice` -> **draft**
2. Finish the building invoice and try to charge: `POST /charge` -> **pending** -> Go to kafka
3. Receive a TOPIC on kafka to try to charge:
    - `TOPIC charge_invoice` -> Gateway success -> **paid**
    - `TOPIC charge_invoice` -> Gateway fail -> **charged_with_error**
4. A cronjob eventually get all invoices in **charged_with_error** status and try to charge again
5. A user can:
    - Cancel: `PATCH /invoice` -> Gateway success -> **canceled**
    - Refund before 7 days: `PATCH /invoice` -> Gateway success -> **refunded**
    - Contest by fraud: `PATCH /invoice` -> Gateway success -> **in_protest** -> **chargeback**

All flows that envolves Gateway that change of status is made by a webhook.

### Difference between Charge/Invoice/PaymentMethod

A invoice is list of itens each one with a description, amount and quantity. A charge is the act register in a payment gateway. We currently track a lifetime of a invoice by using the column status. A PaymentMethod is like credit-card or pix key
