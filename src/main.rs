mod fetchers;
mod types;

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use types::{Item, TagsConfig};

fn load_tags() -> Vec<String> {
    let path = match env::var("TAGS_SOURCE").as_deref() {
        Ok("local") => PathBuf::from("newsletter-tags.json"),
        _ => {
            let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            PathBuf::from(home).join(".hermes/newsletter-tags.json")
        }
    };
    match std::fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str::<TagsConfig>(&data) {
            Ok(config) => config.tags,
            Err(e) => {
                eprintln!("[main] failed to parse {}: {}", path.display(), e);
                vec![]
            }
        },
        Err(e) => {
            eprintln!("[main] failed to read {}: {}", path.display(), e);
            vec![]
        }
    }
}

fn matches_tags(item: &Item, tags: &[String]) -> bool {
    if tags.is_empty() {
        return true;
    }
    let title_lower = item.title.to_lowercase();
    tags.iter().any(|tag| {
        let tag_lower = tag.to_lowercase();
        item.tags.iter().any(|t| t.to_lowercase() == tag_lower)
            || title_lower.split_whitespace().any(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric()) == tag_lower
            })
    })
}

#[tokio::main]
async fn main() {
    let tags = load_tags();
    let client = reqwest::Client::builder()
        .user_agent("hermes-newsletter/0.1")
        .build()
        .expect("failed to build http client");

    let (
        ars,
        tns,
        devto,
        schneier,
        hn_rss,
        krebs,
        cisa,
        nvd,
        pwc,
        hf,
        hn_api,
    ) = tokio::join!(
        fetchers::rss::fetch_ars_technica(&client),
        fetchers::rss::fetch_the_new_stack(&client),
        fetchers::rss::fetch_devto(&client, &tags),
        fetchers::rss::fetch_schneier(&client),
        fetchers::rss::fetch_hacker_news_rss(&client),
        fetchers::rss::fetch_krebs(&client),
        fetchers::api::fetch_cisa_kev(&client),
        fetchers::api::fetch_nvd_cve(&client),
        fetchers::api::fetch_papers_with_code(&client),
        fetchers::api::fetch_huggingface_trending(&client),
        fetchers::api::fetch_hacker_news_api(&client),
    );

    let mut all_items: Vec<Item> = Vec::new();
    all_items.extend(ars);
    all_items.extend(tns);
    all_items.extend(devto);
    all_items.extend(schneier);
    all_items.extend(hn_rss);
    all_items.extend(krebs);
    all_items.extend(cisa);
    all_items.extend(nvd);
    all_items.extend(pwc);
    all_items.extend(hf);
    all_items.extend(hn_api);

    let filtered: Vec<Item> = all_items
        .into_iter()
        .filter(|item| item.bypass_tag_filter || matches_tags(item, &tags))
        .collect();

    let output = if let Ok(tldr_ids) = env::var("FETCH_TLDR") {
        let id_set: HashSet<&str> = tldr_ids.split(',').map(|s| s.trim()).collect();
        let mut selected: Vec<Item> = filtered
            .iter()
            .filter(|item| id_set.contains(item.id.as_str()))
            .cloned()
            .collect();

        for item in &mut selected {
            if item.id.contains("dev.to/") {
                fetchers::rss::enrich_devto(&client, item).await;
            }
        }

        serde_json::to_string(&selected).unwrap_or_else(|e| {
            eprintln!("[main] json serialize error: {}", e);
            "[]".to_string()
        })
    } else {
        let brief: Vec<serde_json::Value> = filtered
            .iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "title": item.title,
                    "tags": item.tags,
                })
            })
            .collect();
        serde_json::to_string(&brief).unwrap_or_else(|e| {
            eprintln!("[main] json serialize error: {}", e);
            "[]".to_string()
        })
    };

    match env::var("OUTPUT_FILE") {
        Ok(path) => {
            if let Err(e) = std::fs::write(&path, &output) {
                eprintln!("[main] failed to write to {}: {}", path, e);
                println!("{}", output);
            } else {
                eprintln!("[main] output written to {}", path);
            }
        }
        Err(_) => println!("{}", output),
    }
}
