//! Web Search Service
//!
//! Provides web search capability for agents that need internet access.
//! Uses a two-step approach:
//! 1. LLM determines if search is needed and generates search queries
//! 2. Backend executes the search via HTTP and returns results
//!
//! Supports multiple search backends:
//! - DuckDuckGo Instant Answer API (default, no API key needed)
//! - Custom search API endpoint (configurable)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A single search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// The response from a web search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub source: String,
}

/// Search intent analysis result from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIntent {
    pub needs_search: bool,
    pub queries: Vec<String>,
    pub reason: String,
}

/// Web Search Service
pub struct WebSearchService {
    http_client: Client,
}

impl WebSearchService {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self { http_client }
    }

    /// Build the prompt for the LLM to determine if web search is needed
    pub fn build_search_intent_prompt() -> String {
        r#"你是一个搜索意图分析引擎。分析用户的问题，判断是否需要联网搜索来获取最新信息。

## 需要搜索的情况
- 询问最新新闻、时事、近期事件
- 询问实时数据（天气、股票、汇率等）
- 询问特定产品、服务的最新价格或状态
- 询问最新技术文档、API 版本
- 询问某人/某事的最新动态
- 需要验证时效性信息的准确性
- 询问不确定或你不了解的事实性问题

## 不需要搜索的情况
- 一般性知识问答（数学、历史常识、编程语法等）
- 创意写作（写诗、写故事、翻译等）
- 日常闲聊、打招呼
- 代码编写、调试
- 分析用户提供的内容
- 概念解释、教学

## 输出格式（严格 JSON，不要输出其他内容）

{"needs_search": true/false, "queries": ["搜索词1", "搜索词2"], "reason": "简短理由"}

注意：
- queries 数组在 needs_search=false 时为空数组 []
- queries 中的搜索词应该是简洁有效的关键词，适合搜索引擎
- 最多生成 3 个搜索词
- 搜索词优先使用中文，除非原文是英文问题"#.to_string()
    }

    /// Parse the LLM's search intent response
    pub fn parse_search_intent(response_text: &str) -> Option<SearchIntent> {
        let text = response_text.trim();

        // Try full text as JSON
        if let Ok(intent) = serde_json::from_str::<SearchIntent>(text) {
            return Some(intent);
        }

        // Try to extract JSON from text
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                let json_str = &text[start..=end];
                if let Ok(intent) = serde_json::from_str::<SearchIntent>(json_str) {
                    return Some(intent);
                }
            }
        }

        None
    }

    /// Execute a web search using DuckDuckGo Instant Answer API
    pub async fn search_duckduckgo(&self, query: &str) -> anyhow::Result<WebSearchResponse> {
        println!("🔍 Web search (DuckDuckGo): {}", query);

        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let response = self.http_client
            .get(&url)
            .header("User-Agent", "ClaudeCodeAgent/1.0")
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;

        let mut results = Vec::new();

        // Parse Abstract (main result)
        if let Some(abstract_text) = body.get("Abstract").and_then(|a| a.as_str()) {
            if !abstract_text.is_empty() {
                let abstract_url = body.get("AbstractURL")
                    .and_then(|u| u.as_str())
                    .unwrap_or("");
                let abstract_source = body.get("AbstractSource")
                    .and_then(|s| s.as_str())
                    .unwrap_or("DuckDuckGo");
                results.push(SearchResult {
                    title: format!("{} - {}", query, abstract_source),
                    url: abstract_url.to_string(),
                    snippet: abstract_text.to_string(),
                });
            }
        }

        // Parse Related Topics
        if let Some(topics) = body.get("RelatedTopics").and_then(|t| t.as_array()) {
            for topic in topics.iter().take(5) {
                if let (Some(text), Some(url)) = (
                    topic.get("Text").and_then(|t| t.as_str()),
                    topic.get("FirstURL").and_then(|u| u.as_str()),
                ) {
                    if !text.is_empty() {
                        results.push(SearchResult {
                            title: text.chars().take(80).collect::<String>(),
                            url: url.to_string(),
                            snippet: text.to_string(),
                        });
                    }
                }
            }
        }

        // Parse Infobox if available
        if let Some(infobox) = body.get("Infobox").and_then(|i| i.get("content")).and_then(|c| c.as_array()) {
            for item in infobox.iter().take(5) {
                if let (Some(label), Some(value)) = (
                    item.get("label").and_then(|l| l.as_str()),
                    item.get("value").and_then(|v| v.as_str()),
                ) {
                    results.push(SearchResult {
                        title: label.to_string(),
                        url: String::new(),
                        snippet: value.to_string(),
                    });
                }
            }
        }

        Ok(WebSearchResponse {
            query: query.to_string(),
            results,
            source: "DuckDuckGo".to_string(),
        })
    }

    /// Execute a web search using a general-purpose search scraper
    /// Falls back to a simpler approach if DuckDuckGo doesn't return useful results
    pub async fn search_fallback(&self, query: &str) -> anyhow::Result<WebSearchResponse> {
        println!("🔍 Web search (fallback scraper): {}", query);

        // Use Bing search (no API key needed, scrape-friendly)
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        let response = self.http_client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?;

        let body = response.text().await?;
        let mut results = Vec::new();

        // Simple HTML parsing for DuckDuckGo HTML results
        // Look for result snippets between known markers
        for segment in body.split("class=\"result__snippet\"") {
            if results.len() >= 5 {
                break;
            }
            if let Some(text_start) = segment.find('>') {
                let text_part = &segment[text_start + 1..];
                if let Some(text_end) = text_part.find("</") {
                    let snippet = text_part[..text_end]
                        .replace("<b>", "")
                        .replace("</b>", "")
                        .replace("&amp;", "&")
                        .replace("&lt;", "<")
                        .replace("&gt;", ">")
                        .replace("&quot;", "\"")
                        .trim()
                        .to_string();

                    if !snippet.is_empty() && snippet.len() > 20 {
                        results.push(SearchResult {
                            title: snippet.chars().take(60).collect::<String>(),
                            url: String::new(),
                            snippet,
                        });
                    }
                }
            }
        }

        // Also try to extract URLs from result titles
        for (i, segment) in body.split("class=\"result__a\"").enumerate() {
            if i == 0 || i > results.len() {
                continue;
            }
            if let Some(href_start) = segment.find("href=\"") {
                let url_part = &segment[href_start + 6..];
                if let Some(href_end) = url_part.find('"') {
                    let url = url_part[..href_end].to_string();
                    if let Some(result) = results.get_mut(i - 1) {
                        result.url = url;
                    }
                }
            }
            if let Some(text_start) = segment.find('>') {
                let text_part = &segment[text_start + 1..];
                if let Some(text_end) = text_part.find("</") {
                    let title = text_part[..text_end]
                        .replace("<b>", "")
                        .replace("</b>", "")
                        .trim()
                        .to_string();
                    if let Some(result) = results.get_mut(i - 1) {
                        if !title.is_empty() {
                            result.title = title;
                        }
                    }
                }
            }
        }

        Ok(WebSearchResponse {
            query: query.to_string(),
            results,
            source: "DuckDuckGo HTML".to_string(),
        })
    }

    /// Execute search: tries API first, falls back to HTML scraping
    pub async fn search(&self, query: &str) -> WebSearchResponse {
        // Try DuckDuckGo Instant Answer API first
        match self.search_duckduckgo(query).await {
            Ok(response) if !response.results.is_empty() => {
                println!("✅ Got {} results from DuckDuckGo API", response.results.len());
                return response;
            }
            Ok(_) => {
                println!("⚠️ DuckDuckGo API returned no results, trying fallback...");
            }
            Err(e) => {
                println!("⚠️ DuckDuckGo API error: {}, trying fallback...", e);
            }
        }

        // Fallback to HTML scraping
        match self.search_fallback(query).await {
            Ok(response) => {
                println!("✅ Got {} results from fallback scraper", response.results.len());
                response
            }
            Err(e) => {
                println!("❌ Fallback search also failed: {}", e);
                WebSearchResponse {
                    query: query.to_string(),
                    results: vec![],
                    source: "error".to_string(),
                }
            }
        }
    }

    /// Execute multiple searches and combine results
    pub async fn multi_search(&self, queries: &[String]) -> Vec<WebSearchResponse> {
        let mut all_results = Vec::new();
        for query in queries {
            let result = self.search(query).await;
            all_results.push(result);
        }
        all_results
    }

    /// Format search results as context for LLM
    pub fn format_search_results_as_context(results: &[WebSearchResponse]) -> String {
        if results.is_empty() {
            return "（未找到相关搜索结果）".to_string();
        }

        let mut context = String::from("## 联网搜索结果\n\n");

        for response in results {
            if response.results.is_empty() {
                context.push_str(&format!("### 搜索: {}\n未找到相关结果。\n\n", response.query));
                continue;
            }

            context.push_str(&format!("### 搜索: {} (来源: {})\n\n", response.query, response.source));

            for (i, result) in response.results.iter().enumerate() {
                context.push_str(&format!("{}. **{}**\n", i + 1, result.title));
                if !result.url.is_empty() {
                    context.push_str(&format!("   链接: {}\n", result.url));
                }
                context.push_str(&format!("   {}\n\n", result.snippet));
            }
        }

        context.push_str("\n---\n请基于以上搜索结果回答用户的问题。如果搜索结果不够充分，可以结合你已有的知识进行补充，但请标明哪些是搜索结果提供的信息。\n");
        context
    }
}

/// URL encoding helper (simple percent-encoding)
mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut encoded = String::new();
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                b' ' => encoded.push('+'),
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        encoded
    }
}
