# AGENTS.md - Auth+ Billing Service

> **Propósito:** Ponto de entrada técnico, guia de arquitetura e manual de regras de negócio para Agentes de IA atuando no microsserviço `auth-plus-billing`.
>
> **Bounded Context:** Ciclo de vida financeiro de faturas (*Invoices*), itens de fatura (*Invoice Items*), métodos de pagamento (*Payment Methods*), cobranças (*Charges*), integração com gateways de pagamento (Stripe), persistência de clientes/integrações e rotinas de retentativa (*Dunning / RCI*).

---

## 1. Visão Geral e Stack Tecnológica

O `auth-plus-billing` gerencia cobranças recorrentes e pontuais, consolida itens faturáveis em rascunhos de fatura, integra-se a provedores externos de pagamento e reconcilia transações no ecossistema `auth-plus`.

| Camada / Componente | Tecnologia | Detalhes & Configuração |
| :--- | :--- | :--- |
| **Linguagem / Runtime** | Rust (Edition 2021) | Tokio async runtime, Actix-web 4 |
| **Banco Relacional** | PostgreSQL (Docker: 17.6) | Conexão via `sqlx::PgPool` (banco `billing`) |
| **Cache & Idempotência** | Redis / Valkey (Docker: 7-alpine) | Caching de respostas idempotentes (`redis` crate) |
| **Mensageria** | Apache Kafka | Consumo assíncrono via `rdkafka` |
| **Observabilidade** | OpenTelemetry + Uptrace | Métricas (`opentelemetry::metrics`), Tracing (`tracing-actix-web`) |
| **Gateway de Pagamento** | Stripe API | Integração via `reqwest` (`/v1/customers`, `/v1/payment_methods`, `/v1/payment_intents`) |

### 1.1. Binários do Projeto (`src/bin`)
- **`http`** (`src/bin/http.rs`): Servidor REST Actix-web principal (porta padrão `5002`). Inicializa o Uptrace e expõe rotas de faturas, cobranças, usuários, métodos de pagamento e health check.
- **`kafka`** (`src/bin/kafka.rs`): Consumidor assíncrono contínuo de eventos do cluster Kafka.
- **`cronjob`** (`src/bin/cronjob.rs`): CLI via `clap` para rotinas agendadas (ex.: comando `rci`).

### 1.2. Variáveis de Ambiente

| Variável | Obrigatória | Padrão / Exemplo | Descrição |
| :--- | :---: | :--- | :--- |
| `APP_NAME` | Sim | `auth-plus-billing` | Nome do serviço para logs e métricas |
| `RUST_ENV` | Sim | `development` / `production` | Ambiente de execução |
| `PORT` | Sim | `5002` | Porta TCP do servidor HTTP |
| `DATABASE_URL` | Sim | `postgres://root:db_password@database:5432/billing` | String de conexão PostgreSQL |
| `REDIS_HOST` | Sim | `cache` (ou `cache:6379`) | Host da instância Redis/Valkey |
| `KAFKA_HOST` | Sim | `kafka:9092` | Brokers de bootstrap do Kafka |
| `STRIPE_KEY` | Sim | `test-key` / `sk_test_...` | Chave de API secreta da Stripe |
| `STRIPE_BASE_URL` | Não | `https://api.stripe.com` | Base URL da Stripe (alterável em testes) |
| `UPTRACE_DSN` | Não | `https://<token>@uptrace:4317/<project>` | DSN OpenTelemetry para Uptrace |

---

## 2. Estrutura Arquitetural do Código

O projeto segue estritamente a **Arquitetura Hexagonal (Ports & Adapters / Clean Architecture)**:

```
src/
├── bin/                       # Pontos de entrada executáveis (http, kafka, cronjob)
├── config/                    # Configurações de ambiente, database, cache, kafka, uptrace
├── presentation/              # Camada condutora (Driving Adapters)
│   ├── http/                  # Servidor Actix-web e rotas REST (/charge, /invoice, /payment_method, /user)
│   ├── kafka/                 # Consumidor de mensagens e dispatcher de eventos
│   └── cronjob/               # Comandos CLI (retry_charging_invoices)
└── core/                      # Domínio e Casos de Uso (Hexágono Central)
    ├── dto/                   # Entidades e Value Objects (User, Invoice, Charge, PaymentMethod, Gateway)
    ├── gateway/               # Implementações concretas de gateways externos (StripeGateway)
    ├── repository/            # Implementações concretas SQLx (PostgreSQL DAOs e ORM helpers)
    └── usecase/               # Regras de negócio da aplicação
        ├── driven/            # Ports dirigidos (Traits de IO: CreatingInvoice, ReadingUser, GatewayAPI...)
        ├── charge/            # Casos de uso de cobrança
        ├── invoice/           # Casos de uso de criação, listagem e atualização de faturas
        ├── payment_method/    # Casos de uso de cadastro de métodos de pagamento
        └── user/              # Casos de uso de sincronização de usuários e clientes no gateway
```

