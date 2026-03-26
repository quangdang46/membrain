use crate::api::NamespaceId;
use crate::engine::confidence::{ConfidenceInputs, ConfidenceOutput};
use crate::engine::lease::LeaseMetadata;
use crate::graph::CausalLinkType;
use crate::store::audit::AuditLogEntry;
use crate::store::tier2::{
    Tier2DurableItemLayout, Tier2MetadataRecord, Tier2PayloadLocator, Tier2PayloadRecord,
};
use crate::types::{
    AffectSignals, CanonicalMemoryType, CompressionMetadata, FastPathRouteFamily, LandmarkMetadata,
    MemoryId, SessionId,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const APP_DIR: &str = ".membrain";
const HOT_DB: &str = "hot.db";
const COLD_DB: &str = "cold.db";
const SOCKET_FILE: &str = "membrain.sock";
const HOT_SCHEMA_VERSION: i64 = 1;
const COLD_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalPaths {
    pub root_dir: PathBuf,
    pub hot_db_path: PathBuf,
    pub cold_db_path: PathBuf,
    pub socket_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedLocalMemoryRecord {
    pub memory_id: u64,
    pub namespace: String,
    pub session_id: u64,
    pub memory_type: CanonicalMemoryType,
    pub route_family: FastPathRouteFamily,
    pub compact_text: String,
    pub provisional_salience: u16,
    pub affect: Option<AffectSignals>,
    pub fingerprint: u64,
    pub payload_size_bytes: usize,
    pub is_landmark: bool,
    pub landmark_label: Option<String>,
    pub era_id: Option<String>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDaemonMemoryRecord {
    pub layout: PersistedTier2Layout,
    pub passive_observation: Option<PersistedPassiveObservationSummary>,
    pub causal_parents: Vec<u64>,
    pub causal_link_type: Option<CausalLinkType>,
    pub confidence_inputs: ConfidenceInputs,
    pub confidence_output: ConfidenceOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPassiveObservationSummary {
    pub source_kind: String,
    pub write_decision: String,
    pub captured_as_observation: bool,
    pub observation_source: Option<String>,
    pub observation_chunk_id: Option<String>,
    pub retention_marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTier2Layout {
    pub namespace: String,
    pub memory_id: u64,
    pub session_id: u64,
    pub memory_type: CanonicalMemoryType,
    pub route_family: FastPathRouteFamily,
    pub compact_text: String,
    pub fingerprint: u64,
    pub payload_size_bytes: usize,
    pub affect: Option<AffectSignals>,
    pub is_landmark: bool,
    pub landmark_label: Option<String>,
    pub era_id: Option<String>,
    pub visibility: String,
    pub raw_text: String,
}

pub fn default_local_paths() -> std::io::Result<LocalPaths> {
    let home = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found")
    })?;
    let root_dir = home.join(APP_DIR);
    Ok(LocalPaths {
        hot_db_path: root_dir.join(HOT_DB),
        cold_db_path: root_dir.join(COLD_DB),
        socket_path: root_dir.join(SOCKET_FILE),
        root_dir,
    })
}

pub fn resolve_local_paths(db_path: Option<&Path>) -> std::io::Result<LocalPaths> {
    let defaults = default_local_paths()?;
    if let Some(path) = db_path {
        if path.extension().is_some_and(|ext| ext == "db") {
            let root_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let hot_db_path = path.to_path_buf();
            let cold_db_path = root_dir.join(COLD_DB);
            let socket_path = root_dir.join(SOCKET_FILE);
            return Ok(LocalPaths {
                root_dir,
                hot_db_path,
                cold_db_path,
                socket_path,
            });
        }
        let root_dir = path.to_path_buf();
        return Ok(LocalPaths {
            hot_db_path: root_dir.join(HOT_DB),
            cold_db_path: root_dir.join(COLD_DB),
            socket_path: root_dir.join(SOCKET_FILE),
            root_dir,
        });
    }
    Ok(defaults)
}

pub fn ensure_local_root(paths: &LocalPaths) -> std::io::Result<()> {
    fs::create_dir_all(&paths.root_dir)
}

pub fn open_hot_db(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    bootstrap_hot_db(&conn)?;
    Ok(conn)
}

pub fn open_cold_db(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    bootstrap_cold_db(&conn)?;
    Ok(conn)
}

fn bootstrap_hot_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS memories (
            namespace TEXT NOT NULL,
            memory_id INTEGER NOT NULL,
            session_id INTEGER NOT NULL,
            memory_type TEXT NOT NULL,
            route_family TEXT NOT NULL,
            compact_text TEXT NOT NULL,
            raw_text TEXT NOT NULL,
            provisional_salience INTEGER NOT NULL,
            fingerprint TEXT NOT NULL,
            payload_size_bytes INTEGER NOT NULL,
            affect_json TEXT,
            is_landmark INTEGER NOT NULL,
            landmark_label TEXT,
            era_id TEXT,
            PRIMARY KEY(namespace, memory_id)
        );
        CREATE TABLE IF NOT EXISTS memory_audit_log (
            sequence INTEGER PRIMARY KEY,
            category TEXT NOT NULL,
            kind TEXT NOT NULL,
            namespace TEXT NOT NULL,
            memory_id INTEGER,
            session_id INTEGER,
            actor_source TEXT NOT NULL,
            request_id TEXT,
            tick INTEGER,
            before_strength INTEGER,
            after_strength INTEGER,
            before_confidence INTEGER,
            after_confidence INTEGER,
            related_snapshot TEXT,
            related_run TEXT,
            redacted INTEGER NOT NULL,
            detail TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_memories_namespace_id ON memories(namespace, memory_id);
        CREATE INDEX IF NOT EXISTS idx_memories_namespace_text ON memories(namespace, compact_text);
        CREATE INDEX IF NOT EXISTS idx_audit_namespace_seq ON memory_audit_log(namespace, sequence DESC);
        CREATE INDEX IF NOT EXISTS idx_audit_memory ON memory_audit_log(memory_id, sequence DESC);
        CREATE TABLE IF NOT EXISTS emotional_timeline (
            namespace TEXT NOT NULL,
            memory_id INTEGER NOT NULL,
            tick_start INTEGER NOT NULL,
            tick_end INTEGER,
            avg_valence REAL NOT NULL,
            avg_arousal REAL NOT NULL,
            memory_count INTEGER NOT NULL,
            era_id TEXT
        );
        ",
    )?;
    set_schema_version(conn, "hot_schema_version", HOT_SCHEMA_VERSION)?;
    Ok(())
}

fn bootstrap_cold_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cold_memories (
            namespace TEXT NOT NULL,
            memory_id INTEGER NOT NULL,
            compact_text TEXT NOT NULL,
            raw_text TEXT NOT NULL,
            payload_size_bytes INTEGER NOT NULL,
            PRIMARY KEY(namespace, memory_id)
        );
        ",
    )?;
    set_schema_version(conn, "cold_schema_version", COLD_SCHEMA_VERSION)?;
    Ok(())
}

