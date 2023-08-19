pub struct NhkWebEasyCrawler;

#[derive(Debug, Clone)]
pub enum NhkWebEasyCrawlerState {
    TopList,
    Post { title: String, id: String, pub_date: String },
}

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TopNews {
    news_id: String,
    news_prearranged_time: String,
    title: String,
}

pub fn construct_news_url(news_id: &str) -> String {
    format!(
        "https://www3.nhk.or.jp/news/easy/{}/{}.html",
        news_id, news_id
    )
}

use crate::post::Post;
use anyhow::Result;
use voyager::scraper::Selector;
use voyager::Collector;
use voyager::{CrawlerConfig, Scraper};
use chrono::{TimeZone, FixedOffset};

impl NhkWebEasyCrawler {
    /// Create a `Collector` and send the init request to NHK website
    pub fn new(delay: voyager::RequestDelay) -> Collector<Self> {
        let config = CrawlerConfig::default().allow_domain_with_delay("www3.nhk.or.jp", delay);
        let mut collector = Collector::new(Self, config);
        collector.crawler_mut().visit_with_state("https://www3.nhk.or.jp/news/easy/top-list.json", NhkWebEasyCrawlerState::TopList);
        collector
    }
}

const HOUR: i32 = 3600;
const UTCP9: Option<FixedOffset> = FixedOffset::east_opt(9 * HOUR);

impl Scraper for NhkWebEasyCrawler {
    type Output = Post;
    type State = NhkWebEasyCrawlerState;

    fn scrape(
        &mut self,
        response: voyager::Response<Self::State>,
        crawler: &mut voyager::Crawler<Self>,
    ) -> Result<Option<Self::Output>> {
        if let Some(state) = response.state.clone() {
            match state {
                NhkWebEasyCrawlerState::TopList => {
                    let json_str = response.text;
                    let news_list: Vec<TopNews> = serde_json::from_str(json_str.as_str())?;

                    for news_item in news_list {
                        let news_url = construct_news_url(&news_item.news_id);
                        let id = news_item.news_id;
                        let title = news_item.title;
                        let pub_date = news_item.news_prearranged_time;
                        let parsed_pub_date = UTCP9.unwrap().datetime_from_str(&pub_date, "%Y-%m-%d %H:%M:%S")?;
                        let parsed_pub_date = parsed_pub_date.to_rfc2822();
                        debug!("News Post: {title} {id}");
                        crawler
                            .visit_with_state(news_url, NhkWebEasyCrawlerState::Post { title, id, pub_date: parsed_pub_date });
                    }
                }
                NhkWebEasyCrawlerState::Post { title, id, pub_date } => {
                    let html = response.html();
                    let selector = Selector::parse("#js-article-body").unwrap();
                    let mut selected = html.select(&selector);
                    let url = construct_news_url(&id);

                    // HACK `selected` is an iterator but we only need the first element.
                    if let Some(article_element) = selected.next() {
                        let content = article_element.inner_html();
                        debug!("Get Post: {title} {id}");
                        return Ok(Some(Post {
                            title,
                            content,
                            url,
                            pub_date,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }
}
