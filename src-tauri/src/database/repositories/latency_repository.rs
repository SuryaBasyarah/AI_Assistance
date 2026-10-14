use crate::{error::AppResult, models::LatencySample};
use rusqlite::{Connection, params};

pub fn add(conn: &Connection, s: &LatencySample) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO latency_samples(created_at,mode,rvc_profile_id,vad_to_stt_ms,rag_ms,llm_total_ms,tts_ms,rvc_ms,voice_switch_ms,post_speech_ttfa_ms,success,error_stage)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![s.created_at,s.mode,s.rvc_profile_id,s.vad_to_stt_ms,s.rag_ms,s.llm_total_ms,s.tts_ms,s.rvc_ms,s.voice_switch_ms,s.post_speech_ttfa_ms,s.success,s.error_stage],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn recent(conn: &Connection, limit: i64) -> AppResult<Vec<LatencySample>> {
    let mut stmt=conn.prepare("SELECT id,created_at,mode,rvc_profile_id,vad_to_stt_ms,rag_ms,llm_total_ms,tts_ms,rvc_ms,voice_switch_ms,post_speech_ttfa_ms,success,error_stage FROM latency_samples ORDER BY id DESC LIMIT ?1")?;
    Ok(stmt
        .query_map([limit], |r| {
            Ok(LatencySample {
                id: r.get(0)?,
                created_at: r.get(1)?,
                mode: r.get(2)?,
                rvc_profile_id: r.get(3)?,
                vad_to_stt_ms: r.get(4)?,
                rag_ms: r.get(5)?,
                llm_total_ms: r.get(6)?,
                tts_ms: r.get(7)?,
                rvc_ms: r.get(8)?,
                voice_switch_ms: r.get(9)?,
                post_speech_ttfa_ms: r.get(10)?,
                success: r.get(11)?,
                error_stage: r.get(12)?,
            })
        })?
        .collect::<Result<_, _>>()?)
}
