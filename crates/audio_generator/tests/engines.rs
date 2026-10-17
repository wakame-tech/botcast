//! 各窓口をフェイクの HTTP サーバーに向けて、リクエストの形・トークン・参照音声の同期を確かめる。

use audio_generator::{
    config::{Secret, TtsConfig},
    models::{ResolvedVoice, VoiceParams},
    registry::TtsRegistry,
    tts::{openai::OpenAiEngine, voicevox::VoicevoxEngine, SynthesisRequest, TtsEngine},
};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
    Router,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Debug)]
struct Captured {
    method: Method,
    path: String,
    query: String,
    auth: Option<String>,
    content_type: Option<String>,
    body: Vec<u8>,
}

impl Captured {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }

    fn query_param(&self, key: &str) -> Option<String> {
        self.query.split('&').find_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            (k == key).then(|| urlencoding::decode(v).unwrap().into_owned())
        })
    }
}

type Responder = Arc<dyn Fn(&Captured) -> (StatusCode, Vec<u8>) + Send + Sync>;

#[derive(Clone)]
struct FakeServer {
    captured: Arc<Mutex<Vec<Captured>>>,
    responder: Responder,
}

async fn handle(
    State(server): State<FakeServer>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let header = |name: &str| headers.get(name).map(|v| v.to_str().unwrap().to_string());
    let captured = Captured {
        method,
        path: uri.path().to_string(),
        query: uri.query().unwrap_or_default().to_string(),
        auth: header("authorization"),
        content_type: header("content-type"),
        body: body.to_vec(),
    };
    let res = (server.responder)(&captured);
    server.captured.lock().unwrap().push(captured);
    res
}

/// フェイクのサーバーを立て、ベースの URL と受けたリクエストの一覧を返す。
async fn spawn_server(
    responder: impl Fn(&Captured) -> (StatusCode, Vec<u8>) + Send + Sync + 'static,
) -> (String, Arc<Mutex<Vec<Captured>>>) {
    let server = FakeServer {
        captured: Arc::default(),
        responder: Arc::new(responder),
    };
    let captured = server.captured.clone();
    let app = Router::new().fallback(handle).with_state(server);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), captured)
}

const TIMEOUT: Duration = Duration::from_secs(10);

fn voicevox_responder(c: &Captured) -> (StatusCode, Vec<u8>) {
    match c.path.as_str() {
        "/audio_query" => (
            StatusCode::OK,
            json!({"speedScale": 1.0, "pitchScale": 0.0, "accent_phrases": []})
                .to_string()
                .into_bytes(),
        ),
        "/synthesis" => (StatusCode::OK, b"RIFF-fake-wav".to_vec()),
        "/version" => (StatusCode::OK, b"\"0.14.0\"".to_vec()),
        _ => (StatusCode::NOT_FOUND, vec![]),
    }
}

#[tokio::test]
async fn voicevox_request_shape() {
    let (url, captured) = spawn_server(voicevox_responder).await;
    let engine = VoicevoxEngine::new(&url, TIMEOUT).unwrap();

    let mut extra = serde_json::Map::new();
    extra.insert("outputStereo".to_string(), json!(true));
    let voice = ResolvedVoice {
        engine: "voicevox".to_string(),
        voice_id: "3".to_string(),
        params: VoiceParams {
            speed: Some(1.3),
            volume: Some(0.8),
            extra,
            ..Default::default()
        },
        style: Some("happy".to_string()),
        styles: BTreeMap::from([("happy".to_string(), "5".to_string())]),
        seed: Some(42),
        ref_audio: None,
    };
    let audio = engine
        .synthesize(SynthesisRequest {
            text: "こんにちは。",
            voice: &voice,
        })
        .await
        .unwrap();
    assert_eq!(audio, b"RIFF-fake-wav");

    let captured = captured.lock().unwrap().clone();
    assert_eq!(captured.len(), 2);
    let (query, synthesis) = (&captured[0], &captured[1]);
    assert_eq!(
        (&query.method, query.path.as_str()),
        (&Method::POST, "/audio_query")
    );
    assert_eq!(query.query_param("text").as_deref(), Some("こんにちは。"));
    // style は styles で引いたスタイル ID になる
    assert_eq!(query.query_param("speaker").as_deref(), Some("5"));
    assert_eq!(
        (&synthesis.method, synthesis.path.as_str()),
        (&Method::POST, "/synthesis")
    );
    assert_eq!(synthesis.query_param("speaker").as_deref(), Some("5"));
    assert_eq!(
        synthesis.json(),
        json!({
            "speedScale": 1.3,
            "pitchScale": 0.0,
            "volumeScale": 0.8,
            "outputStereo": true,
            "accent_phrases": []
        })
    );
}

