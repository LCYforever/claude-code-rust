//! History Snip Manager - Intelligent conversation history compression
//!
//! Provides 4 compression strategies to keep conversation history within token limits:
//! - KeepRecent: Simply keeps the N most recent messages
//! - PriorityBased: Keeps messages by priority score (system > recent > assistant > old user)
//! - Smart: Groups messages into segments, compresses old segments into summaries
//! - Hybrid: Combines PriorityBased + Smart for best results

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::context::ContextEntry;

/// Compression strategy for history snipping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnipStrategy {
    /// Keep only the N most recent messages (simple FIFO)
    KeepRecent,
    /// Keep messages based on priority scoring
    PriorityBased,
    /// Intelligently group and compress old message segments into summaries
    Smart,
    /// Hybrid: PriorityBased + Smart combined
    Hybrid,
}

impl Default for SnipStrategy {
    fn default() -> Self {
        Self::Hybrid
    }
}

/// Configuration for HistorySnipManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnipConfig {
    /// Maximum tokens allowed in the history window
    pub max_tokens: usize,
    /// Strategy to use for compression
    pub strategy: SnipStrategy,
    /// Number of recent messages to always keep (for KeepRecent / Smart / Hybrid)
    pub keep_recent_count: usize,
    /// Number of messages per segment for Smart compression
    pub segment_size: usize,
    /// Token budget reserved for system prompt
    pub system_prompt_reserve: usize,
    /// Whether snipping is enabled
    pub enabled: bool,
}

impl Default for HistorySnipConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,
            strategy: SnipStrategy::Hybrid,
            keep_recent_count: 10,
            segment_size: 6,
            system_prompt_reserve: 4000,
            enabled: true,
        }
    }
}

/// A compressed summary representing a group of snipped messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippedSegment {
    pub id: String,
    pub summary: String,
    pub original_count: usize,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
    pub created_at: DateTime<Utc>,
}

/// Statistics about snipping operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipStats {
    pub total_snips: usize,
    pub total_messages_compressed: usize,
    pub total_tokens_saved: usize,
    pub segments_created: usize,
    pub current_strategy: SnipStrategy,
    pub is_enabled: bool,
}

/// History Snip Manager - manages conversation history compression
pub struct HistorySnipManager {
    config: Arc<RwLock<HistorySnipConfig>>,
    segments: Arc<RwLock<Vec<SnippedSegment>>>,
    stats: Arc<RwLock<SnipStats>>,
}

