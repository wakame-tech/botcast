use super::EpisodeScraper;
use crate::{
    script::{Episode, Serif},
    voicevox::Speaker,
};
use scraper::{Html, Selector};

struct Narou;

impl EpisodeScraper for Narou {
    fn scrape(&self, html: &Html) -> anyhow::Result<Episode> {
        Ok(Episode::new(
            get_title(&html)?,
            get_body(&html)?
                .split("。")
                .map(|s| Serif::new(format!("{}。", s), Speaker::ZundaNormal))
                .collect(),
        ))
    }
}

fn clean_text(text: &str) -> String {
    text.replace("\n", "").replace("　", "")
}

fn get_title(html: &Html) -> anyhow::Result<String> {
    let title_selector = Selector::parse("title").unwrap();
    let mut title = html.select(&title_selector);
    let title = title.next().unwrap();
    let title = title
        .text()
        .next()
        .map(|s| s.to_string())
        .ok_or(anyhow::anyhow!("No title found"))?;
    let parts = title.split("-").collect::<Vec<_>>();
    let episode_title = parts[1].trim().to_string();
    Ok(clean_text(&episode_title))
}

fn get_body(html: &Html) -> anyhow::Result<String> {
    let body_selector = Selector::parse("#novel_honbun").unwrap();
    let mut body = html.select(&body_selector);
    let body = body.next().unwrap();
    let body = body.text().collect::<String>();
    Ok(clean_text(&body))
}

#[cfg(test)]
mod tests {
    use super::{get_body, Narou};
    use crate::scrape::{narou::get_title, EpisodeScraper};
    use scraper::Html;
    use std::{fs::File, io::Read, path::PathBuf};

    fn read_html(path: &str) -> anyhow::Result<Html> {
        let mut f = File::open(PathBuf::from(path))?;
        let mut html = String::new();
        f.read_to_string(&mut html)?;
        let html = Html::parse_document(&html);
        Ok(html)
    }

    #[test]
    fn scrape_title() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let title = get_title(&html)?;
        assert_eq!(title, "0歳3ヶ月―――神童セフィリア");
        Ok(())
    }

    #[test]
    fn scrape_body() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let body = get_body(&html)?;
        assert!(body.starts_with("労働とは、"));
        Ok(())
    }

    #[test]
    fn scrape_episode() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let episode = Narou.scrape(&html)?;
        dbg!(episode);
        Ok(())
    }
}
