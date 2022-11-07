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
