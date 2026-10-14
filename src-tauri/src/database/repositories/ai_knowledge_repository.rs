use crate::{
    error::AppResult,
    models::{AiKnowledge, RankedAiKnowledge},
};
use chrono::Utc;
use rusqlite::{Connection, params};

pub fn active(conn: &Connection) -> AppResult<Vec<(AiKnowledge, Option<Vec<f32>>)>> {
    let mut stmt = conn.prepare(
        "SELECT id,slug,category,title,content,source_kind,source_reference,importance,
                access_count,created_at,updated_at,embedding_json
         FROM ai_knowledge WHERE active=1",
    )?;
    Ok(stmt
        .query_map([], |row| {
            let raw: Option<String> = row.get(11)?;
            let embedding = raw.and_then(|value| serde_json::from_str(&value).ok());
            Ok((
                AiKnowledge {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    category: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    source_kind: row.get(5)?,
                    source_reference: row.get(6)?,
                    importance: row.get(7)?,
                    access_count: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                },
                embedding,
            ))
        })?
        .collect::<Result<_, _>>()?)
}

pub fn set_embedding(conn: &Connection, id: i64, embedding: &[f32]) -> AppResult<()> {
    conn.execute(
        "UPDATE ai_knowledge SET embedding_json=?2,updated_at=?3 WHERE id=?1",
        params![
            id,
            serde_json::to_string(embedding)?,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn touch(conn: &Connection, knowledge: &[RankedAiKnowledge]) -> AppResult<()> {
    for item in knowledge {
        conn.execute(
            "UPDATE ai_knowledge SET access_count=access_count+1 WHERE id=?1",
            [item.knowledge.id],
        )?;
    }
    Ok(())
}
