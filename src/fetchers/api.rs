use crate::types::{Item, sanitize};
use std::collections::HashMap;

pub async fn fetch_cisa_kev(client: &reqwest::Client) -> Vec<Item> {
    let url = "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json";
    let data: serde_json::Value = match client.get(url).send().await {
        Ok(resp) => match resp.json().await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[api] cisa kev json error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[api] cisa kev fetch error: {}", e);
            return vec![];
        }
    };

    let Some(vulns) = data.get("vulnerabilities").and_then(|v| v.as_array()) else {
        eprintln!("[api] cisa kev: no vulnerabilities array");
        return vec![];
    };

    vulns.iter()
        .take(20)
        .filter_map(|v| {
            let cve_id = v.get("cveID")?.as_str()?.to_string();
            let name = v.get("vulnerabilityName")?.as_str()?.to_string();
            let vendor = v.get("vendorProject").and_then(|x| x.as_str()).unwrap_or("");
            let _product = v.get("product").and_then(|x| x.as_str()).unwrap_or("");
            let summary = v.get("shortDescription")
                .and_then(|x| x.as_str())
                .map(|s| sanitize(s, 400));

            Some(Item {
                id: format!("https://nvd.nist.gov/vuln/detail/{}", cve_id),
                title: format!("{}: {} ({})", cve_id, name, vendor),
                tags: vec!["security".to_string(), "cve".to_string(), "kev".to_string()],
                source: "cisa-kev".to_string(),
                summary,
                bypass_tag_filter: true,
                metadata: HashMap::new(),
            })
        })
        .collect()
}

pub async fn fetch_nvd_cve(client: &reqwest::Client) -> Vec<Item> {
    let url = "https://services.nvd.nist.gov/rest/json/cves/2.0?cvssV3Severity=HIGH&resultsPerPage=20";
    let data: serde_json::Value = match client.get(url).send().await {
        Ok(resp) => match resp.json().await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[api] nvd cve json error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[api] nvd cve fetch error: {}", e);
            return vec![];
        }
    };

    let Some(vulns) = data.get("vulnerabilities").and_then(|v| v.as_array()) else {
        eprintln!("[api] nvd cve: no vulnerabilities array");
        return vec![];
    };

    vulns.iter()
        .filter_map(|v| {
            let cve = v.get("cve")?;
            let id = cve.get("id")?.as_str()?.to_string();
            let descriptions = cve.get("descriptions")?.as_array()?;
            let desc = descriptions.iter()
                .find(|d| d.get("lang").and_then(|l| l.as_str()) == Some("en"))
                .and_then(|d| d.get("value")?.as_str())
                .unwrap_or("No description");
            let summary = Some(sanitize(desc, 400));

            let metrics = cve.get("metrics")?;
            let cvss_data = metrics.get("cvssMetricV31")
                .or_else(|| metrics.get("cvssMetricV30"))
                .and_then(|m| m.as_array())
                .and_then(|a| a.first())?;
            let score = cvss_data.get("cvssData")
                .and_then(|d| d.get("baseScore"))
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0);

            Some(Item {
                id: format!("https://nvd.nist.gov/vuln/detail/{}", id),
                title: format!("{} (CVSS {:.1})", id, score),
                tags: vec!["security".to_string(), "cve".to_string()],
                source: "nvd".to_string(),
                summary,
                bypass_tag_filter: true,
                metadata: HashMap::new(),
            })
        })
        .collect()
}

pub async fn fetch_papers_with_code(client: &reqwest::Client) -> Vec<Item> {
    let url = "https://huggingface.co/api/daily_papers?limit=20";
    let data: serde_json::Value = match client.get(url).send().await {
        Ok(resp) => match resp.json().await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[api] daily papers json error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[api] daily papers fetch error: {}", e);
            return vec![];
        }
    };

    let Some(papers) = data.as_array() else {
        eprintln!("[api] daily papers: response is not an array");
        return vec![];
    };

    papers.iter()
        .filter_map(|entry| {
            let paper = entry.get("paper")?;
            let id = paper.get("id")?.as_str()?.to_string();
            let title = paper.get("title")?.as_str()?.to_string();
            let summary = paper.get("summary").and_then(|v| v.as_str()).map(String::from);

            Some(Item {
                id: format!("https://huggingface.co/papers/{}", id),
                title,
                tags: vec!["ai".to_string(), "ml".to_string(), "research".to_string()],
                source: "huggingface-papers".to_string(),
                summary,
                bypass_tag_filter: false,
                metadata: HashMap::new(),
            })
        })
        .collect()
}

pub async fn fetch_huggingface_trending(client: &reqwest::Client) -> Vec<Item> {
    let url = "https://huggingface.co/api/models?sort=trendingScore&direction=-1&limit=20";
    let data: serde_json::Value = match client.get(url).send().await {
        Ok(resp) => match resp.json().await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[api] huggingface json error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[api] huggingface fetch error: {}", e);
            return vec![];
        }
    };

    let Some(models) = data.as_array() else {
        eprintln!("[api] huggingface: response is not an array");
        return vec![];
    };

    models.iter()
        .filter_map(|m| {
            let id = m.get("id")?.as_str()?.to_string();
            let tags = m.get("tags")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let pipeline = m.get("pipeline_tag")
                .and_then(|p| p.as_str())
                .unwrap_or("unknown");
            let likes = m.get("likes").and_then(|v| v.as_u64()).unwrap_or(0);
            let downloads = m.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);
            let author = id.split('/').next().unwrap_or("");

            let summary = format!(
                "Pipeline: {} | Author: {} | Likes: {} | Downloads: {}",
                pipeline, author, likes, downloads
            );

            Some(Item {
                id: format!("https://huggingface.co/{}", id),
                title: id.clone(),
                tags,
                source: "huggingface".to_string(),
                summary: Some(summary),
                bypass_tag_filter: true,
                metadata: HashMap::new(),
            })
        })
        .collect()
}

pub async fn fetch_hacker_news_api(client: &reqwest::Client) -> Vec<Item> {
    let top_stories: Vec<u64> = match client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await
    {
        Ok(resp) => match resp.json().await {
            Ok(ids) => ids,
            Err(e) => {
                eprintln!("[api] hacker news api json error: {}", e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("[api] hacker news api fetch error: {}", e);
            return vec![];
        }
    };

    let mut items = Vec::new();
    for story_id in top_stories.iter().take(20) {
        let url = format!(
            "https://hacker-news.firebaseio.com/v0/item/{}.json",
            story_id
        );
        let story: serde_json::Value = match client.get(&url).send().await {
            Ok(resp) => match resp.json().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[api] hn story {} json error: {}", story_id, e);
                    continue;
                }
            },
            Err(e) => {
                eprintln!("[api] hn story {} fetch error: {}", story_id, e);
                continue;
            }
        };

        let title = match story.get("title").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => continue,
        };
        let story_url = story
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", story_id));

        items.push(Item {
            id: story_url,
            title,
            tags: vec!["tech".to_string()],
            source: "hacker-news-api".to_string(),
            summary: None,
            bypass_tag_filter: true,
            metadata: HashMap::new(),
        });
    }
    items
}
