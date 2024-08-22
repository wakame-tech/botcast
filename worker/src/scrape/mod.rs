use crate::script::Episode;
use scraper::Html;

pub(crate) mod narou;

pub(crate) trait EpisodeScraper {
    fn scrape(&self, html: Html) -> anyhow::Result<Episode>;
}

#[cfg(test)]
pub(crate) mod tests {
    use scraper::Html;
    use std::{fs::File, io::Read, path::PathBuf};

    pub(crate) fn read_html(path: &str) -> anyhow::Result<Html> {
        let mut f = File::open(PathBuf::from(path))?;
        let mut html = String::new();
        f.read_to_string(&mut html)?;
        let html = Html::parse_document(&html);
        Ok(html)
    }
}
