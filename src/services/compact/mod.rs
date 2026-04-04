//! Compact Service - Three-tier conversation compression
//!
//! Provides intelligent multi-level compression for conversation histories:
//!
//! **Level 1: Microcompact** — Lightweight per-message trimming
//! - Strips redundant whitespace, code block deduplication
//! - Truncates very long assistant responses to key points
//! - Minimal information loss, always applied first
//!
//! **Level 2: Session Compact** — Session-level summarization
//! - Groups messages into batches and produces summaries
//! - Keeps the N most recent messages intact
//! - Medium compression ratio (~3:1)
//!
//! **Level 3: Memory Compact** — Cross-session memory extraction
//! - Extracts key facts, decisions, and user preferences
//! - Produces a compact "memory block" for long-term context
//! - Highest compression, used when session history is very large

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::memory::context::ContextEntry;

// ===== Configuration =====

/// Configuration for the three-tier compact service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactConfig {
    /// Enable Level 1 (microcompact)
    pub enable_microcompact: bool,
    /// Enable Level 2 (session compact)
    pub enable_session_compact: bool,
    /// Enable Level 3 (memory compact)
    pub enable_memory_compact: bool,
    /// Max characters per assistant message before truncation (Level 1)
    pub micro_max_assistant_chars: usize,
    /// Max characters per code block before truncation (Level 1)
    pub micro_max_code_block_chars: usize,
    /// Number of recent messages to keep intact (Level 2)
    pub session_keep_recent: usize,
    /// Batch size for session summarization (Level 2)
    pub session_batch_size: usize,
    /// Keywords that indicate important memory content (Level 3)
    pub memory_keywords: Vec<String>,
    /// Token threshold to trigger Level 2 compression
    pub session_compact_threshold: usize,
    /// Token threshold to trigger Level 3 compression
    pub memory_compact_threshold: usize,
}

impl Default for CompactConfig {
    fn default() -> Self {
        Self {
            enable_microcompact: true,
            enable_session_compact: true,
            enable_memory_compact: true,
            micro_max_assistant_chars: 2000,
            micro_max_code_block_chars: 800,
            session_keep_recent: 10,
            session_batch_size: 6,
            memory_keywords: vec![
                "important".to_string(),
                "key".to_string(),
                "remember".to_string(),
                "注意".to_string(),
                "重要".to_string(),
                "记住".to_string(),
                "关键".to_string(),
                "决定".to_string(),
                "结论".to_string(),
                "TODO".to_string(),
            ],
            session_compact_threshold: 50_000,
            memory_compact_threshold: 80_000,
        }
    }
}

// ===== Compact Result =====

/// Result of a compact operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResult {
    pub level_applied: CompactLevel,
    pub entries_before: usize,
    pub entries_after: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub compression_ratio: f64,
    pub memories_extracted: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompactLevel {
    None,
    Micro,
    Session,
    Memory,
}

// ===== Compact Service =====

/// Three-tier compact service for conversation compression
pub struct CompactService {
    config: CompactConfig,
}

impl CompactService {
    pub fn new(config: CompactConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(CompactConfig::default())
    }

    /// Run the full compact pipeline on a list of entries.
    /// Returns the compacted entries and a result summary.
    pub fn compact(&self, entries: &[ContextEntry]) -> (Vec<ContextEntry>, CompactResult) {
        let tokens_before: usize = entries.iter().map(|e| e.token_count).sum();
        let entries_before = entries.len();
        let mut current = entries.to_vec();
        let mut level = CompactLevel::None;
        let mut memories_extracted = 0;

        // Level 1: Microcompact (always runs if enabled)
        if self.config.enable_microcompact {
            current = self.microcompact(&current);
            level = CompactLevel::Micro;
            let micro_tokens: usize = current.iter().map(|e| e.token_count).sum();
            println!(
                "🔬 Microcompact: {} -> {} tokens ({} entries)",
                tokens_before, micro_tokens, current.len()
            );
        }

        // Level 2: Session Compact (if total tokens exceed threshold)
        let current_tokens: usize = current.iter().map(|e| e.token_count).sum();
        if self.config.enable_session_compact && current_tokens > self.config.session_compact_threshold {
            current = self.session_compact(&current);
            level = CompactLevel::Session;
            let session_tokens: usize = current.iter().map(|e| e.token_count).sum();
            println!(
                "📦 SessionCompact: {} -> {} tokens ({} entries)",
                current_tokens, session_tokens, current.len()
            );
        }

        // Level 3: Memory Compact (if still exceeding memory threshold)
        let current_tokens: usize = current.iter().map(|e| e.token_count).sum();
        if self.config.enable_memory_compact && current_tokens > self.config.memory_compact_threshold {
            let (compacted, extracted) = self.memory_compact(&current);
            current = compacted;
            memories_extracted = extracted;
            level = CompactLevel::Memory;
            let memory_tokens: usize = current.iter().map(|e| e.token_count).sum();
            println!(
                "🧠 MemoryCompact: {} -> {} tokens, {} memories extracted",
                current_tokens, memory_tokens, memories_extracted
            );
        }

        let tokens_after: usize = current.iter().map(|e| e.token_count).sum();
        let compression_ratio = if tokens_before > 0 {
            tokens_after as f64 / tokens_before as f64
        } else {
            1.0
        };

        let result = CompactResult {
            level_applied: level,
            entries_before,
            entries_after: current.len(),
            tokens_before,
            tokens_after,
            compression_ratio,
            memories_extracted,
        };

        (current, result)
    }

