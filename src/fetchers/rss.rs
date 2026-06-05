use crate::types::{Item, sanitize};
use feed_rs::parser;
use std::collections::HashMap;

fn parse_rss(xml: &str, source: &str) -> Vec<Item> {
    let feed = match parser::parse(xml.as_bytes()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[rss] failed to parse {}: {}", source, e);
            return vec![];
        }
    };

    feed.entries
        .into_iter()
        .filter_map(|entry| {
            let title = entry.title?.content;
            let id = if entry.id.is_empty() {
                entry.links.first().map(|l| l.href.clone())?
            } else {
                entry.id
            };
            let tags: Vec<String> = entry.categories.iter().map(|c| c.term.clone()).collect();
            let summary = entry
                .content
                .and_then(|c| c.body)
                .or_else(|| entry.summary.map(|s| s.content))
                .map(|s| sanitize(&s, 600));
            Some(Item {
                id,
                title,
                tags,
                source: source.to_string(),
                summary,
                bypass_tag_filter: false,
                metadata: HashMap::new(),
            })
        })
        .collect()
}

pub async fn fetch_ars_technica(client: &reqwest::Client) -> Vec<Item> {
    let xml = match client
        .get("https://feeds.arstechnica.com/arstechnica/index")
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[rss] ars technica body error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[rss] ars technica fetch error: {}", e);
            return vec![];
        }
    };
    parse_rss(&xml, "ars-technica")
}

pub async fn fetch_the_new_stack(client: &reqwest::Client) -> Vec<Item> {
    let xml = match client
        .get("https://thenewstack.io/feed/")
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[rss] the new stack body error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[rss] the new stack fetch error: {}", e);
            return vec![];
        }
    };
    parse_rss(&xml, "the-new-stack")
}

pub async fn enrich_devto(client: &reqwest::Client, item: &mut Item) {
    let id = match item.metadata.get("devto_id") {
        Some(id_str) => id_str.clone(),
        None => return,
    };

    let api_url = format!("https://dev.to/api/articles/{}", id);
    let article: serde_json::Value = match client.get(&api_url).send().await {
        Ok(resp) => match resp.json().await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[rss] dev.to enrich {} json error: {}", item.id, e);
                return;
            }
        },
        Err(e) => {
            eprintln!("[rss] dev.to enrich {} fetch error: {}", item.id, e);
            return;
        }
    };

    if let Some(body) = article.get("body_markdown").and_then(|v| v.as_str()) {
        let paragraphs: Vec<&str> = body
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!') && l.len() > 40)
            .take(3)
            .collect();
        if !paragraphs.is_empty() {
            let summary = paragraphs.join(" ");
            item.summary = Some(sanitize(&summary, 600));
        }
    }
}

pub async fn fetch_devto(client: &reqwest::Client, tags: &[String]) -> Vec<Item> {
    let mut all_items = Vec::new();
    let default_tags = ["ai", "machinelearning", "devops", "docker", "kubernetes"];
    let tags_to_use: Vec<&str> = if tags.is_empty() {
        default_tags.to_vec()
    } else {
        tags.iter().map(|t| t.as_str()).collect()
    };

    for tag in tags_to_use {
        let url = format!("https://dev.to/api/articles?tag={}&top=7", tag);
        let articles: Vec<serde_json::Value> = match client.get(&url).send().await {
            Ok(resp) => match resp.json().await {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("[rss] dev.to {} json error: {}", tag, e);
                    continue;
                }
            },
            Err(e) => {
                eprintln!("[rss] dev.to {} fetch error: {}", tag, e);
                continue;
            }
        };

        for article in articles {
            let url = match article.get("url").and_then(|v| v.as_str()) {
                Some(u) => u.to_string(),
                None => continue,
            };
            let article_id = match article.get("id").and_then(|v| v.as_i64()) {
                Some(id) => id.to_string(),
                None => continue,
            };
            let title = match article.get("title").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => continue,
            };
            let article_tags: Vec<String> = article
                .get("tag_list")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let summary = article
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);

            let mut metadata = HashMap::new();
            metadata.insert("devto_id".to_string(), article_id);

            all_items.push(Item {
                id: url,
                title,
                tags: article_tags,
                source: "dev.to".to_string(),
                summary,
                bypass_tag_filter: false,
                metadata,
            });
        }
    }
    all_items
}

pub async fn fetch_schneier(client: &reqwest::Client) -> Vec<Item> {
    let xml = match client
        .get("https://www.schneier.com/feed/atom/")
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[rss] schneier body error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[rss] schneier fetch error: {}", e);
            return vec![];
        }
    };
    parse_rss(&xml, "schneier")
}

pub async fn fetch_hacker_news_rss(client: &reqwest::Client) -> Vec<Item> {
    let xml = match client
        .get("https://hnrss.org/frontpage")
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[rss] hacker news rss body error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[rss] hacker news rss fetch error: {}", e);
            return vec![];
        }
    };
    parse_rss(&xml, "hacker-news")
}

pub async fn fetch_krebs(client: &reqwest::Client) -> Vec<Item> {
    let xml = match client
        .get("https://krebsonsecurity.com/feed/")
        .send()
        .await
    {
        Ok(resp) => match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[rss] krebs body error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[rss] krebs fetch error: {}", e);
            return vec![];
        }
    };
    parse_rss(&xml, "krebs-on-security")
}
