# Personalization Engine

AI-powered personalization engine for Easy AI Platform. Provides dynamic content personalization, lead scoring, and recommendations.

## Architecture

```
Rybbit (ClickHouse) ─┐
                     ├──▶ Data Ingestion ──▶ Gorse (Recommendations)
CDP (MySQL) ─────────┘          │                     │
                                └──────────┬──────────┘
                                           ▼
                               ┌───────────────────────┐
                               │  Orchestration Layer  │
                               │  - Lead Scoring       │
                               │  - Content Rules      │
                               │  - Channel Router     │
                               └───────────────────────┘
```

## Quick Start

```bash
# 1. Start infrastructure
make docker-up

# 2. Copy env file
cp .env.example .env

# 3. Run development server
make dev
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/personalize/:user_id` | Full personalization |
| GET | `/api/v1/recommend/:user_id` | Recommendations only |
| GET | `/api/v1/lead-score/:user_id` | Get lead score |
| POST | `/api/v1/lead-score/:user_id` | Recalculate score |
| GET | `/api/v1/admin/rules` | List content rules |
| POST | `/api/v1/admin/rules` | Create rule |
| POST | `/webhook/rybbit` | Rybbit events |
| POST | `/webhook/cdp` | CDP events |

## Tech Stack

- **Rust + Axum** - High-performance API server
- **Gorse** - ML recommendation engine
- **ClickHouse** - Analytics data (Rybbit)
- **MySQL** - CDP customer data
- **Redis** - Caching layer

## Development

```bash
make fmt      # Format + lint
make test     # Run tests
make build    # Release build
```

## Configuration

Environment variables (prefix `PE__`):

```bash
PE__SERVER__PORT=8080
PE__DATABASE__CLICKHOUSE_URL=http://localhost:8123
PE__DATABASE__MYSQL_URL=mysql://...
PE__REDIS__URL=redis://localhost:6379
PE__GORSE__URL=http://localhost:8087
PE__GORSE__API_KEY=your-key
```