    // ===== Level 1: Microcompact =====

    /// Per-message lightweight compression:
    /// - Collapse excessive whitespace
    /// - Truncate very long assistant messages
    /// - Truncate large code blocks within messages
    fn microcompact(&self, entries: &[ContextEntry]) -> Vec<ContextEntry> {
        entries
            .iter()
            .map(|entry| {
                let content = self.micro_compress_content(&entry.role, &entry.content);
                if content.len() == entry.content.len() {
                    entry.clone()
                } else {
                    let mut new_entry = ContextEntry::new(&entry.role, &content);
                    new_entry.priority = entry.priority.clone();
                    new_entry.source = entry.source.clone();
                    new_entry.timestamp = entry.timestamp;
                    new_entry.id = entry.id.clone();
                    new_entry
                }
            })
            .collect()
    }

    fn micro_compress_content(&self, role: &str, content: &str) -> String {
        let mut result = content.to_string();

        // Collapse multiple blank lines into one
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }

        // Collapse multiple spaces
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }

        // Truncate long assistant messages
        if role == "assistant" && result.len() > self.config.micro_max_assistant_chars {
            result = self.truncate_assistant_message(&result);
        }

        // Truncate large code blocks
        result = self.truncate_code_blocks(&result);

        result
    }

    /// Truncate a long assistant message, preserving the beginning and end
    fn truncate_assistant_message(&self, content: &str) -> String {
        let max = self.config.micro_max_assistant_chars;
        let keep_start = max * 3 / 5; // 60% from beginning
        let keep_end = max * 2 / 5;   // 40% from end

        let chars: Vec<char> = content.chars().collect();
        if chars.len() <= max {
            return content.to_string();
        }

        let start: String = chars[..keep_start].iter().collect();
        let end: String = chars[chars.len() - keep_end..].iter().collect();
        let omitted = chars.len() - keep_start - keep_end;

        format!("{}\n\n... [省略 {} 字符] ...\n\n{}", start, omitted, end)
    }

    /// Find and truncate large code blocks (```...```)
    fn truncate_code_blocks(&self, content: &str) -> String {
        let max_code = self.config.micro_max_code_block_chars;
        let mut result = String::new();
        let mut in_code_block = false;
        let mut code_block_content = String::new();
        let mut code_block_header = String::new();

        for line in content.lines() {
            if line.starts_with("```") && !in_code_block {
                in_code_block = true;
                code_block_header = line.to_string();
                code_block_content.clear();
            } else if line.starts_with("```") && in_code_block {
                in_code_block = false;
                // Emit the (possibly truncated) code block
                result.push_str(&code_block_header);
                result.push('\n');
                if code_block_content.len() > max_code {
                    let truncated: String = code_block_content.chars().take(max_code).collect();
                    result.push_str(&truncated);
                    result.push_str(&format!(
                        "\n// ... [代码块已截断，省略 {} 字符] ...\n",
                        code_block_content.len() - max_code
                    ));
                } else {
                    result.push_str(&code_block_content);
                }
                result.push_str("```\n");
            } else if in_code_block {
                code_block_content.push_str(line);
                code_block_content.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        // Handle unclosed code block
        if in_code_block {
            result.push_str(&code_block_header);
            result.push('\n');
            result.push_str(&code_block_content);
        }

        result.trim_end().to_string()
    }

    // ===== Level 2: Session Compact =====

    /// Compress older messages into summaries while keeping recent ones intact.
    fn session_compact(&self, entries: &[ContextEntry]) -> Vec<ContextEntry> {
        let total = entries.len();
        let keep = self.config.session_keep_recent.min(total);
        let split = total.saturating_sub(keep);

        if split == 0 {
            return entries.to_vec();
        }

        let old = &entries[..split];
        let recent = &entries[split..];

        // Summarize old entries in batches
        let mut summarized: Vec<ContextEntry> = Vec::new();

        for batch in old.chunks(self.config.session_batch_size) {
            let summary = self.create_session_summary(batch);
            let mut entry = ContextEntry::new("system", &summary);
            entry.priority = crate::memory::context::ContextPriority::Low;
            entry.source = crate::memory::context::ContextSource::Memory;
            summarized.push(entry);
        }

        summarized.extend(recent.iter().cloned());
        summarized
    }

    /// Create a summary of a batch of messages
    fn create_session_summary(&self, entries: &[ContextEntry]) -> String {
        let mut summary = String::from("[会话历史摘要]\n");
        let mut topics = Vec::new();
        let mut key_points = Vec::new();

        for entry in entries {
            match entry.role.as_str() {
                "user" => {
                    let preview: String = entry.content.chars().take(100).collect();
                    topics.push(format!("Q: {}", preview));
                }
                "assistant" => {
                    // Extract first meaningful line
                    let first_line = entry
                        .content
                        .lines()
                        .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                        .unwrap_or("(回复)");
                    let preview: String = first_line.chars().take(100).collect();
                    key_points.push(format!("A: {}", preview));
                }
                _ => {}
            }
        }

        if !topics.is_empty() {
            summary.push_str("话题:\n");
            for t in &topics {
                summary.push_str(&format!("  {}\n", t));
            }
        }
        if !key_points.is_empty() {
            summary.push_str("要点:\n");
            for p in &key_points {
                summary.push_str(&format!("  {}\n", p));
            }
        }

        summary
    }

    // ===== Level 3: Memory Compact =====

    /// Extract key memories and heavily compress the rest.
    /// Returns (compacted_entries, number_of_memories_extracted)
    fn memory_compact(&self, entries: &[ContextEntry]) -> (Vec<ContextEntry>, usize) {
        let mut memories: Vec<String> = Vec::new();
        let mut remaining: Vec<ContextEntry> = Vec::new();

        for entry in entries {
            // Check if this entry contains memory-worthy content
            let memory_content = self.extract_memory_content(&entry.content);
            if !memory_content.is_empty() {
                memories.extend(memory_content);
            }
        }

        // Create a consolidated memory block entry
        if !memories.is_empty() {
            let memory_block = format!(
                "[长期记忆 - {} 条关键信息]\n{}",
                memories.len(),
                memories
                    .iter()
                    .enumerate()
                    .map(|(i, m)| format!("{}. {}", i + 1, m))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            let mut memory_entry = ContextEntry::new("system", &memory_block);
            memory_entry.priority = crate::memory::context::ContextPriority::High;
            memory_entry.source = crate::memory::context::ContextSource::Memory;
            remaining.insert(0, memory_entry);
        }

        let memory_count = memories.len();

        // Keep only the most recent entries that fit
        let keep_count = self.config.session_keep_recent;
        let recent: Vec<ContextEntry> = entries
            .iter()
            .rev()
            .take(keep_count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        remaining.extend(recent);

        (remaining, memory_count)
    }

    /// Extract memory-worthy content from a message
    fn extract_memory_content(&self, content: &str) -> Vec<String> {
        let mut memories = Vec::new();
        let content_lower = content.to_lowercase();

        // Check if any memory keywords are present
        let has_keyword = self
            .config
            .memory_keywords
            .iter()
            .any(|kw| content_lower.contains(&kw.to_lowercase()));

        if !has_keyword {
            return memories;
        }

        // Extract sentences containing keywords
        let separators = &['.', '。', '！', '!', '？', '?', '\n'];
        for sentence in content.split(|c: char| separators.contains(&c)) {
            let sentence = sentence.trim();
            if sentence.is_empty() || sentence.len() < 10 {
                continue;
            }
            let sentence_lower = sentence.to_lowercase();
            if self
                .config
                .memory_keywords
                .iter()
                .any(|kw| sentence_lower.contains(&kw.to_lowercase()))
            {
                let memory: String = sentence.chars().take(200).collect();
                memories.push(memory);
            }
        }

        memories
    }

    /// Get current configuration
    pub fn config(&self) -> &CompactConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: CompactConfig) {
        self.config = config;
    }
}

impl Default for CompactService {
    fn default() -> Self {
        Self::with_defaults()
    }
}
