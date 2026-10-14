use crate::{error::AppResult, models::Memory};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

pub fn active(conn: &Connection) -> AppResult<Vec<(Memory, Option<Vec<f32>>)>> {
    let mut stmt=conn.prepare("SELECT id,content,importance,access_count,created_at,updated_at,embedding_json FROM memories WHERE active=1")?;
    Ok(stmt
        .query_map([], |r| {
            let raw: Option<String> = r.get(6)?;
            let embedding = raw.and_then(|v| serde_json::from_str(&v).ok());
            Ok((
                Memory {
                    id: r.get(0)?,
                    content: r.get(1)?,
                    importance: r.get(2)?,
                    access_count: r.get(3)?,
                    created_at: r.get(4)?,
                    updated_at: r.get(5)?,
                },
                embedding,
            ))
        })?
        .collect::<Result<_, _>>()?)
}

pub fn add_if_new(conn: &Connection, content: &str, embedding: Option<&[f32]>) -> AppResult<i64> {
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM memories WHERE active=1 AND lower(content)=lower(?1)",
            [content],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let now = Utc::now().to_rfc3339();
    let json = embedding.map(serde_json::to_string).transpose()?;
    conn.execute("INSERT INTO memories(content,embedding_json,importance,active,created_at,updated_at) VALUES (?1,?2,0.7,1,?3,?3)",params![content,json,now])?;
    Ok(conn.last_insert_rowid())
}

pub fn set_embedding(conn: &Connection, id: i64, embedding: &[f32]) -> AppResult<()> {
    conn.execute(
        "UPDATE memories SET embedding_json=?2,updated_at=?3 WHERE id=?1",
        params![
            id,
            serde_json::to_string(embedding)?,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn touch(conn: &Connection, ids: &[i64]) -> AppResult<()> {
    for id in ids {
        conn.execute(
            "UPDATE memories SET access_count=access_count+1 WHERE id=?1",
            [id],
        )?;
    }
    Ok(())
}
