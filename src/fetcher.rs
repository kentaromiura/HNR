// Define the Hackernews API
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;

use chrono::{DateTime, Utc};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

// caching stories_id and item to implement lazy load,
// originally part of the HackerNews struct but made static as easier to work with.
type Cache = HashMap<usize, StoryItem>;
static CACHE: Lazy<ArcSwap<Cache>> = Lazy::new(|| ArcSwap::default());
static STORIES_ID: Lazy<ArcSwap<Vec<usize>>> = Lazy::new(|| ArcSwap::default());

// API from example MIT code here: https://github.com/DioxusLabs/dioxus/blob/main/packages/fullstack/examples/hackernews/src/main.rs#L230
pub static BASE_API_URL: &str = "https://hacker-news.firebaseio.com/v0/";
pub static ITEM_API: &str = "item/";
// pub static USER_API: &str = "user/";
// const COMMENT_DEPTH: i64 = 2;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoryItem {
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub descendants: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub kids: Vec<i64>,
    pub r#type: String,
}

pub struct HackerNews {
    pub client: reqwest::Client,
}

#[derive(Debug, Clone)]
struct AnError;

impl fmt::Display for AnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error")
    }
}

impl Error for AnError {}

impl HackerNews {
    async fn fill_stories(&self) -> Result<(), reqwest::Error> {
        let url = format!("{}topstories.json", BASE_API_URL);
        let ids = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Vec<usize>>()
            .await?;
        STORIES_ID.rcu(|cache| {
            let mut cache = Vec::clone(&cache);

            for id in &ids {
                cache.push(*id);
            }
            cache
        });

        Ok(())
    }

    async fn get_story_item(&self, id: usize) -> Result<StoryItem, reqwest::Error> {
        let url = format!("{}{}{}.json", BASE_API_URL, ITEM_API, id);
        self.client.get(&url).send().await?.json().await
    }

    pub async fn get_story(&self, story_id: usize) -> Result<StoryItem, reqwest::Error> {
        let cache = CACHE.load();
        if let Some(result) = cache.get(&story_id) {
            return Ok(result.clone());
        }

        let story = self.get_story_item(story_id).await?;
        CACHE.rcu(|cache| {
            let mut cache = HashMap::clone(&cache);
            cache.insert(story_id, story.clone());
            cache
        });
        Ok(story)
    }

    pub async fn get_stories_from_to(
        &self,
        from: usize,
        to: usize,
    ) -> Result<Vec<StoryItem>, Box<dyn Error>> {
        let stories_id = STORIES_ID.load();
        if stories_id.len() == 0 {
            let _ = self.fill_stories().await?;
        }
        let stories_id = STORIES_ID.load();
        let to = std::cmp::min(stories_id.len(), to);
        if from < stories_id.len() {
            Ok(join_all(
                stories_id[from..to]
                    .iter()
                    .map(|story_id| self.get_story(*story_id)),
            )
            .await
            .into_iter()
            .filter_map(|c| c.ok())
            .collect())
        } else {
            Err(Box::<dyn Error>::from(AnError))
        }
    }
}