#[tokio::test]
async fn voicevox_unknown_style_uses_default_and_strips_emoji() {
    let (url, captured) = spawn_server(voicevox_responder).await;
    let engine = VoicevoxEngine::new(&url, TIMEOUT).unwrap();
    let voice = ResolvedVoice {
        engine: "voicevox".to_string(),
        voice_id: "3".to_string(),
        style: Some("sad".to_string()),
        ..Default::default()
    };
    assert_eq!(engine.prepare_text("やった🎉 "), "やった");
    engine
        .synthesize(SynthesisRequest {
            text: "やった",
            voice: &voice,
        })
        .await
        .unwrap();
    assert_eq!(
        captured.lock().unwrap()[0]
            .query_param("speaker")
            .as_deref(),
        Some("3")
    );

    engine.health().await.unwrap();
    let last = captured.lock().unwrap().last().cloned().unwrap();
    assert_eq!((last.method, last.path.as_str()), (Method::GET, "/version"));
}

#[tokio::test]
async fn openai_request_shape_with_token() {
    let (url, captured) = spawn_server(|c| match c.path.as_str() {
        "/v1/audio/speech" => (StatusCode::OK, b"RIFF-fake-wav".to_vec()),
        "/v1/models" => (StatusCode::OK, br#"{"object":"list","data":[]}"#.to_vec()),
        _ => (StatusCode::NOT_FOUND, vec![]),
    })
    .await;
    let engine = OpenAiEngine::new(
        format!("{url}/v1/"),
        Some(Secret::new("secret-token")),
        Some("irodori-tts-v3".to_string()),
        TIMEOUT,
    )
    .unwrap();
    assert!(!format!("{engine:?}").contains("secret-token"));
    // 絵文字はそのまま渡す
    assert_eq!(engine.prepare_text("やった🎉"), "やった🎉");

    let mut extra = serde_json::Map::new();
    extra.insert("stream_format".to_string(), json!("audio"));
    let voice = ResolvedVoice {
        engine: "irodori".to_string(),
        voice_id: "alice".to_string(),
        params: VoiceParams {
            speed: Some(1.2),
            pitch: Some(0.1),
            extra,
            ..Default::default()
        },
        style: Some("明るく元気に".to_string()),
        seed: Some(1234),
        ..Default::default()
    };
    let audio = engine
        .synthesize(SynthesisRequest {
            text: "やった🎉",
            voice: &voice,
        })
        .await
        .unwrap();
    assert_eq!(audio, b"RIFF-fake-wav");
    engine.health().await.unwrap();

    let captured = captured.lock().unwrap().clone();
    let speech = &captured[0];
    assert_eq!(
        (&speech.method, speech.path.as_str()),
        (&Method::POST, "/v1/audio/speech")
    );
    assert_eq!(speech.auth.as_deref(), Some("Bearer secret-token"));
    assert_eq!(
        speech.json(),
        json!({
            "model": "irodori-tts-v3",
            "input": "やった🎉",
            "voice": "alice",
            "response_format": "wav",
            "speed": 1.2,
            "irodori": {"seed": 1234, "caption": "明るく元気に"},
            "stream_format": "audio"
        })
    );
    let health = &captured[1];
    assert_eq!(
        (&health.method, health.path.as_str()),
        (&Method::GET, "/v1/models")
    );
    assert_eq!(health.auth.as_deref(), Some("Bearer secret-token"));
}

#[tokio::test]
async fn openai_without_token_and_options() {
    let (url, captured) = spawn_server(|_| (StatusCode::OK, b"RIFF-fake-wav".to_vec())).await;
    let engine = OpenAiEngine::new(&url, None, None, TIMEOUT).unwrap();
    let voice = ResolvedVoice {
        engine: "openai".to_string(),
        voice_id: "alloy".to_string(),
        ..Default::default()
    };
    engine
        .synthesize(SynthesisRequest {
            text: "こんにちは",
            voice: &voice,
        })
        .await
        .unwrap();
    let captured = captured.lock().unwrap().clone();
    assert_eq!(captured[0].path, "/audio/speech");
    assert_eq!(captured[0].auth, None);
    assert_eq!(
        captured[0].json(),
        json!({
            "model": "irodori-tts",
            "input": "こんにちは",
            "voice": "alloy",
            "response_format": "wav"
        })
    );
}

/// `/v1/audio/voices` を持つ Irodori-TTS-Server のフェイク
fn irodori_voices_responder(
    voices: Arc<Mutex<HashSet<String>>>,
) -> impl Fn(&Captured) -> (StatusCode, Vec<u8>) + Send + Sync + 'static {
    move |c| {
        let mut voices = voices.lock().unwrap();
        match (&c.method, c.path.as_str()) {
            (&Method::GET, "/v1/audio/voices") => {
                let data: Vec<Value> = voices.iter().map(|id| json!({"id": id})).collect();
                (
                    StatusCode::OK,
                    json!({"object": "list", "data": data})
                        .to_string()
                        .into_bytes(),
                )
            }
            (&Method::POST, "/v1/audio/voices") => {
                let body = String::from_utf8_lossy(&c.body);
                let id = body
                    .split("name=\"voice_id\"\r\n\r\n")
                    .nth(1)
                    .and_then(|rest| rest.split("\r\n").next())
                    .unwrap()
                    .to_string();
                if !voices.insert(id) {
                    return (StatusCode::CONFLICT, vec![]);
                }
                (StatusCode::CREATED, b"{}".to_vec())
            }
            (&Method::PUT, path) if path.starts_with("/v1/audio/voices/") => {
                (StatusCode::OK, b"{}".to_vec())
            }
            _ => (StatusCode::NOT_FOUND, vec![]),
        }
    }
}