---

## 3. Modelo de Dados e Relacionamentos (PostgreSQL)

```mermaid
erDiagram
    user ||--o{ invoice : "possui"
    user ||--o{ payment_method : "possui"
    user ||--o{ gateway_integration : "possui"
    invoice ||--|{ invoice_item : "contém"
    invoice ||--o{ charge : "gera"
    payment_method ||--o{ charge : "usado_em"
    payment_method ||--o{ gateway_integration : "vinculado"
    gateway ||--o{ gateway_integration : "integra"

    user {
        UUID id PK
        UUID external_id UK "ID externo vindo do auth-plus-authentication"
        TIMESTAMPTZ created_at
    }

    invoice {
        UUID id PK
        UUID user_id FK
        TEXT status "draft | pending | charged_with_error | paid | cancelled | uncollectible..."
        TEXT idempotency_key UK
        TIMESTAMPTZ created_at
    }

    invoice_item {
        UUID id PK
        UUID invoice_id FK
        TEXT description
        INT quantity
        DECIMAL amount
        VARCHAR currency "ex: BRL, USD"
        TIMESTAMPTZ created_at
    }

    payment_method {
        UUID id PK
        UUID user_id FK
        BOOLEAN is_default
        TEXT method "credit_card | pix"
        JSONB info "last4digit, flag, expire_data, key..."
        TIMESTAMPTZ created_at
    }

    charge {
        UUID id PK
        UUID invoice_id FK
        UUID payment_method_id FK
        TEXT status "progress | succeed | failed"
        TEXT external_id "ID de transação retornado pelo Gateway"
        TIMESTAMPTZ created_at
    }

    gateway {
        UUID id PK
        TEXT name UK "ex: stripe"
        INTEGER priority
        TIMESTAMPTZ deleted_at
        TIMESTAMPTZ created_at
    }

    gateway_integration {
        UUID id PK
        UUID gateway_id FK
        UUID user_id FK
        UUID payment_method_id FK
        TEXT gateway_external_user_id "Customer ID no Gateway (ex: cus_...)"
        TEXT gateway_external_payment_method_id UK "ID do PM no Gateway"
        TIMESTAMPTZ created_at
    }
```

---

## 4. Regras de Negócio e Máquinas de Estado

### 4.1. Máquina de Estados da Fatura (`InvoiceStatus`)

A alteração manual ou via API do status da fatura (`PATCH /invoice`) obedece à matriz estrita implementada em `InvoiceUpdateUsecase::validate_update_status_change`:

```
                 ┌─────────┐
                 │  Draft  │
                 └────┬────┘
                      │
                      ▼
                ┌───────────┐
                │  Pending  │ ◄────────────────────────┐
                └─────┬─────┘                          │
                      │                                │
         ┌────────────┴────────────┐                   │
         ▼                         ▼                   │
  ┌──────────────┐      ┌────────────────────┐         │
  │     Paid     │      │  ChargedWithError  │ ────────┘ (Disparo de Cobrança / Rci)
  └──┬───┬───┬───┘      └─────────┬──────────┘
     │   │   │                    │
     │   │   │                    ▼
     │   │   │          ┌────────────────────┐
     │   │   │          │   Uncollectible    │
     │   │   │          └────────────────────┘
     │   │   │
     │   │   └──────────────────────┐
     │   ▼                          ▼
     │ ┌────────────────┐ ┌───────────────────┐
     │ │    Refunded    │ │     InProtest     │
     │ └────────────────┘ └─────────┬─────────┘
     ▼                              │
┌──────────────┐                    ▼
│  Cancelled   │          ┌───────────────────┐
└──────────────┘          │    Chargeback     │
                          └───────────────────┘
```

