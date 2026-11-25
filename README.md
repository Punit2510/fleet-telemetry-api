# Fleet Telemetry Ingestion API

A Rust REST API for registering fleet devices, ingesting telemetry batches, querying telemetry, and managing alert rules. Built with Axum, Tokio, SQLx and PostgreSQL.

## Features

- Device registration and listing
- Batch telemetry ingestion with a configurable maximum batch size
- Telemetry filtering by device, metric and time range
- Alert-rule creation and listing
- JWT bearer authentication and role-based access (`admin`, `ingest`, `viewer`)
- PostgreSQL migrations, structured tracing, health check and typed JSON errors
- Docker Compose, CI, unit/integration tests and a Criterion benchmark

## Repository layout

```text
.
├── .github/workflows/ci.yml
├── benches/ingestion.rs
├── migrations/0001_init.sql
├── src/
│   ├── routes/
│   ├── alerts.rs
│   ├── auth.rs
│   ├── config.rs
│   ├── db.rs
│   ├── error.rs
│   ├── logging.rs
│   ├── models.rs
│   ├── state.rs
│   └── validation.rs
├── tests/
└── Dockerfile
```

## Requirements

- Rust stable toolchain (edition 2021)
- Docker and Docker Compose (recommended for PostgreSQL)
- PostgreSQL 14+ if running the database outside Docker

## Run with Docker Compose

1. Clone the repository.
2. Copy `.env.example` to `.env`.
3. Change `JWT_SECRET` to a random value of at least 32 characters and change the bootstrap admin password.
4. Start the stack:

```bash
docker compose up --build
```

The API listens on `http://localhost:8080`. The database schema is migrated automatically at startup.

## Run locally

Start PostgreSQL first, then configure environment variables:

```bash
cp .env.example .env
# Edit DATABASE_URL, JWT_SECRET and bootstrap credentials
docker compose up -d db
cargo run
```

Run checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo bench --bench ingestion
```

## Configuration

| Variable | Required | Default | Description |
|---|---:|---|---|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `JWT_SECRET` | Yes | — | Secret of at least 32 characters |
| `BIND_ADDR` | No | `0.0.0.0:8080` | HTTP bind address |
| `JWT_TTL_SECONDS` | No | `3600` | JWT lifetime |
| `DB_MAX_CONNECTIONS` | No | `10` | SQL connection pool size |
| `MAX_BATCH_SIZE` | No | `1000` | Maximum telemetry events per request |
| `LOG_FORMAT` | No | `pretty` | Set to `json` for JSON logs |
| `BOOTSTRAP_ADMIN_EMAIL` | No | — | Set together with the password to create the initial admin |
| `BOOTSTRAP_ADMIN_PASSWORD` | No | — | Minimum 12 characters |

Bootstrap creation is idempotent: an existing email is not overwritten. To add additional users, provision them through a controlled administrative process; this starter intentionally does not expose public user registration.

## Authentication and roles

Obtain an access token:

```bash
curl -s http://localhost:8080/auth/login \
  -H 'content-type: application/json' \
  -d '{"email":"admin@example.com","password":"change-this-password-123"}'
```

Pass the returned token as `Authorization: Bearer <token>`.

| Role | Permissions |
|---|---|
| `admin` | Register devices, list devices, ingest/query telemetry, manage alert rules |
| `ingest` | List devices, ingest/query telemetry |
| `viewer` | Read devices, telemetry and alert rules |

## API endpoints

All timestamps are ISO-8601/RFC 3339. JSON request bodies are validated.

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/health` | No | Database-backed health check |
| `POST` | `/auth/login` | No | Exchange credentials for JWT |
| `POST` | `/devices` | Admin | Register a device |
| `GET` | `/devices` | Yes | List devices |
| `POST` | `/telemetry` | Admin, ingest | Ingest a batch |
| `GET` | `/telemetry` | Yes | Query telemetry |
| `POST` | `/alert-rules` | Admin | Create an alert rule |
| `GET` | `/alert-rules` | Admin, viewer | List alert rules |

### Register a device

```bash
curl -X POST http://localhost:8080/devices \
  -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"name":"Truck 42","fleet_id":"north-region","external_id":"truck-42","metadata":"{}"}'
```

### Ingest telemetry

```bash
curl -X POST http://localhost:8080/telemetry \
  -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  --data-binary @tests/fixtures/telemetry_batch.json
```

The ingestion operation is transactional: if any event references an unknown device or fails insertion, the whole batch rolls back.

### Query telemetry

```bash
curl -G http://localhost:8080/telemetry \
  -H "Authorization: Bearer $TOKEN" \
  --data-urlencode "metric=engine_temperature" \
  --data-urlencode "limit=100"
```

Supported query parameters: `device_id`, `metric`, `from`, `to`, and `limit` (1–1000).

### Create an alert rule

```bash
curl -X POST http://localhost:8080/alert-rules \
  -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"name":"Engine hot","metric":"engine_temperature","operator":"gt","threshold":95,"enabled":true}'
```

Supported operators: `gt`, `gte`, `lt`, `lte`, `eq`. This API stores rule definitions; evaluating rules and dispatching notifications is an extension point, not part of this ingestion service.

## Architecture

- **HTTP layer:** Axum routes parse JSON/query parameters and enforce request size limits. Tower HTTP adds request tracing.
- **Application layer:** handlers validate inputs, enforce role permissions and coordinate transactions.
- **Persistence:** SQLx connection pool and versioned SQL migrations. SQL statements are parameterized.
- **Security:** JWTs are signed with an environment-provided secret; passwords are bcrypt-hashed; roles are checked per operation. Secrets are redacted from `Config` debug output.
- **Operations:** `/health` verifies PostgreSQL connectivity; `tracing` supports human-readable or JSON logs; Docker and GitHub Actions provide repeatable delivery.

## GitHub upload

This ZIP is already a normal Git repository layout (without a nested parent folder or generated `target/` directory). Extract it, then:

```bash
git init
git add .
git commit -m "Initial Fleet Telemetry API"
git branch -M main
git remote add origin https://github.com/YOUR_USERNAME/fleet-telemetry-api.git
git push -u origin main
```

Do not commit `.env`; it is ignored by `.gitignore`. Rotate all sample credentials before deployment.

## Notes

This is a production-oriented starter, not a substitute for a deployment security review. Before production, add TLS termination, secret management, rate limiting, audit events, user provisioning/rotation, alert evaluation workers, retention/partitioning and observability dashboards. Review the role model and tenant isolation against your fleet's requirements.
