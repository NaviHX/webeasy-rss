#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod post;
mod post_crawler;
mod args;

use clap::Parser;
use futures::stream::StreamExt;
use rss::ItemBuilder;
use std::sync::{Arc, RwLock};
use warp::Filter;

const TITLE: &str = "NHK Web Easy RSS Feed";
const LINK: &str = "https://www3.nhk.or.jp/news/easy/";
const DESCRIPTION: &str = "A 3rd-party NHK Web Easy RSS Feed";

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let args = args::Args::parse();
    let app_port = args.port;
    let app_crawl_delay = args.crawl_delay;
    let app_request_delay = args.request_delay;

    let channel_handle = Arc::new(RwLock::new(
        rss::ChannelBuilder::default()
            .title(TITLE.to_string())
            .link(LINK.to_string())
            .description(DESCRIPTION.to_string())
            .build(),
    ));
    let rss_server = warp::path("rss").and(warp::get()).map({
        let channel_handle = channel_handle.clone();
        move || channel_handle.read().unwrap().to_string()
    });

    tokio::spawn(async move { warp::serve(rss_server).run(([127, 0, 0, 1], app_port)).await });

    loop {
        let mut news_crawler = post_crawler::NhkWebEasyCrawler::new(voyager::RequestDelay::Fixed(
            std::time::Duration::from_millis(app_request_delay),
        ));
        let mut posts: Vec<rss::Item> = vec![];

        while let Some(post) = news_crawler.next().await {
            let Ok(post) = post else { continue };
            let p = ItemBuilder::default()
                .title(Some(post.title))
                .link(Some(post.url))
                .content(Some(post.content))
                .pub_date(Some(post.pub_date))
                .build();
            posts.push(p);
        }

        let channel = rss::ChannelBuilder::default()
            .title(TITLE.to_string())
            .link(LINK.to_string())
            .description(DESCRIPTION.to_string())
            .items(posts)
            .build();

        *(channel_handle.write().unwrap()) = channel;
        debug!("Crawled NHK site");
        tokio::time::sleep(tokio::time::Duration::from_millis(app_crawl_delay)).await;
    }
}
