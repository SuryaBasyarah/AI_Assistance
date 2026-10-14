//! Microphone capture is intentionally performed by the WebView MediaRecorder.
//! Recorded bytes cross IPC once and are transcribed by the Rust backend so the API key never leaves Rust.
