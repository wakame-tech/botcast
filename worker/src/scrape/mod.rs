use crate::script::Episode;
use scraper::Html;

pub(crate) mod narou;

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";

#[derive(Debug)]
struct Scraper {
    client: reqwest::Client,
}

impl Scraper {
    pub(crate) fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        }
    }
}

trait EpisodeScraper {
    fn scrape(&self, html: &Html) -> anyhow::Result<Episode>;
}
