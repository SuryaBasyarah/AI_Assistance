use crate::{
    database::repositories::{ai_knowledge_repository, memory_repository},
    error::{AppError, AppResult},
    models::{RagContext, RankedAiKnowledge, RankedMemory},
    services::{AppState, openai_response_service},
};
use chrono::{DateTime, Utc};

fn cosine(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut aa, mut bb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        dot += *x as f64 * *y as f64;
        aa += (*x as f64).powi(2);
        bb += (*y as f64).powi(2);
    }
    if aa == 0.0 || bb == 0.0 {
        0.0
    } else {
        dot / (aa.sqrt() * bb.sqrt())
    }
}

fn lexical(query: &str, content: &str) -> f64 {
    let query = query.to_lowercase();
    let content = content.to_lowercase();
    let terms = query
        .split(|character: char| !character.is_alphanumeric())
        .filter(|term| term.len() > 2)
        .collect::<Vec<_>>();
    if terms.is_empty() {
        0.0
    } else {
        terms.iter().filter(|term| content.contains(**term)).count() as f64 / terms.len() as f64
    }
}

pub async fn retrieve(
    state: &AppState,
    query: &str,
    user_limit: usize,
    ai_limit: usize,
) -> AppResult<RagContext> {
    let query_vector = openai_response_service::embedding(state, query).await?;
    let (user_rows, ai_rows) = {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        (
            memory_repository::active(&conn)?,
            ai_knowledge_repository::active(&conn)?,
        )
    };

    let mut user_memories = Vec::new();
    for (memory, stored) in user_rows {
        let lexical_score = lexical(query, &memory.content);
        let semantic = stored
            .as_ref()
            .map(|vector| cosine(&query_vector, vector))
            .unwrap_or(lexical_score);
        let age_days = DateTime::parse_from_rfc3339(&memory.updated_at)
            .ok()
            .map(|date| (Utc::now() - date.with_timezone(&Utc)).num_days().max(0) as f64)
            .unwrap_or(365.0);
        let recency = 1.0 / (1.0 + age_days / 30.0);
        let frequency = (memory.access_count as f64 + 1.0).ln() / 5.0;
        let score = semantic * 0.60
            + memory.importance * 0.15
            + recency * 0.10
            + frequency.min(1.0) * 0.05
            + if lexical_score > 0.0 { 0.10 } else { 0.0 };
        if score >= 0.28 {
            user_memories.push(RankedMemory { memory, score });
        }
    }
    user_memories.sort_by(|left, right| right.score.total_cmp(&left.score));
    user_memories.truncate(user_limit);

    let mut ai_knowledge = Vec::new();
    for (knowledge, stored) in ai_rows {
        let searchable = format!(
            "{} {} {} {}",
            knowledge.category, knowledge.title, knowledge.content, knowledge.source_kind
        );
        let lexical_score = lexical(query, &searchable);
        let semantic = stored
            .as_ref()
            .map(|vector| cosine(&query_vector, vector))
            .unwrap_or(lexical_score);
        let frequency = (knowledge.access_count as f64 + 1.0).ln() / 5.0;
        let score = semantic * 0.72
            + knowledge.importance * 0.18
            + frequency.min(1.0) * 0.04
            + if lexical_score > 0.0 { 0.06 } else { 0.0 };
        if score >= 0.25 {
            ai_knowledge.push(RankedAiKnowledge { knowledge, score });
        }
    }
    ai_knowledge.sort_by(|left, right| right.score.total_cmp(&left.score));
    ai_knowledge.truncate(ai_limit);

    {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        let user_ids = user_memories
            .iter()
            .map(|item| item.memory.id)
            .collect::<Vec<_>>();
        memory_repository::touch(&conn, &user_ids)?;
        ai_knowledge_repository::touch(&conn, &ai_knowledge)?;
    }

    Ok(RagContext {
        user_memories,
        ai_knowledge,
    })
}

pub async fn backfill_embeddings(state: &AppState) -> AppResult<usize> {
    let (user_rows, ai_rows) = {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        (
            memory_repository::active(&conn)?,
            ai_knowledge_repository::active(&conn)?,
        )
    };

    let missing_users = user_rows
        .into_iter()
        .filter(|(_, embedding)| embedding.is_none())
        .map(|(memory, _)| memory)
        .collect::<Vec<_>>();
    let missing_ai = ai_rows
        .into_iter()
        .filter(|(_, embedding)| embedding.is_none())
        .map(|(knowledge, _)| knowledge)
        .collect::<Vec<_>>();
    let inputs = missing_users
        .iter()
        .map(|memory| memory.content.clone())
        .chain(missing_ai.iter().map(|knowledge| {
            format!(
                "{}: {}. {}",
                knowledge.category, knowledge.title, knowledge.content
            )
        }))
        .collect::<Vec<_>>();
    if inputs.is_empty() {
        return Ok(0);
    }

    let vectors = openai_response_service::embedding_batch(state, &inputs).await?;
    if vectors.len() != inputs.len() {
        return Err(AppError::from(
            "Embedding batch returned an unexpected size",
        ));
    }
    let conn = state
        .db
        .0
        .lock()
        .map_err(|_| AppError::from("Database lock poisoned"))?;
    let mut vectors = vectors.into_iter();
    for memory in &missing_users {
        memory_repository::set_embedding(
            &conn,
            memory.id,
            &vectors.next().ok_or("Missing user-memory embedding")?,
        )?;
    }
    for knowledge in &missing_ai {
        ai_knowledge_repository::set_embedding(
            &conn,
            knowledge.id,
            &vectors.next().ok_or("Missing AI-knowledge embedding")?,
        )?;
    }
    Ok(inputs.len())
}