#### Matriz de Transições Permitidas (`validate_update_status_change`):
- `Draft` $\rightarrow$ `Pending`
- `Pending` $\rightarrow$ `Paid` ou `ChargedWithError`
- `ChargedWithError` $\rightarrow$ `Paid` ou `Uncollectible`
- `Paid` $\rightarrow$ `Cancelled`, `Refunded` ou `InProtest`
- `InProtest` $\rightarrow$ `Chargeback`
- Todas as outras transições retornam erro de validação: `"Updating to invalid status"`.

### 4.2. Acúmulo de Itens em Fatura Rascunho (`InvoiceInsertItemUsecase`)
Ao receber requisições de criação de fatura (`POST /invoice`):
1. **Verificação de Rascunho Existente:** Se o usuário já possuir uma fatura com status `Draft`, o sistema **não** cria uma nova fatura. Em vez disso, anexa os novos itens à fatura `Draft` existente e retorna `InvoiceInsertItemResponse::Item(Vec<InvoiceItem>)`.
2. **Nova Fatura:** Caso não haja fatura em `Draft`, cria uma nova fatura com status `Draft` contendo os itens informados e retorna `InvoiceInsertItemResponse::Invoice(Invoice)`.
3. **Idempotência em Dois Níveis:**
   - **Cache (Redis):** A chave `create_invoice:{idempotency_key}` armazena a resposta serializada para deduplicação instantânea.
   - **Banco de Dados:** A coluna `invoice.idempotency_key` possui constraint `UNIQUE`.

### 4.3. Ciclo de Cobrança (`ChargeCreateUsecase`)
1. **Status Elegíveis:** Apenas faturas em `Draft`, `Pending` ou `ChargedWithError` podem ser cobradas. Qualquer outro status é rejeitado.
2. **Método de Pagamento:** Recupera o método de pagamento padrão (`is_default = true`) associado ao usuário.
3. **Cálculo de Valor:** O valor total é somado através de `invoice_item.quantity * invoice_item.amount`.
4. **Gateway:** Seleciona o gateway de maior prioridade cadastrado e emite a requisição de débito (no Stripe: converte para centavos em BRL).
5. **Registro e Transição:** Registra a entidade `Charge` com status `Progress` e atualiza a fatura para o status `Pending`.

### 4.4. Rotina de Retentativa de Cobrança (`cronjob rci`)
- **Comando:** `cargo run --bin cronjob -- rci`
- **Operação:** Consulta todas as faturas nos status `ChargedWithError` ou `Pending` através de `list_all_should_be_charged` e reexecuta o usecase de cobrança (`create_charge`) para cada uma.

### 4.5. Consumo de Eventos Kafka
- O consumidor subscreve os tópicos:
  - `USER_CREATED`: Consome `{ external_id, name, email }`. Cadastra o cliente na Stripe via API (`create_customer`), insere o registro na tabela `user` e vincula na tabela `gateway_integration`.
  - `2FA_EMAIL_CREATED`, `2FA_PHONE_CREATED`, `2FA_EMAIL_SENT`, `2FA_PHONE_SENT`, `ORGANIZATION_CREATED`: Tópicos registrados para expansões futuras de billing por consumo/evento (atualmente registrados como stubs informativos).

---

## 5. Contratos de API HTTP (Actix-web)

> [!IMPORTANT]
> Observe rigorosamente as rotas e os nomes de campos no JSON (ex.: `itens` com grafia em português no payload de fatura e `payment_method` com underscore).

### 5.1. `GET /health`
Verificação de liveness/readiness.
- **Resposta:** `200 OK` com corpo `"Ok"`.

### 5.2. `POST /user`
Criação e sincronização de usuário e cliente no gateway.
- **Payload:**
  ```json
  {
    "external_id": "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11",
    "name": "Jane Doe",
    "email": "jane@example.com"
  }
  ```
- **Resposta Sucesso:** `200 OK` com `{ "id": "<uuid>", "external_id": "<uuid>" }`.

### 5.3. `POST /payment_method`
Cadastro e vinculação de método de pagamento ao usuário e gateway.
- **Payload (Cartão de Crédito):**
  ```json
  {
    "external_user_id": "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11",
    "is_default": true,
    "method": "credit_card",
    "info": {
      "CreditCardInfo": {
        "last4digit": "4242",
        "flag": "visa",
        "expire_data": "12/2030",
        "external_id": "tok_123456"
      }
    }
  }
  ```
