//! RAG — поиск контекста в Arch Wiki

use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const ARCH_WIKI_API: &str = "https://wiki.archlinux.org/api.php";

#[derive(Deserialize)]
struct SearchResponse {
    query: SearchQuery,
}

#[derive(Deserialize)]
struct SearchQuery {
    search: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    title: String,
}

#[derive(Deserialize)]
struct ExtractResponse {
    query: ExtractQuery,
}

#[derive(Deserialize)]
struct ExtractQuery {
    pages: std::collections::HashMap<String, PageContent>,
}

#[derive(Deserialize)]
struct PageContent {
    extract: Option<String>,
}

/// Получает релевантный контекст из Arch Wiki
pub struct ArchWikiRag {
    client: Client,
}

impl ArchWikiRag {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Ищет статью и возвращает контекст для подстановки в промпт
    pub async fn fetch_context(&self, query: &str) -> Option<String> {
        let title = self.search(query).await?;
        let extract = self.fetch_extract(&title).await?;
        if extract.trim().is_empty() {
            return None;
        }
        Some(format!("[Arch Wiki: {}]\n{}", title, extract))
    }

    async fn search(&self, query: &str) -> Option<String> {
        let resp = self
            .client
            .get(ARCH_WIKI_API)
            .query(&[
                ("action", "query"),
                ("list", "search"),
                ("srsearch", query),
                ("srnamespace", "0"),
                ("format", "json"),
                ("srlimit", "1"),
            ])
            .send()
            .await
            .ok()?;

        let data: SearchResponse = resp.json().await.ok()?;
        data.query.search.into_iter().next().map(|r| r.title)
    }

    async fn fetch_extract(&self, title: &str) -> Option<String> {
        let resp = self
            .client
            .get(ARCH_WIKI_API)
            .query(&[
                ("action", "query"),
                ("titles", title),
                ("prop", "extracts"),
                ("explaintext", "true"),
                ("exsectionformat", "plain"),
                ("exchars", "3000"),
                ("format", "json"),
            ])
            .send()
            .await
            .ok()?;

        let data: ExtractResponse = resp.json().await.ok()?;
        data.query.pages.into_values().next()?.extract
    }
}

impl Default for ArchWikiRag {
    fn default() -> Self {
        Self::new()
    }
}
