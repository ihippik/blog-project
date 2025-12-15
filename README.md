# Blog WASM (Rust + HTTP/gRPC + WebAssembly)

A small demo blog platform built in Rust with:

- **HTTP API (Actix Web)** + middleware (**JWT auth**, **x-request-id**)
- **gRPC API (tonic)** with JWT passed via gRPC metadata (`authorization`)
- **PostgreSQL (sqlx)** + migrations
- **Rust client SDK** supporting **HTTP** and **gRPC** transports
- **WASM frontend** (Rust → WebAssembly) + a simple HTML/JS UI

---

## Features

- User **registration** and **login** (JWT)
- Posts **CRUD** (Create / Read / Update / Delete)
- **Per-user post listing** (by authenticated user)
- Request tracing with **x-request-id**
- Logging in **text** or **JSON**

---

## Project layout (high-level)

- `domain/` — core domain models (`User`, `Post`) and domain errors
- `data/` — repositories (`UserRepository`, `PostRepository`) + Postgres implementations
- `application/` — services (`AuthService`, `PostService`)
- `infrastructure/` — config, database pool, migrations, logging, security (Argon2 + JWT)
- `presentation/` — HTTP handlers, DTOs, middleware, gRPC services, generated proto code
- `blog_client/` — Rust client crate (HTTP + gRPC)
- `blog_wasm/` — WASM client exposed to JS (`BlogApp`)
- `blog-cli/` — CLI tool using `blog_client`

---

## Requirements

- Rust toolchain (stable)
- PostgreSQL
- `sqlx` migrations enabled in the server crate
- (Optional) `wasm-pack` if you build the WASM package yourself

---

## Configuration

The server loads configuration from environment variables (supports `.env`):

| Variable       | Description             | Example                                              |
|----------------|-------------------------|------------------------------------------------------|
| `HOST`         | Bind host               | `127.0.0.1`                                          |
| `HTTP_PORT`    | HTTP port               | `8080`                                               |
| `GRPC_PORT`    | gRPC port               | `50051` (or your chosen port)                        |
| `DATABASE_URL` | Postgres URL            | `postgres://user:pass@localhost:5432/blog`           |
| `JWT_SECRET`   | Secret for JWT signing  | `super-secret`                                       |
| `CORS_ORIGINS` | Comma-separated origins | `http://localhost:5173,http://localhost:8080` or `*` |
| `LOG_FORMAT`   | `text` or `json`        | `text`                                               |

# Blog HTTP API – Endpoints

## Health
- `GET /health`

## Auth (public)
- `POST /api/public/auth/register`
- `POST /api/public/auth/login`

## Posts (protected, JWT required)
- `GET /api/protected/posts`
- `GET /api/protected/posts/{id}`
- `POST /api/protected/posts`
- `PUT /api/protected/posts/{id}`
- `DELETE /api/protected/posts/{id}`

## Authentication
- Header: `Authorization: Bearer <access_token>`