- **Payload (Pix):**
  ```json
  {
    "external_user_id": "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11",
    "is_default": true,
    "method": "pix",
    "info": {
      "PixInfo": {
        "key": "jane@example.com",
        "external_id": "pix_key_123"
      }
    }
  }
  ```

### 5.4. `POST /invoice`
Criação de fatura ou anexação de itens à fatura rascunho existente.
- **Payload:**
  ```json
  {
    "external_user_id": "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11",
    "idempotency_key": "unique-request-uuid-or-hash-12345",
    "itens": [
      {
        "id": null,
        "description": "Assinatura Mensal Pro",
        "quantity": 1,
        "amount": "99.00",
        "currency": "BRL"
      }
    ]
  }
  ```
- **Resposta:**
  - Se criou fatura nova: `200 OK` com `{"Invoice": {"id": "...", "user_id": "...", "status": "draft", "created_at": "..."}}`.
  - Se anexou a rascunho existente: `200 OK` com `{"Item": [{"id": null, "description": "...", "quantity": 1, "amount": "99.00", "currency": "BRL"}]}`.

### 5.5. `GET /invoice/{user_id}`
Lista todas as faturas pertencentes ao usuário indicado pelo `external_user_id`.
- **Parâmetro de Path:** `user_id` (UUID externo do cliente).
- **Resposta Sucesso:** `200 OK` com `{"invoices": [...]}`.

### 5.6. `PATCH /invoice`
Transiciona o status da fatura seguindo a matriz de estados permitida.
- **Payload:**
  ```json
  {
    "invoice_id": "c1f8a846-950c-4e8c-8515-373e34b12345",
    "status": "pending"
  }
  ```
- **Resposta Sucesso:** `200 OK` com a entidade `Invoice` atualizada.

### 5.7. `POST /charge`
Dispara a cobrança imediata de uma fatura contra o método de pagamento padrão do usuário.
- **Payload:**
  ```json
  {
    "invoice_id": "c1f8a846-950c-4e8c-8515-373e34b12345"
  }
  ```
- **Resposta Sucesso:** `200 OK` com `Charge` (`status: "progress"`).

---

## 6. Instruções de Desenvolvimento, Testes e Verificação

### 6.1. Comandos Cargo
```bash
# Verificação rápida de sintaxe e tipos
cargo check

# Execução da suíte de testes unitários e de integração
cargo test

# Execução de teste específico com logs habilitados
RUST_LOG=info cargo test --test invoice_create -- --nocapture

# Formatação e Linting obrigatórios
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

### 6.2. Execução dos Binários
```bash
# Servidor REST HTTP
cargo run --bin http

# Consumidor de Eventos Kafka
cargo run --bin kafka

# Rotina de retentativa de cobrança (Cronjob RCI)
cargo run --bin cronjob -- rci
```

---

## 7. Diretrizes Críticas para Agentes de IA

1. **Integridade da Camada de Domínio (Port & Adapter):**
   - Nunca instancie conexões SQL ou clientes HTTP diretamente dentro de usecases (`src/core/usecase/`).
   - Todo acesso a banco ou serviços externos nos usecases deve ser tipado como trait object `Box<dyn DrivenTrait>` definido em `src/core/usecase/driven/`.
2. **Contrato de Tipos Monetários:**
   - Valores monetários utilizam obrigatoriamente `rust_decimal::Decimal` (não utilize `f32` ou `f64` para persistência ou cálculos financeiros, a fim de evitar erros de ponto flutuante).
3. **Idempotência Rigorosa:**
   - Qualquer novo endpoint ou rotina de cobrança/faturamento deve suportar chaves de idempotência armazenadas no Redis e validadas por constraints no banco relacional.
4. **Isolamento de Persistência:**
   - Este microsserviço acessa exclusivamente o banco `billing`. Jamais tente consultar ou modificar tabelas de `auth-plus-authentication` ou outros bounded contexts diretamente.
5. **Tratamento de Erros e Observabilidade:**
   - Use as macros `tracing::instrument`, `tracing::info!` e `tracing::error!` para manter correlação distribuída nas chamadas assíncronas.
   - Preservar anotações de métricas OTel (`C_HTTP_SUCCESS`, `C_HTTP_FAIL`).