impl HistorySnipManager {
    pub fn new(config: HistorySnipConfig) -> Self {
        let strategy = config.strategy.clone();
        let enabled = config.enabled;
        Self {
            config: Arc::new(RwLock::new(config)),
            segments: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SnipStats {
                total_snips: 0,
                total_messages_compressed: 0,
                total_tokens_saved: 0,
                segments_created: 0,
                current_strategy: strategy,
                is_enabled: enabled,
            })),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(HistorySnipConfig::default())
    }

    /// Check if snipping is needed and apply the configured strategy.
    /// Returns a new list of ContextEntry that fits within the token budget.
    pub async fn snip_if_needed(&self, entries: &[ContextEntry]) -> Vec<ContextEntry> {
        let config = self.config.read().await;
        if !config.enabled {
            return entries.to_vec();
        }

        let total_tokens: usize = entries.iter().map(|e| e.token_count).sum();
        let budget = config.max_tokens.saturating_sub(config.system_prompt_reserve);

        if total_tokens <= budget {
            return entries.to_vec();
        }

        println!(
            "✂️  HistorySnip: {} tokens exceeds budget {} (strategy: {:?})",
            total_tokens, budget, config.strategy
        );

        let result = match config.strategy {
            SnipStrategy::KeepRecent => {
                self.strategy_keep_recent(entries, budget, config.keep_recent_count).await
            }
            SnipStrategy::PriorityBased => {
                self.strategy_priority_based(entries, budget).await
            }
            SnipStrategy::Smart => {
                self.strategy_smart(entries, budget, config.keep_recent_count, config.segment_size).await
            }
            SnipStrategy::Hybrid => {
                self.strategy_hybrid(entries, budget, config.keep_recent_count, config.segment_size).await
            }
        };

        // Update stats
        let saved_tokens = total_tokens.saturating_sub(result.iter().map(|e| e.token_count).sum::<usize>());
        let compressed_count = entries.len().saturating_sub(result.len());
        {
            let mut stats = self.stats.write().await;
            stats.total_snips += 1;
            stats.total_messages_compressed += compressed_count;
            stats.total_tokens_saved += saved_tokens;
        }

        println!(
            "✂️  HistorySnip: {} entries -> {} entries, saved {} tokens",
            entries.len(), result.len(), saved_tokens
        );

        result
    }

    /// Strategy 1: KeepRecent - simply keep the last N messages
    async fn strategy_keep_recent(
        &self,
        entries: &[ContextEntry],
        budget: usize,
        keep_count: usize,
    ) -> Vec<ContextEntry> {
        let mut result: Vec<ContextEntry> = Vec::new();
        let mut token_sum = 0usize;

        // Take from the end (most recent) up to keep_count or budget
        for entry in entries.iter().rev().take(keep_count) {
            if token_sum + entry.token_count > budget {
                break;
            }
            token_sum += entry.token_count;
            result.push(entry.clone());
        }
        result.reverse();
        result
    }

    /// Strategy 2: PriorityBased - score each message and keep highest scoring ones
    async fn strategy_priority_based(
        &self,
        entries: &[ContextEntry],
        budget: usize,
    ) -> Vec<ContextEntry> {
        let total = entries.len();
        let mut scored: Vec<(usize, f64, &ContextEntry)> = entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let score = self.calculate_priority_score(entry, i, total);
                (i, score, entry)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Greedily pick highest-scored entries within budget
        let mut selected_indices: Vec<usize> = Vec::new();
        let mut token_sum = 0usize;
        for (idx, _score, entry) in &scored {
            if token_sum + entry.token_count > budget {
                continue;
            }
            token_sum += entry.token_count;
            selected_indices.push(*idx);
        }

        // Sort by original order to maintain conversation flow
        selected_indices.sort();
        selected_indices.iter().map(|&i| entries[i].clone()).collect()
    }

    /// Calculate priority score for a message (higher = more important to keep)
    fn calculate_priority_score(&self, entry: &ContextEntry, index: usize, total: usize) -> f64 {
        let mut score = 0.0;

        // Role weight: system > user > assistant
        match entry.role.as_str() {
            "system" => score += 100.0,
            "user" => score += 3.0,
            "assistant" => score += 2.0,
            _ => score += 1.0,
        }

        // Recency: newer messages get higher score
        // Linear decay: most recent = 10.0, oldest = 0.0
        if total > 1 {
            score += 10.0 * (index as f64) / ((total - 1) as f64);
        } else {
            score += 10.0;
        }

        // Short messages (likely questions/commands) get a slight boost
        if entry.token_count < 50 {
            score += 1.0;
        }

        // Penalize very long assistant responses (they consume lots of tokens)
        if entry.role == "assistant" && entry.token_count > 500 {
            score -= 2.0;
        }

        score
    }

    /// Strategy 3: Smart - group old messages into segments and create summaries
    async fn strategy_smart(
        &self,
        entries: &[ContextEntry],
        budget: usize,
        keep_recent: usize,
        segment_size: usize,
    ) -> Vec<ContextEntry> {
        let total = entries.len();
        if total <= keep_recent {
            return entries.to_vec();
        }

        let split_point = total.saturating_sub(keep_recent);
        let old_entries = &entries[..split_point];
        let recent_entries = &entries[split_point..];

        // Calculate token budget for recent messages
        let recent_tokens: usize = recent_entries.iter().map(|e| e.token_count).sum();
        let summary_budget = budget.saturating_sub(recent_tokens);

        // Group old entries into segments and compress each
        let mut compressed_entries: Vec<ContextEntry> = Vec::new();
        let mut compressed_tokens = 0usize;

        for chunk in old_entries.chunks(segment_size) {
            let refs: Vec<&ContextEntry> = chunk.iter().collect();
            let summary = self.create_segment_summary(&refs).await;
            let summary_entry = ContextEntry::new("system", &summary);

            if compressed_tokens + summary_entry.token_count > summary_budget {
                break; // No more budget for summaries
            }
            compressed_tokens += summary_entry.token_count;

            // Track the segment
            let segment = SnippedSegment {
                id: uuid::Uuid::new_v4().to_string(),
                summary: summary.clone(),
                original_count: chunk.len(),
                original_tokens: chunk.iter().map(|e| e.token_count).sum(),
                compressed_tokens: summary_entry.token_count,
                time_range: (
                    chunk.first().map(|e| e.timestamp).unwrap_or_else(Utc::now),
                    chunk.last().map(|e| e.timestamp).unwrap_or_else(Utc::now),
                ),
                created_at: Utc::now(),
            };

            {
                let mut segments = self.segments.write().await;
                segments.push(segment);
                let mut stats = self.stats.write().await;
                stats.segments_created += 1;
            }

            compressed_entries.push(summary_entry);
        }

        // Combine: compressed summaries + recent messages
        compressed_entries.extend(recent_entries.iter().cloned());
        compressed_entries
    }

    /// Strategy 4: Hybrid - PriorityBased first pass, then Smart compression on remainder
    async fn strategy_hybrid(
        &self,
        entries: &[ContextEntry],
        budget: usize,
        keep_recent: usize,
        segment_size: usize,
    ) -> Vec<ContextEntry> {
        let total = entries.len();
        if total <= keep_recent {
            return entries.to_vec();
        }

        let split_point = total.saturating_sub(keep_recent);
        let old_entries = &entries[..split_point];
        let recent_entries = &entries[split_point..];

        let recent_tokens: usize = recent_entries.iter().map(|e| e.token_count).sum();
        let old_budget = budget.saturating_sub(recent_tokens);

        // Phase 1: Priority-score old entries to decide what to keep vs compress
        let old_total = old_entries.len();
        let mut scored: Vec<(usize, f64, &ContextEntry)> = old_entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let score = self.calculate_priority_score(entry, i, old_total);
                (i, score, entry)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Split: keep high-priority individually, compress the rest
        let mut kept_entries: Vec<(usize, ContextEntry)> = Vec::new();
        let mut to_compress: Vec<&ContextEntry> = Vec::new();
        let mut kept_tokens = 0usize;
        let half_budget = old_budget / 2; // Reserve half for kept, half for summaries

        for (idx, _score, entry) in &scored {
            if kept_tokens + entry.token_count <= half_budget {
                kept_tokens += entry.token_count;
                kept_entries.push((*idx, (*entry).clone()));
            } else {
                to_compress.push(entry);
            }
        }

        // Phase 2: Compress low-priority entries into summaries
        let summary_budget = old_budget.saturating_sub(kept_tokens);
        let mut summary_entries: Vec<ContextEntry> = Vec::new();
        let mut summary_tokens = 0usize;

        for chunk in to_compress.chunks(segment_size) {
            let summary = self.create_segment_summary(chunk).await;
            let summary_entry = ContextEntry::new("system", &summary);
            if summary_tokens + summary_entry.token_count > summary_budget {
                break;
            }
            summary_tokens += summary_entry.token_count;
            summary_entries.push(summary_entry);
        }

        // Combine: summaries first (oldest context), then kept entries in order, then recent
        kept_entries.sort_by_key(|(idx, _)| *idx);
        let mut result: Vec<ContextEntry> = summary_entries;
        result.extend(kept_entries.into_iter().map(|(_, e)| e));
        result.extend(recent_entries.iter().cloned());
        result
    }

    /// Create a compressed summary for a segment of messages.
    /// This is a local heuristic summary (no LLM call needed).
    async fn create_segment_summary(&self, entries: &[&ContextEntry]) -> String {
        let mut user_topics: Vec<String> = Vec::new();
        let mut assistant_points: Vec<String> = Vec::new();

        for entry in entries {
            let preview: String = entry.content.chars().take(120).collect();
            match entry.role.as_str() {
                "user" => user_topics.push(format!("• 用户: {}", preview)),
                "assistant" => {
                    // Extract first sentence or first 80 chars
                    let first_sentence = entry.content
                        .split(&['.', '。', '！', '!', '\n'][..])
                        .next()
                        .unwrap_or(&preview);
                    let short: String = first_sentence.chars().take(80).collect();
                    assistant_points.push(format!("• 助手: {}", short));
                }
                _ => {}
            }
        }

        let mut summary = String::from("[历史对话摘要]\n");
        if !user_topics.is_empty() {
            summary.push_str(&user_topics.join("\n"));
            summary.push('\n');
        }
        if !assistant_points.is_empty() {
            summary.push_str(&assistant_points.join("\n"));
        }
        summary
    }

    /// Create a compressed summary (overload for owned ContextEntry slices)
    async fn _create_segment_summary_owned(&self, entries: &[ContextEntry]) -> String {
        let refs: Vec<&ContextEntry> = entries.iter().collect();
        self.create_segment_summary(&refs).await
    }

    /// Get current snip statistics
    pub async fn stats(&self) -> SnipStats {
        self.stats.read().await.clone()
    }

    /// Get all snipped segments (compressed history)
    pub async fn get_segments(&self) -> Vec<SnippedSegment> {
        self.segments.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: HistorySnipConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Enable or disable snipping
    pub async fn set_enabled(&self, enabled: bool) {
        let mut cfg = self.config.write().await;
        cfg.enabled = enabled;
        let mut stats = self.stats.write().await;
        stats.is_enabled = enabled;
    }

    /// Change strategy
    pub async fn set_strategy(&self, strategy: SnipStrategy) {
        let mut cfg = self.config.write().await;
        cfg.strategy = strategy.clone();
        let mut stats = self.stats.write().await;
        stats.current_strategy = strategy;
    }

    /// Clear all segments (reset compression state)
    pub async fn clear(&self) {
        let mut segments = self.segments.write().await;
        segments.clear();
    }
}

impl Default for HistorySnipManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}
