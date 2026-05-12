//! Persistance des événements dans PostgreSQL — pattern outbox.
//!
//! Toutes les écritures passent par cette couche. Les événements sont
//! stockés dans la table `events` avec un `sequence BIGSERIAL` qui définit
//! un ordre total. Les subscribers consomment via un watermark par
//! identifiant (`event_subscriptions.last_sequence`), ce qui garantit
//! une livraison **at-least-once** : un événement n'est marqué comme
//! consommé que lorsque le subscriber confirme l'avoir traité.
//!
//! Les échecs sont tracés dans `event_deliveries` pour l'observabilité,
//! sans bloquer la table principale.

use chrono::{DateTime, Utc};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::Row;
use uuid::Uuid;

use crate::error::EventResult;
use crate::event::{Event, EventKind};

/// Schéma SQL du bus d'événements. Idempotent (CREATE IF NOT EXISTS).
pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    id              UUID PRIMARY KEY,
    sequence        BIGSERIAL UNIQUE NOT NULL,
    flow_id         UUID NOT NULL,
    kind            TEXT NOT NULL,
    invoice_key     TEXT,
    tenant_siren    TEXT,
    route_id        TEXT,
    step            TEXT,
    message         TEXT,
    error_detail    TEXT,
    payload         JSONB,
    occurred_at     TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_events_flow      ON events(flow_id);
CREATE INDEX IF NOT EXISTS idx_events_invoice   ON events(invoice_key);
CREATE INDEX IF NOT EXISTS idx_events_tenant    ON events(tenant_siren);
CREATE INDEX IF NOT EXISTS idx_events_kind      ON events(kind);
CREATE INDEX IF NOT EXISTS idx_events_occurred  ON events(occurred_at);
CREATE INDEX IF NOT EXISTS idx_events_seq       ON events(sequence);