fn set_schema_version(conn: &Connection, key: &str, version: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO schema_meta(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, version.to_string()],
    )?;
    Ok(())
}

pub fn save_cli_records(
    conn: &mut Connection,
    records: &[PersistedLocalMemoryRecord],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM memories", [])?;
    for record in records {
        let affect_json = record
            .affect
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(to_sql_err)?;
        tx.execute(
            "INSERT INTO memories(namespace, memory_id, session_id, memory_type, route_family, compact_text, raw_text, provisional_salience, fingerprint, payload_size_bytes, affect_json, is_landmark, landmark_label, era_id)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                record.namespace,
                record.memory_id,
                record.session_id,
                record.memory_type.as_str(),
                record.route_family.as_str(),
                record.compact_text,
                record.raw_text,
                record.provisional_salience,
                record.fingerprint.to_string(),
                record.payload_size_bytes,
                affect_json,
                i64::from(record.is_landmark),
                record.landmark_label,
                record.era_id,
            ],
        )?;
    }
    tx.commit()
}

pub fn load_cli_records(conn: &Connection) -> rusqlite::Result<Vec<PersistedLocalMemoryRecord>> {
    let mut stmt = conn.prepare(
        "SELECT namespace, memory_id, session_id, memory_type, route_family, compact_text, raw_text, provisional_salience, fingerprint, payload_size_bytes, affect_json, is_landmark, landmark_label, era_id
         FROM memories ORDER BY memory_id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let affect_json: Option<String> = row.get(10)?;
        let affect = affect_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(to_sql_err)?;
        Ok(PersistedLocalMemoryRecord {
            namespace: row.get(0)?,
            memory_id: row.get(1)?,
            session_id: row.get(2)?,
            memory_type: parse_memory_type(&row.get::<_, String>(3)?)?,
            route_family: parse_route_family(&row.get::<_, String>(4)?)?,
            compact_text: row.get(5)?,
            raw_text: row.get(6)?,
            provisional_salience: row.get(7)?,
            fingerprint: parse_fingerprint_value(row.get_ref(8)?)?,
            payload_size_bytes: row.get(9)?,
            affect,
            is_landmark: row.get::<_, i64>(11)? != 0,
            landmark_label: row.get(12)?,
            era_id: row.get(13)?,
        })
    })?;
    rows.collect()
}