#[tokio::test]
async fn openai_ensure_voice_uploads_only_when_needed() {
    let voices = Arc::new(Mutex::new(HashSet::new()));
    let (url, captured) = spawn_server(irodori_voices_responder(voices.clone())).await;
    let engine = OpenAiEngine::new(
        format!("{url}/v1"),
        Some(Secret::new("secret-token")),
        None,
        TIMEOUT,
    )
    .unwrap();
    let take = || std::mem::take(&mut *captured.lock().unwrap());

    // サーバーに無い → POST（multipart の file と voice_id）
    engine
        .ensure_voice("alice", "hash-1", b"RIFF-alice")
        .await
        .unwrap();
    let requests = take();
    assert_eq!(
        requests
            .iter()
            .map(|c| (c.method.clone(), c.path.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (Method::GET, "/v1/audio/voices"),
            (Method::POST, "/v1/audio/voices")
        ]
    );
    let upload = &requests[1];
    assert!(upload.auth.as_deref() == Some("Bearer secret-token"));
    assert!(upload
        .content_type
        .as_deref()
        .unwrap()
        .starts_with("multipart/form-data"));
    let body = String::from_utf8_lossy(&upload.body);
    assert!(
        body.contains("name=\"file\"; filename=\"alice.wav\""),
        "{body}"
    );
    assert!(body.contains("RIFF-alice"));
    assert!(
        body.contains("name=\"voice_id\"\r\n\r\nalice\r\n"),
        "{body}"
    );
    assert!(voices.lock().unwrap().contains("alice"));

    // 同じハッシュ → 何もしない
    engine
        .ensure_voice("alice", "hash-1", b"RIFF-alice")
        .await
        .unwrap();
    assert!(take().is_empty());

    // ハッシュが違う → 置き換える
    engine
        .ensure_voice("alice", "hash-2", b"RIFF-alice-2")
        .await
        .unwrap();
    let requests = take();
    assert_eq!(
        requests
            .iter()
            .map(|c| (c.method.clone(), c.path.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (Method::GET, "/v1/audio/voices"),
            (Method::PUT, "/v1/audio/voices/alice")
        ]
    );
    assert!(String::from_utf8_lossy(&requests[1].body).contains("RIFF-alice-2"));

    // 別のプロセス（エンジン）からはハッシュが分からないので、サーバーにあっても置き換える
    let other = OpenAiEngine::new(format!("{url}/v1"), None, None, TIMEOUT).unwrap();
    other
        .ensure_voice("alice", "hash-2", b"RIFF-alice-2")
        .await
        .unwrap();
    assert_eq!(take()[1].method, Method::PUT);
}

#[tokio::test]
async fn voicevox_ensure_voice_is_noop() {
    let (url, captured) = spawn_server(voicevox_responder).await;
    let engine = VoicevoxEngine::new(&url, TIMEOUT).unwrap();
    engine.ensure_voice("3", "hash", b"RIFF").await.unwrap();
    assert!(captured.lock().unwrap().is_empty());
}

#[tokio::test]
async fn registry_health_check_names_unreachable_engine() {
    let (ok_url, _) = spawn_server(voicevox_responder).await;
    let (ng_url, _) = spawn_server(|_| (StatusCode::INTERNAL_SERVER_ERROR, vec![])).await;
    let vars = BTreeMap::from([
        ("TTS_ENGINES", "voicevox,irodori".to_string()),
        ("TTS_ENGINE_VOICEVOX_KIND", "voicevox".to_string()),
        ("TTS_ENGINE_VOICEVOX_ENDPOINT", ok_url),
        ("TTS_ENGINE_IRODORI_KIND", "openai".to_string()),
        ("TTS_ENGINE_IRODORI_ENDPOINT", format!("{ng_url}/v1")),
    ]);
    let config = TtsConfig::from_lookup(|k| vars.get(k).cloned()).unwrap();
    let registry = TtsRegistry::from_config(&config).unwrap();
    assert_eq!(registry.default_engine().unwrap(), "voicevox");
    assert_eq!(
        registry.engine_names().collect::<Vec<_>>(),
        vec!["irodori", "voicevox"]
    );

    registry.health_check(["voicevox"]).await.unwrap();
    let err = registry
        .health_check(["voicevox", "irodori"])
        .await
        .unwrap_err();
    assert!(
        format!("{err:#}").contains("TTS engine \"irodori\" is unreachable"),
        "{err:#}"
    );
    let err = registry.health_check(["unknown"]).await.unwrap_err();
    assert!(
        err.to_string().contains("\"unknown\" is not configured"),
        "{err}"
    );
}

#[test]
fn empty_registry_has_no_default_engine() {
    let registry = TtsRegistry::from_config(&TtsConfig::default()).unwrap();
    assert!(registry.default_engine().is_err());
}
