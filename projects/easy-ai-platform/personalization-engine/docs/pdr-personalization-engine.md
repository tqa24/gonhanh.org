# Personalization Engine - Product Development Requirements

## Overview

**Project:** Easy AI Personalization Engine
**Purpose:** Analyze user behavior data (from Rybbit/ClickHouse) + customer data (from CDP/MySQL) to generate personalized content recommendations and lead scores across multiple channels.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PERSONALIZATION ENGINE                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐    ┌──────────────────┐    ┌──────────────────────────┐  │
│  │   Rybbit     │───▶│  Data Ingestion  │───▶│   Gorse Recommendation   │  │
│  │ (ClickHouse) │    │  Service (Rust)  │    │   Engine (Go)            │  │
│  └──────────────┘    └────────┬─────────┘    └───────────┬──────────────┘  │
│                               │                          │                  │
│  ┌──────────────┐             │                          │                  │
│  │     CDP      │─────────────┤                          │                  │
│  │   (MySQL)    │             │                          │                  │
│  └──────────────┘             ▼                          ▼                  │
│                      ┌────────────────────────────────────────────┐        │
│                      │         Orchestration Layer (Rust)         │        │
│                      │  - Lead Scoring Engine                     │        │
│                      │  - Content Rules Engine                    │        │
│                      │  - Channel Router                          │        │
│                      │  - A/B Test Manager                        │        │
│                      └────────────────────┬───────────────────────┘        │
│                                           │                                 │
└───────────────────────────────────────────┼─────────────────────────────────┘
                                            ▼
                    ┌───────────────────────────────────────────┐
                    │              Output Channels               │
                    ├───────────┬───────────┬───────────┬───────┤
                    │  Website  │   Email   │    SMS    │  Push │
                    │  (API)    │ (Webhook) │ (Webhook) │ (FCM) │
                    └───────────┴───────────┴───────────┴───────┘
```

## Components

### 1. Data Ingestion Service (Rust)
- Connect to ClickHouse (Rybbit analytics data)
- Connect to MySQL (CDP customer data)
- Sync user profiles + events to Gorse
- Real-time event streaming via webhooks

### 2. Gorse Recommendation Engine (Go - Open Source)
- Item-based collaborative filtering
- User-based recommendations
- Popular/trending items
- Similar items
- REST API for recommendations

### 3. Orchestration Layer (Rust)
- **Lead Scoring Engine:** Calculate lead scores based on behavior + demographics
- **Content Rules Engine:** Define rules for dynamic content (if segment X, show content Y)
- **Channel Router:** Route personalized content to appropriate channels
- **A/B Test Manager:** Manage experiments and variant allocation

## Data Flow

```
1. User visits website → Rybbit tracks behavior → ClickHouse
2. User submits form → CDP captures lead → MySQL
3. Data Ingestion syncs to Gorse (items, users, feedback)
4. Orchestration Layer:
   a. Fetch recommendations from Gorse
   b. Apply lead scoring rules
   c. Apply content personalization rules
   d. Route to output channels
5. Channel delivers personalized experience
```

## Tech Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Data Ingestion | Rust + Axum | High performance, type safety |
| Orchestration | Rust + Axum | Consistent with ingestion |
| Recommendations | Gorse (Go) | Mature ML, supports ClickHouse/MySQL |
| Cache | Redis | Session data, hot recommendations |
| Config | YAML + env | 12-factor app compliance |
| API | REST + GraphQL | REST for internal, GraphQL for frontend |

## Database Schema

### Lead Scores Table (MySQL)
```sql
CREATE TABLE lead_scores (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id VARCHAR(255) NOT NULL,
    score DECIMAL(5,2) NOT NULL,
    segment VARCHAR(100),
    factors JSON,
    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_user_id (user_id),
    INDEX idx_segment (segment)
);
```

### Content Rules Table (MySQL)
```sql
CREATE TABLE content_rules (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    conditions JSON NOT NULL,
    actions JSON NOT NULL,
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### A/B Experiments Table (MySQL)
```sql
CREATE TABLE ab_experiments (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    variants JSON NOT NULL,
    traffic_allocation JSON NOT NULL,
    status ENUM('draft', 'running', 'paused', 'completed') DEFAULT 'draft',
    start_at TIMESTAMP NULL,
    end_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## API Endpoints

### Personalization API
```
GET  /api/v1/personalize/:user_id          # Get personalized content for user
GET  /api/v1/recommend/:user_id            # Get item recommendations
POST /api/v1/lead-score/:user_id           # Calculate/update lead score
GET  /api/v1/segment/:user_id              # Get user segment
```

### Admin API
```
GET    /api/v1/admin/rules                 # List content rules
POST   /api/v1/admin/rules                 # Create rule
PUT    /api/v1/admin/rules/:id             # Update rule
DELETE /api/v1/admin/rules/:id             # Delete rule

GET    /api/v1/admin/experiments           # List A/B experiments
POST   /api/v1/admin/experiments           # Create experiment
PUT    /api/v1/admin/experiments/:id       # Update experiment
```

### Webhook Endpoints
```
POST /webhook/rybbit                       # Receive Rybbit events
POST /webhook/cdp                          # Receive CDP events
```

## Lead Scoring Model

```
Score = Σ (Factor × Weight)

Factors:
- page_views_last_7d      × 0.5
- time_on_site_avg        × 1.0
- form_submissions        × 5.0
- pricing_page_visits     × 3.0
- demo_request            × 10.0
- email_opens             × 0.5
- email_clicks            × 1.0
- return_visits           × 2.0

Segments:
- Cold Lead:    score < 20
- Warm Lead:    20 <= score < 50
- Hot Lead:     50 <= score < 80
- Sales Ready:  score >= 80
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Setup Rust project with Axum
- [ ] Deploy Gorse with Docker
- [ ] ClickHouse connector
- [ ] MySQL connector
- [ ] Basic data sync to Gorse

### Phase 2: Core Engine (Week 3-4)
- [ ] Lead Scoring Engine
- [ ] Content Rules Engine
- [ ] Personalization API
- [ ] Redis caching layer

### Phase 3: Channels & Admin (Week 5-6)
- [ ] Webhook integrations (email, SMS)
- [ ] Admin API for rules management
- [ ] A/B Test Manager
- [ ] Dashboard UI (optional)

### Phase 4: Polish & Scale (Week 7-8)
- [ ] Performance optimization
- [ ] Monitoring & alerting
- [ ] Documentation
- [ ] Load testing

## Environment Variables

```bash
# Database
CLICKHOUSE_URL=clickhouse://localhost:9000/rybbit
MYSQL_URL=mysql://user:pass@localhost:3306/cdp
REDIS_URL=redis://localhost:6379

# Gorse
GORSE_URL=http://localhost:8087
GORSE_API_KEY=your-api-key

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
LOG_LEVEL=info

# Channels
EMAIL_WEBHOOK_URL=https://email-service/webhook
SMS_WEBHOOK_URL=https://sms-service/webhook
```

## Success Metrics

1. **Recommendation Quality:** CTR on recommended items > 5%
2. **Lead Scoring Accuracy:** 70% of "Sales Ready" leads convert
3. **Latency:** P99 < 100ms for personalization API
4. **Throughput:** Handle 1000 RPS per node

## Open Questions

1. Gorse hay build custom recommendation từ đầu?
2. Real-time scoring hay batch processing?
3. Admin UI riêng hay integrate vào existing dashboard?
