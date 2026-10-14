use crate::{error::AppResult, models::ChatTurn};
use chrono::Utc;
use rusqlite::{Connection, params};

pub fn add(
    conn: &Connection,
    role: &str,
    content_id: &str,
    content_ja: Option<&str>,
) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO conversations(role,content_id,content_ja,created_at) VALUES (?1,?2,?3,?4)",
        params![role, content_id, content_ja, Utc::now().to_rfc3339()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn recent(conn: &Connection, limit: i64) -> AppResult<Vec<ChatTurn>> {
    let mut stmt = conn.prepare(
        "SELECT id,role,content_id,content_ja,created_at FROM conversations ORDER BY id DESC LIMIT ?1"
    )?;
    let mut rows: Vec<_> = stmt
        .query_map([limit], |r| {
            Ok(ChatTurn {
                id: r.get(0)?,
                role: r.get(1)?,
                content_id: r.get(2)?,
                content_ja: r.get(3)?,
                created_at: r.get(4)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    rows.reverse();
    Ok(rows)
}