pub fn save_audit_entries(
    conn: &mut Connection,
    entries: &[AuditLogEntry],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM memory_audit_log", [])?;
    for entry in entries {
        tx.execute(
            "INSERT INTO memory_audit_log(sequence, category, kind, namespace, memory_id, session_id, actor_source, request_id, tick, before_strength, after_strength, before_confidence, after_confidence, related_snapshot, related_run, redacted, detail)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                entry.sequence,
                entry.kind.category().as_str(),
                entry.kind.as_str(),
                entry.namespace.as_str(),
                entry.memory_id.map(|id| id.0),
                entry.session_id.map(|id| id.0),
                entry.actor_source,
                entry.request_id,
                entry.tick,
                entry.before_strength,
                entry.after_strength,
                entry.before_confidence,
                entry.after_confidence,
                entry.related_snapshot,
                entry.related_run,
                i64::from(entry.redacted),
                entry.detail,
            ],
        )?;
    }
    tx.commit()
}

pub fn max_memory_id(conn: &Connection) -> rusqlite::Result<u64> {
    conn.query_row(
        "SELECT COALESCE(MAX(memory_id), 0) FROM memories",
        [],
        |row| row.get(0),
    )
}

pub fn save_affect_row(
    conn: &Connection,
    namespace: &NamespaceId,
    memory_id: MemoryId,
    tick_start: u64,
    affect: AffectSignals,
    era_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO emotional_timeline(namespace, memory_id, tick_start, tick_end, avg_valence, avg_arousal, memory_count, era_id)
         VALUES(?1, ?2, ?3, NULL, ?4, ?5, 1, ?6)",
        params![namespace.as_str(), memory_id.0, tick_start, affect.valence, affect.arousal, era_id],
    )?;
    Ok(())
}

pub fn save_runtime_records(
    conn: &mut Connection,
    records: &[PersistedDaemonMemoryRecord],
) -> rusqlite::Result<()> {
    let cli_records = records
        .iter()
        .map(|record| PersistedLocalMemoryRecord {
            memory_id: record.layout.memory_id,
            namespace: record.layout.namespace.clone(),
            session_id: record.layout.session_id,
            memory_type: record.layout.memory_type,
            route_family: record.layout.route_family,
            compact_text: record.layout.compact_text.clone(),
            provisional_salience: 0,
            affect: record.layout.affect,
            fingerprint: record.layout.fingerprint,
            payload_size_bytes: record.layout.payload_size_bytes,
            is_landmark: record.layout.is_landmark,
            landmark_label: record.layout.landmark_label.clone(),
            era_id: record.layout.era_id.clone(),
            raw_text: record.layout.raw_text.clone(),
        })
        .collect::<Vec<_>>();
    save_cli_records(conn, &cli_records)
}

pub fn load_runtime_records(
    conn: &Connection,
) -> rusqlite::Result<Vec<PersistedDaemonMemoryRecord>> {
    load_cli_records(conn).map(|rows| {
        rows.into_iter()
            .map(|row| PersistedDaemonMemoryRecord {
                layout: PersistedTier2Layout {
                    namespace: row.namespace,
                    memory_id: row.memory_id,
                    session_id: row.session_id,
                    memory_type: row.memory_type,
                    route_family: row.route_family,
                    compact_text: row.compact_text,
                    fingerprint: row.fingerprint,
                    payload_size_bytes: row.payload_size_bytes,
                    affect: row.affect,
                    is_landmark: row.is_landmark,
                    landmark_label: row.landmark_label,
                    era_id: row.era_id,
                    visibility: "private".to_string(),
                    raw_text: row.raw_text,
                },
                passive_observation: None,
                causal_parents: Vec::new(),
                causal_link_type: None,
                confidence_inputs: ConfidenceInputs {
                    corroboration_count: 0,
                    reconsolidation_count: 0,
                    ticks_since_last_access: 0,
                    age_ticks: 0,
                    resolution_state: crate::engine::contradiction::ResolutionState::None,
                    conflict_score: 0,
                    causal_parent_count: 0,
                    authoritativeness: 0,
                    recall_count: 0,
                },
                confidence_output: ConfidenceOutput {
                    uncertainty_score: 1000,
                    corroboration_uncertainty: 1000,
                    reconsolidation_uncertainty: 1000,
                    freshness_uncertainty: 1000,
                    contradiction_uncertainty: 0,
                    missing_evidence_uncertainty: 1000,
                    confidence: 0,
                    confidence_interval: None,
                },
            })
            .collect()
    })
}

