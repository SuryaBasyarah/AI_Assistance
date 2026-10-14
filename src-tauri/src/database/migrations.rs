use crate::error::AppResult;
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

struct Migration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "001_core",
        sql: include_str!("../../migrations/001_core.sql"),
    },
    Migration {
        version: "009_multi_rvc_profiles",
        sql: include_str!("../../migrations/009_multi_rvc_profiles.sql"),
    },
    Migration {
        version: "010_ai_knowledge",
        sql: include_str!("../../migrations/010_ai_knowledge.sql"),
    },
    Migration {
        version: "011_seed_user_memory",
        sql: include_str!("../../migrations/011_seed_user_memory.sql"),
    },
    Migration {
        version: "012_seed_mizuki_knowledge",
        sql: include_str!("../../migrations/012_seed_mizuki_knowledge.sql"),
    },
];

pub fn run(conn: &mut Connection) -> AppResult<()> {
    conn.execute_batch(include_str!("../../migrations/000_schema_migrations.sql"))?;

    for migration in MIGRATIONS {
        let applied: Option<String> = conn
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                [migration.version],
                |row| row.get(0),
            )
            .optional()?;
        if applied.is_some() {
            continue;
        }

        let transaction = conn.transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
            params![migration.version, Utc::now().to_rfc3339()],
        )?;
        transaction.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_separate_user_and_ai_rag_stores_idempotently() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run(&mut connection).expect("apply migrations");
        run(&mut connection).expect("reapply migrations");

        let user_memories: i64 = connection
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
            .expect("count user memories");
        let ai_knowledge: i64 = connection
            .query_row("SELECT COUNT(*) FROM ai_knowledge", [], |row| row.get(0))
            .expect("count AI knowledge");
        let migrations: i64 = connection
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("count migrations");

        assert_eq!(user_memories, 1);
        assert!(ai_knowledge >= 30);
        assert_eq!(migrations, MIGRATIONS.len() as i64);
    }
}
