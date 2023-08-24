#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod args;
mod post;
mod post_crawler;

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
    let args::Args {
        ip,
        port,
        crawl_delay,
        request_delay,
    } = args::Args::parse();

    let channel_handle = Arc::new(RwLock::new(
        rss::ChannelBuilder::default()
            .title(TITLE.to_string())
            .link(LINK.to_string())
            .description(DESCRIPTION.to_string())
            .build(),
    ));
    let rss_server = warp::path("rss").and(warp::get()).map({
        let channel_handle = channel_handle.clone();
        move || {
            info!("Receive an RSS request");
            let reply = channel_handle.read().unwrap().to_string();
            info!("{:?}", channel_handle.read().unwrap().items());
            info!("Reply: {}", reply);
            reply
        }
    });

    tokio::spawn(async move { warp::serve(rss_server).run((ip, port)).await });

    loop {
        let mut news_crawler = post_crawler::NhkWebEasyCrawler::new(voyager::RequestDelay::Fixed(
            std::time::Duration::from_millis(request_delay),
        ));
        let mut posts: Vec<rss::Item> = vec![];

        while let Some(post) = news_crawler.next().await {
            match post {
                Err(e) => {
                    info!("{e}");
                    continue;
                }
                Ok(post) => {
                    let p = ItemBuilder::default()
                        .title(Some(post.title))
                        .link(Some(post.url))
                        .content(Some(post.content))
                        .pub_date(Some(post.pub_date))
                        .build();
                    posts.push(p);
                }
            }
        }

        let channel = rss::ChannelBuilder::default()
            .title(TITLE.to_string())
            .link(LINK.to_string())
            .description(DESCRIPTION.to_string())
            .items(posts)
            .build();

        *(channel_handle.write().unwrap()) = channel;
        info!("Crawled NHK site");
        tokio::time::sleep(tokio::time::Duration::from_millis(crawl_delay)).await;
    }
}