CREATE TABLE IF NOT EXISTS event_subscriptions (
    subscriber_id   TEXT PRIMARY KEY,
    last_sequence   BIGINT NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS event_deliveries (
    subscriber_id   TEXT NOT NULL,
    event_id        UUID NOT NULL,
    attempts        INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    delivered_at    TIMESTAMPTZ,
    last_error      TEXT,
    PRIMARY KEY (subscriber_id, event_id)
);

CREATE INDEX IF NOT EXISTS idx_deliveries_pending
    ON event_deliveries(subscriber_id, delivered_at)
    WHERE delivered_at IS NULL;
"#;

/// Store d'événements adossé à PostgreSQL.
#[derive(Clone)]
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Initialise le schéma. Idempotent.
    pub async fn migrate(&self) -> EventResult<()> {
        for stmt in SCHEMA_SQL.split(';') {
            let s = stmt.trim();
            if s.is_empty() {
                continue;
            }
            sqlx::query(s).execute(&self.pool).await?;
        }
        tracing::info!("Schéma pdp-events initialisé");
        Ok(())
    }

    /// Insère un événement et renvoie l'événement complet avec son `sequence`.
    pub async fn append(&self, event: &Event) -> EventResult<Event> {
        let row = sqlx::query(
            r#"INSERT INTO events
               (id, flow_id, kind, invoice_key, tenant_siren, route_id, step,
                message, error_detail, payload, occurred_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
               RETURNING sequence"#,
        )
        .bind(event.id)
        .bind(event.flow_id)
        .bind(event.kind.as_code())
        .bind(event.invoice_key.as_deref())
        .bind(event.tenant_siren.as_deref())
        .bind(event.route_id.as_deref())
        .bind(event.step.as_deref())
        .bind(event.message.as_deref())
        .bind(event.error_detail.as_deref())
        .bind(event.payload.as_ref())
        .bind(event.occurred_at)
        .fetch_one(&self.pool)
        .await?;

        let sequence: i64 = row.try_get("sequence")?;
        let mut out = event.clone();
        out.sequence = Some(sequence);
        Ok(out)
    }

    /// Récupère un événement par son `id`.
    pub async fn get(&self, id: Uuid) -> EventResult<Option<Event>> {
        let row = sqlx::query(SELECT_COLUMNS_SQL)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(row_to_event))
    }

    /// Renvoie tous les événements d'un flux, triés par `sequence`.
    pub async fn list_by_flow(&self, flow_id: Uuid) -> EventResult<Vec<Event>> {
        let rows = sqlx::query(LIST_BY_FLOW_SQL)
            .bind(flow_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    /// Renvoie tous les événements d'une facture (clef métier).
    pub async fn list_by_invoice(&self, invoice_key: &str) -> EventResult<Vec<Event>> {
        let rows = sqlx::query(LIST_BY_INVOICE_SQL)
            .bind(invoice_key)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    /// Récupère les événements postérieurs à un `sequence` donné, limités à `max`.
    pub async fn fetch_after(&self, after: i64, max: i64) -> EventResult<Vec<Event>> {
        let rows = sqlx::query(FETCH_AFTER_SQL)
            .bind(after)
            .bind(max)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(row_to_event).collect())
    }

    /// Watermark actuel d'un subscriber (0 si inconnu).
    pub async fn get_watermark(&self, subscriber_id: &str) -> EventResult<i64> {
        let row = sqlx::query(
            "SELECT last_sequence FROM event_subscriptions WHERE subscriber_id = $1",
        )
        .bind(subscriber_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.and_then(|r| r.try_get("last_sequence").ok()).unwrap_or(0))
    }

    /// Avance le watermark du subscriber (ne descend jamais).
    pub async fn set_watermark(&self, subscriber_id: &str, sequence: i64) -> EventResult<()> {
        sqlx::query(
            r#"INSERT INTO event_subscriptions (subscriber_id, last_sequence, updated_at)
               VALUES ($1, $2, NOW())
               ON CONFLICT (subscriber_id) DO UPDATE
               SET last_sequence = GREATEST(event_subscriptions.last_sequence, EXCLUDED.last_sequence),
                   updated_at = NOW()"#,
        )
        .bind(subscriber_id)
        .bind(sequence)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Enregistre une tentative de livraison (succès ou échec).
    pub async fn record_delivery(
        &self,
        subscriber_id: &str,
        event_id: Uuid,
        success: bool,
        error: Option<&str>,
    ) -> EventResult<()> {
        let delivered_at: Option<DateTime<Utc>> = if success { Some(Utc::now()) } else { None };
        sqlx::query(
            r#"INSERT INTO event_deliveries
                 (subscriber_id, event_id, attempts, last_attempt_at, delivered_at, last_error)
               VALUES ($1, $2, 1, NOW(), $3, $4)
               ON CONFLICT (subscriber_id, event_id) DO UPDATE
               SET attempts = event_deliveries.attempts + 1,
                   last_attempt_at = NOW(),
                   delivered_at = COALESCE(event_deliveries.delivered_at, EXCLUDED.delivered_at),
                   last_error = EXCLUDED.last_error"#,
        )
        .bind(subscriber_id)
        .bind(event_id)
        .bind(delivered_at)
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Nombre total d'événements (debug/observabilité).
    pub async fn count(&self) -> EventResult<i64> {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;
        Ok(n)
    }
}

const SELECT_COLUMNS: &str = "id, sequence, flow_id, kind, invoice_key, tenant_siren, \
     route_id, step, message, error_detail, payload, occurred_at";

const SELECT_COLUMNS_SQL: &str = "SELECT id, sequence, flow_id, kind, invoice_key, \
    tenant_siren, route_id, step, message, error_detail, payload, occurred_at \
    FROM events WHERE id = $1";

const LIST_BY_FLOW_SQL: &str = "SELECT id, sequence, flow_id, kind, invoice_key, \
    tenant_siren, route_id, step, message, error_detail, payload, occurred_at \
    FROM events WHERE flow_id = $1 ORDER BY sequence";

const LIST_BY_INVOICE_SQL: &str = "SELECT id, sequence, flow_id, kind, invoice_key, \
    tenant_siren, route_id, step, message, error_detail, payload, occurred_at \
    FROM events WHERE invoice_key = $1 ORDER BY sequence";

const FETCH_AFTER_SQL: &str = "SELECT id, sequence, flow_id, kind, invoice_key, \
    tenant_siren, route_id, step, message, error_detail, payload, occurred_at \
    FROM events WHERE sequence > $1 ORDER BY sequence LIMIT $2";

fn row_to_event(row: &PgRow) -> Event {
    let kind_code: String = row.try_get("kind").unwrap_or_default();
    let kind = EventKind::from_code(&kind_code).unwrap_or(EventKind::Error);
    Event {
        id: row.try_get("id").unwrap_or_else(|_| Uuid::nil()),
        flow_id: row.try_get("flow_id").unwrap_or_else(|_| Uuid::nil()),
        kind,
        invoice_key: row.try_get::<Option<String>, _>("invoice_key").ok().flatten(),
        tenant_siren: row.try_get::<Option<String>, _>("tenant_siren").ok().flatten(),
        route_id: row.try_get::<Option<String>, _>("route_id").ok().flatten(),
        step: row.try_get::<Option<String>, _>("step").ok().flatten(),
        message: row.try_get::<Option<String>, _>("message").ok().flatten(),
        error_detail: row.try_get::<Option<String>, _>("error_detail").ok().flatten(),
        payload: row.try_get::<Option<serde_json::Value>, _>("payload").ok().flatten(),
        occurred_at: row.try_get("occurred_at").unwrap_or_else(|_| Utc::now()),
        sequence: row.try_get("sequence").ok(),
    }
}

#[allow(dead_code)]
const _COLUMNS_DOC: &str = SELECT_COLUMNS;
