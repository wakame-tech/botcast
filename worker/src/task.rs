use crate::{
    scrape::{narou::Narou, EpisodeScraper},
    synthesizer::{
        concat_wavs,
        voicevox::{VoiceVox, VoiceVoxSpeaker},
        Synthesizer,
    },
    DB,
};
use scraper::Html;
use std::{path::PathBuf, time::Duration};
use surrealdb::opt::RecordId;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Task {
    pub(crate) id: RecordId,
    pub(crate) url: String,
}

impl Task {
    pub(crate) fn new(url: String) -> Self {
        Self {
            id: RecordId::from(("task".to_string(), Uuid::new_v4().to_string())),
            url,
        }
    }
}

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";

pub(crate) async fn run_task(task: &Task) -> anyhow::Result<()> {
    println!("Running task: {:?}", task);

    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let res = client.get(&task.url).send().await?;
    if res.status() != reqwest::StatusCode::OK {
        anyhow::bail!("Failed to fetch: {}", res.status());
    }
    let html = res.text().await?;
    let html = Html::parse_document(&html);
    let scraper = Narou;
    let episode = scraper.scrape(html)?;

    let dir = PathBuf::from("temp").join(task.id.id.to_raw());
    if !dir.exists() {
        std::fs::create_dir(&dir)?;
    }

    let episode_path = dir.join("episode.json");
    let f = std::fs::File::create(&episode_path)?;
    serde_json::to_writer_pretty(f, &episode)?;

    let synthesizer = VoiceVox::new();
    let speaker = VoiceVoxSpeaker::ZundaNormal;
    let mut paths = vec![];
    for (i, serif) in episode.serifs.iter().enumerate() {
        let out = dir.join(format!("{}.wav", i));
        println!("Synthesis: ({}) {}", out.display(), serif.text);
        synthesizer.synthesis(&serif.text, &speaker, &out).await?;
        paths.push(out);
    }
    let text_path = dir.join("text.txt");

    let out = dir.join("out.wav");
    concat_wavs(&paths, &text_path, &out).await?;
    Ok(())
}

pub(crate) async fn watch_tasks() {
    loop {
        println!("Watching tasks...");
        let db = DB.clone();
        let tasks: Vec<Task> = db.select("tasks").await.unwrap();
        for task in tasks {
            run_task(&task).await.unwrap();
            let res: Option<Task> = db.delete(&task.id).await.unwrap();
            println!("Deleted: {:?}", res);
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