pub fn to_tier2_layout(
    record: &PersistedTier2Layout,
) -> Result<Tier2DurableItemLayout, Box<dyn std::error::Error + Send + Sync>> {
    let namespace = NamespaceId::new(&record.namespace)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?;
    let memory_id = MemoryId(record.memory_id);
    let session_id = SessionId(record.session_id);
    let payload_locator = Tier2PayloadLocator::for_memory(&namespace, memory_id);
    Ok(Tier2DurableItemLayout {
        metadata: Tier2MetadataRecord {
            namespace: namespace.clone(),
            memory_id,
            session_id,
            memory_type: record.memory_type,
            route_family: record.route_family,
            compact_text: record.compact_text.clone(),
            fingerprint: record.fingerprint,
            normalization_generation: "normalize-v1",
            payload_size_bytes: record.payload_size_bytes,
            affect: record.affect,
            landmark: LandmarkMetadata {
                is_landmark: record.is_landmark,
                landmark_label: record.landmark_label.clone(),
                era_id: record.era_id.clone(),
                era_started_at_tick: None,
                detection_score: 0,
                detection_reason: None,
            },
            visibility: crate::policy::SharingVisibility::parse(&record.visibility)
                .unwrap_or(crate::policy::SharingVisibility::Private),
            workspace_id: None,
            agent_id: None,
            observation_source: None,
            observation_chunk_id: None,
            lease: LeaseMetadata::recommended(record.memory_type, false),
            has_causal_parents: false,
            has_causal_children: false,
            compression: CompressionMetadata::default(),
            confidence_inputs: None,
            confidence_output: None,
            payload_locator: payload_locator.clone(),
        },
        payload: Tier2PayloadRecord {
            namespace,
            memory_id,
            payload_locator,
            raw_text: record.raw_text.clone(),
            raw_size_bytes: record.raw_text.len(),
        },
    })
}

fn to_sql_err<E>(err: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    rusqlite::Error::ToSqlConversionFailure(Box::new(err))
}

fn parse_memory_type(raw: &str) -> rusqlite::Result<CanonicalMemoryType> {
    match raw {
        "event" => Ok(CanonicalMemoryType::Event),
        "observation" => Ok(CanonicalMemoryType::Observation),
        "tool_outcome" => Ok(CanonicalMemoryType::ToolOutcome),
        "user_preference" => Ok(CanonicalMemoryType::UserPreference),
        "session_marker" => Ok(CanonicalMemoryType::SessionMarker),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid memory type: {raw}"),
            )),
        )),
    }
}

fn parse_route_family(raw: &str) -> rusqlite::Result<FastPathRouteFamily> {
    match raw {
        "event" => Ok(FastPathRouteFamily::Event),
        "observation" => Ok(FastPathRouteFamily::Observation),
        "tool_outcome" => Ok(FastPathRouteFamily::ToolOutcome),
        "user_preference" => Ok(FastPathRouteFamily::UserPreference),
        "session_marker" => Ok(FastPathRouteFamily::SessionMarker),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid route family: {raw}"),
            )),
        )),
    }
}

fn parse_fingerprint_value(value: rusqlite::types::ValueRef<'_>) -> rusqlite::Result<u64> {
    use rusqlite::types::ValueRef;
    match value {
        ValueRef::Text(bytes) => std::str::from_utf8(bytes)
            .map_err(to_sql_err)?
            .parse::<u64>()
            .map_err(to_sql_err),
        ValueRef::Integer(value) => u64::try_from(value).map_err(to_sql_err),
        ValueRef::Real(value) => {
            if value.is_finite() && value >= 0.0 {
                Ok(value as u64)
            } else {
                Err(rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Real,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid fingerprint real value: {value}"),
                    )),
                ))
            }
        }
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            8,
            other.data_type(),
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported fingerprint storage type",
            )),
        )),
    }
}

pub fn current_schema_version(conn: &Connection, key: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT value FROM schema_meta WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|value| value.and_then(|raw| raw.parse::<i64>().ok()))
}
