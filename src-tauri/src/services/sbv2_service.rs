use crate::{
    error::{AppError, AppResult},
    services::AppState,
};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::io::Cursor;

const SBV2_TEXT_LIMIT: usize = 100;

fn split_text(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for character in text.trim().chars() {
        current.push(character);
        let sentence_end = matches!(character, '。' | '！' | '？' | '!' | '?' | '\n');
        if current.chars().count() >= SBV2_TEXT_LIMIT || sentence_end {
            let chunk = current.trim().to_owned();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
            current.clear();
        }
    }

    let remainder = current.trim().to_owned();
    if !remainder.is_empty() {
        chunks.push(remainder);
    }
    chunks
}

async fn synthesize_chunk(state: &AppState, text: &str, style: &str) -> AppResult<Vec<u8>> {
    let response = state
        .http
        .post(format!("{}/voice", state.sbv2_base_url))
        .query(&[
            ("text", text),
            ("model_id", "0"),
            ("speaker_id", "0"),
            ("language", "JP"),
            ("style", style),
            ("style_weight", "1.0"),
            ("length", "1.0"),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "SBV2 synthesis failed: {}",
            response.text().await.unwrap_or_default()
        )));
    }
    Ok(response.bytes().await?.to_vec())
}

fn append_samples(
    writer: &mut WavWriter<Cursor<&mut Vec<u8>>>,
    wav: &[u8],
    expected: WavSpec,
) -> AppResult<()> {
    let mut reader = WavReader::new(Cursor::new(wav))
        .map_err(|error| AppError::from(format!("Invalid SBV2 WAV: {error}")))?;
    if reader.spec() != expected {
        return Err(AppError::from(
            "SBV2 returned incompatible WAV formats between speech chunks",
        ));
    }
    match (expected.sample_format, expected.bits_per_sample) {
        (SampleFormat::Float, _) => {
            for sample in reader.samples::<f32>() {
                writer
                    .write_sample(sample.map_err(|error| AppError::from(error.to_string()))?)
                    .map_err(|error| AppError::from(error.to_string()))?;
            }
        }
        (SampleFormat::Int, 0..=8) => {
            for sample in reader.samples::<i8>() {
                writer
                    .write_sample(sample.map_err(|error| AppError::from(error.to_string()))?)
                    .map_err(|error| AppError::from(error.to_string()))?;
            }
        }
        (SampleFormat::Int, 9..=16) => {
            for sample in reader.samples::<i16>() {
                writer
                    .write_sample(sample.map_err(|error| AppError::from(error.to_string()))?)
                    .map_err(|error| AppError::from(error.to_string()))?;
            }
        }
        (SampleFormat::Int, _) => {
            for sample in reader.samples::<i32>() {
                writer
                    .write_sample(sample.map_err(|error| AppError::from(error.to_string()))?)
                    .map_err(|error| AppError::from(error.to_string()))?;
            }
        }
    }
    Ok(())
}

fn merge_wavs(wavs: &[Vec<u8>]) -> AppResult<Vec<u8>> {
    if wavs.len() == 1 {
        return Ok(wavs[0].clone());
    }
    let spec = WavReader::new(Cursor::new(&wavs[0]))
        .map_err(|error| AppError::from(format!("Invalid SBV2 WAV: {error}")))?
        .spec();
    let mut output = Vec::new();
    {
        let mut writer = WavWriter::new(Cursor::new(&mut output), spec)
            .map_err(|error| AppError::from(error.to_string()))?;
        for wav in wavs {
            append_samples(&mut writer, wav, spec)?;
        }
        writer
            .finalize()
            .map_err(|error| AppError::from(error.to_string()))?;
    }
    Ok(output)
}

pub async fn synthesize_japanese(state: &AppState, text: &str, style: &str) -> AppResult<Vec<u8>> {
    let chunks = split_text(text);
    if chunks.is_empty() {
        return Err(AppError::from("Cannot synthesize empty Japanese text"));
    }

    // model_id=0 has Neutral and numbered custom styles, not Happy/Sad.
    let supported_style = if style == "Neutral" { style } else { "Neutral" };
    let mut wavs = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        wavs.push(synthesize_chunk(state, &chunk, supported_style).await?);
    }
    merge_wavs(&wavs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_at_sentences_and_server_limit() {
        let text = format!("{}。短い文。", "あ".repeat(120));
        let chunks = split_text(&text);
        assert!(chunks.len() >= 3);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 100));
        assert_eq!(chunks.concat(), text);
    }
}

pub fn emotion_style(emotion: &str) -> &'static str {
    match emotion.to_ascii_lowercase().as_str() {
        "happy" | "excited" => "Happy",
        "sad" => "Sad",
        "angry" => "Angry",
        _ => "Neutral",
    }
}
