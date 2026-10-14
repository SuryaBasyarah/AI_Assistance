use crate::{error::AppResult, models::RvcProfile};
use chrono::Utc;
use rusqlite::{Connection, params};

pub fn sync(conn: &Connection, profiles: &[RvcProfile]) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    for p in profiles.iter().filter(|p| p.valid) {
        conn.execute(
            "INSERT INTO voice_profiles(profile_id,display_name,pitch,f0_method,index_rate,protect,enabled,last_seen_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?8)
             ON CONFLICT(profile_id) DO UPDATE SET display_name=excluded.display_name,last_seen_at=excluded.last_seen_at",
            params![p.id,p.display_name,p.pitch,p.f0_method,p.index_rate,p.protect,p.enabled,now],
        )?;
    }
    Ok(())
}

pub fn apply_overrides(conn: &Connection, profile: &mut RvcProfile) -> AppResult<()> {
    let result = conn.query_row(
        "SELECT display_name,pitch,f0_method,index_rate,protect,enabled FROM voice_profiles WHERE profile_id=?1",
        [&profile.id],
        |r| Ok((r.get::<_,String>(0)?,r.get::<_,i32>(1)?,r.get::<_,String>(2)?,r.get::<_,f32>(3)?,r.get::<_,f32>(4)?,r.get::<_,bool>(5)?)),
    );
    if let Ok((name, pitch, f0, index_rate, protect, enabled)) = result {
        profile.display_name = name;
        profile.pitch = pitch;
        profile.f0_method = f0;
        profile.index_rate = index_rate;
        profile.protect = protect;
        profile.enabled = enabled;
    }
    Ok(())
}

pub fn update(
    conn: &Connection,
    id: &str,
    pitch: i32,
    f0: &str,
    index_rate: f32,
    protect: f32,
) -> AppResult<()> {
    conn.execute(
        "UPDATE voice_profiles SET pitch=?2,f0_method=?3,index_rate=?4,protect=?5,updated_at=?6 WHERE profile_id=?1",
        params![id,pitch,f0,index_rate,protect,Utc::now().to_rfc3339()],
    )?;
    Ok(())
}
