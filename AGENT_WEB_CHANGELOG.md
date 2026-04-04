# Agent Web 上下文管理系统 - 变更日志

> **Commit**: `feat: 集成上下文管理、历史裁剪与三级压缩管线`  
> **分支**: `dev-opus`  
> **日期**: 2026-04-04  
> **改动规模**: 8 files changed, +1171 -14 lines

---

## 目录

- [背景与问题](#背景与问题)
- [架构概览](#架构概览)
- [新增模块](#新增模块)
  - [ContextManager 上下文窗口管理器](#1-contextmanager-上下文窗口管理器)
  - [HistorySnipManager 历史裁剪管理器](#2-historysnipmanager-历史裁剪管理器)
  - [CompactService 三级压缩服务](#3-compactservice-三级压缩服务)
- [修改文件清单](#修改文件清单)
  - [src/memory/context.rs](#srcmemorycontextrs)
  - [src/memory/history_snip.rs (新建)](#srcmemoryhistory_sniprs-新建)
  - [src/memory/mod.rs](#srcmemorymodrs)
  - [src/services/compact/mod.rs (新建)](#srcservicescompactmodrs-新建)
  - [src/services/mod.rs](#srcservicesmodrs)
  - [src/web/agent/state.rs](#srcwebagentstatersrs)
  - [src/web/agent/handlers.rs](#srcwebagenthanlersrs)
  - [web-frontend/src/hooks/useChat.ts](#web-frontendsrchooksusechatts)
- [数据流与处理管线](#数据流与处理管线)
- [改动前后对比](#改动前后对比)
- [配置参数参考](#配置参数参考)

---

## 背景与问题

项目的 Web 聊天链路（`web-frontend` → `chat_handler` → LLM）存在以下核心问题：

| 问题 | 具体表现 |
|------|----------|
| **无多轮记忆** | `chat_handler` 每次请求仅构造 `[system_prompt, user_message]` 两条消息发给 LLM，不携带任何历史对话 |
| **session_id 空转** | 前端虽然传递了 `session_id`，但后端完全忽略，未进行任何会话隔离 |
| **上下文模块未集成** | `ContextManager`（128K token 窗口）已实现但从未被 handler 调用 |
| **无 Token 保护** | 没有上限管控，长对话可能超出模型 context window 导致 API 报错 |
| **中文 Token 估算错误** | 原有的 `text.split_whitespace().count() / 3 * 4` 对中文恒返回 0（中文无空格分词） |

---

## 架构概览

本次改动构建了完整的三层上下文压缩管线：

```
┌─────────────────────────────────────────────────────────────────┐
│                    用户发送消息                                    │
│                         │                                        │
│                         ▼                                        │
│            ┌──────────────────────┐                              │
│  Layer 0   │   ContextManager     │  128K token 窗口              │
│            │   + 优先级驱逐策略    │  system 消息永不被驱逐           │
│            └──────────┬───────────┘                              │
│                       │ get_entries()                            │
│                       ▼                                          │
│            ┌──────────────────────┐                              │
│  Layer 1   │  HistorySnipManager  │  4 种策略（默认 Hybrid）       │
│            │  智能历史裁剪         │  评分 + 分段摘要               │
│            └──────────┬───────────┘                              │
│                       │ snip_if_needed()                        │
│                       ▼                                          │
│            ┌──────────────────────┐                              │
│  Layer 2   │   CompactService     │  三级压缩管线                  │
│            │   L1: 微压缩         │  去空白/截断长文               │
│            │   L2: 会话压缩       │  >50K 时旧消息→摘要            │
│            │   L3: 记忆压缩       │  >80K 时提取关键记忆            │
│            └──────────┬───────────┘                              │
│                       │ compact()                                │
│                       ▼                                          │
│          [system_prompt] + 压缩后历史 → 发送给 LLM                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 新增模块

### 1. ContextManager 上下文窗口管理器

**文件**: `src/memory/context.rs`（已有，本次修改 Token 估算逻辑）

ContextManager 提供 128K token 的滑动窗口，管理多轮对话的消息存储与驱逐：

- **Token 窗口**: 默认 128,000 tokens，预留 10% 给系统开销
- **优先级驱逐**: 当窗口满时，按优先级从低到高驱逐（`Critical` 级别的 system 消息永不被驱逐）
- **按会话隔离**: 通过 `session_id` 维护独立的上下文实例

**本次修复 — Token 估算算法**:

```
改动前 (有 bug):
  text.split_whitespace().count() / 3 * 4
  → 中文文本没有空格分词，恒返回 0

改动后 (正确):
  - CJK 字符 (中日韩): 每字符 ≈ 1 token
  - 英文单词: 每词 ≈ 1.33 tokens (4/3)
  - 支持中英文混合文本
  - 最小返回 1 token（非空文本）
```

**优先级体系**:

| 优先级 | 适用对象 | 说明 |
|--------|----------|------|
| `Critical` | system 消息 | 永不被驱逐 |
| `High` | 记忆压缩提取的长期记忆 | 仅在极端情况下被驱逐 |
| `Normal` | user / assistant 消息 | 正常驱逐优先级 |
| `Low` | 会话摘要 | 优先被驱逐 |

---

### 2. HistorySnipManager 历史裁剪管理器

**文件**: `src/memory/history_snip.rs`（新建，477 行）

HistorySnipManager 在 ContextManager 之上提供智能裁剪，支持 4 种策略：

#### 策略 1: KeepRecent（保留最近 N 条）

最简单的策略 — 从最近的消息往前取，直到填满 token 预算：

```
[msg1, msg2, msg3, ..., msg98, msg99, msg100]
                                ↓ KeepRecent(10)
                    [msg91, msg92, ..., msg99, msg100]
```

- **优点**: 快速、零信息变形
- **缺点**: 完全丢失早期上下文
- **适用**: 短对话、实时性要求高的场景

#### 策略 2: PriorityBased（优先级评分筛选）

为每条消息计算综合优先级分数，贪心选取高分消息：

| 评分维度 | 规则 | 权重 |
|----------|------|------|
| 角色权重 | system: +100, user: +3, assistant: +2 | 基础分 |
| 时间衰减 | 线性衰减，最新 = +10.0，最旧 = +0.0 | 0~10 |
| 短消息加成 | token_count < 50 → +1.0（用户问题/指令） | +1 |
| 长回复惩罚 | assistant 且 token > 500 → -2.0 | -2 |

- **优点**: 精确保留重要内容
- **缺点**: 可能打断对话连贯性（虽然保持原序输出）
- **适用**: Token 紧张时需要精准保留关键信息

#### 策略 3: Smart（分段摘要压缩）

将旧消息分组为固定大小的 segment，每组压缩为一条摘要：

```
[旧msg1, 旧msg2, ..., 旧msg6] → "[历史对话摘要] • 用户: xxx • 助手: xxx"
[旧msg7, 旧msg8, ..., 旧msg12] → "[历史对话摘要] • 用户: xxx • 助手: xxx"
[最近msg1, 最近msg2, ..., 最近msg10] → 保持原文不变
```

摘要生成规则：
- user 消息：提取前 120 字符作为话题预览
- assistant 消息：提取第一个句子（句号/换行分割），取前 80 字符

- **优点**: 保留上下文概览 + 最近消息原文
- **缺点**: 摘要质量依赖启发式规则（非 LLM 生成）
- **适用**: 长对话保持上下文连贯

#### 策略 4: Hybrid（混合策略，默认）

PriorityBased + Smart 的两阶段组合：

**Phase 1**: 对旧消息（非最近 N 条）进行优先级评分
  - 高分消息 → 保留原文（占旧消息预算的 50%）
  - 低分消息 → 送入 Phase 2

**Phase 2**: 对低分消息按 segment_size 分组，生成摘要

**最终输出**: `[摘要块] + [高分旧消息原文] + [最近 N 条原文]`

- **优点**: 兼顾重要性 + 连贯性，效果最佳
- **适用**: 通用场景（默认策略）

#### 配置项

```rust
HistorySnipConfig {
    max_tokens: 100_000,       // Token 上限
    strategy: Hybrid,           // 默认策略
    keep_recent_count: 10,      // 保留最近 10 条
    segment_size: 6,            // 每 6 条压缩为 1 条摘要
    system_prompt_reserve: 4000,// 为 system prompt 预留
    enabled: true,              // 默认开启
}
```

---

### 3. CompactService 三级压缩服务

**文件**: `src/services/compact/mod.rs`（新建，476 行）

CompactService 是无状态的压缩管线，可跨会话共享。依次执行三级压缩：

#### Level 1: 微压缩 (Microcompact)

**触发条件**: 始终执行（`enable_microcompact: true`）

| 操作 | 说明 | 参数 |
|------|------|------|
| 去冗余空白 | 合并连续 3+ 空行为 2 空行，合并连续多空格 | 无 |
| 截断长 assistant 回复 | 保留开头 60% + 结尾 40%，中间标注省略字数 | `micro_max_assistant_chars: 2000` |
| 截断大代码块 | 保留代码块前 N 个字符 + 截断提示 | `micro_max_code_block_chars: 800` |

**压缩示例**:

```
原始 assistant 回复 (5000 字符):
  "这是一个很长的回复...（5000字）..."

微压缩后:
  "这是一个很长的回复...（前1200字）...
   
   ... [省略 2200 字符] ...
   
   ...最后的总结部分（后800字）..."
```

#### Level 2: 会话压缩 (SessionCompact)

**触发条件**: 总 token 数 > 50,000

| 操作 | 说明 |
|------|------|
| 分割消息 | 分为「旧消息区」和「最近消息区」（最近 10 条） |
| 批次摘要 | 旧消息每 6 条一批，生成 `[会话历史摘要]` |
| 格式 | `话题: Q: 用户问题预览` + `要点: A: 助手回复首行` |

**摘要格式示例**:

```
[会话历史摘要]
话题:
  Q: 请帮我分析这个 Rust 项目的性能瓶颈
  Q: 如何优化异步任务的并发数量
要点:
  A: 根据 profiling 结果，主要瓶颈在于...
  A: 建议使用 tokio::sync::Semaphore 来限制并发...
```

#### Level 3: 记忆压缩 (MemoryCompact)

**触发条件**: 总 token 数 > 80,000（Level 2 之后仍超标）

| 操作 | 说明 |
|------|------|
| 关键词扫描 | 在所有消息中搜索 memory_keywords 关键词 |
| 句子提取 | 提取包含关键词的完整句子（10~200 字符） |
| 记忆块 | 生成 `[长期记忆 - N 条关键信息]` 系统消息 |
| 保留最近 | 仅保留最近 10 条原始消息 |

**关键词列表** (可配置):

```
important, key, remember, 注意, 重要, 记住, 关键, 决定, 结论, TODO
```

**记忆块格式示例**:

```
[长期记忆 - 3 条关键信息]
1. 用户的项目使用 Rust + Tokio 异步框架，这是一个重要的技术决策
2. 记住用户偏好中文回复，并且需要代码注释
3. TODO: 后续需要添加数据库连接池的配置
```

#### 三级压缩流程图

```
输入: entries[] (原始历史消息)
     │
     ├─ Level 1: Microcompact [始终执行]
     │   ├─ 去冗余空白
     │   ├─ 截断长 assistant 回复 (>2000字)
     │   └─ 截断大代码块 (>800字)
     │
     ├─ Level 2: SessionCompact [>50K tokens 触发]
     │   ├─ 旧消息 → 每 6 条批次摘要
     │   └─ 保留最近 10 条原文
     │
     └─ Level 3: MemoryCompact [>80K tokens 触发]
         ├─ 提取含关键词的句子 → 长期记忆块
         └─ 仅保留最近 10 条原文

输出: (compacted_entries[], CompactResult)
```

---

## 修改文件清单

### `src/memory/context.rs`

**改动类型**: 修改（修复 bug）

- 修复 `estimate_tokens()` 方法，从 `split_whitespace().count() / 3 * 4`（中文恒返回 0）改为 CJK/英文混合估算
- CJK 字符：每字符 ≈ 1 token
- 英文单词：每词 ≈ 1.33 tokens
- 最小返回 1 token

### `src/memory/history_snip.rs` (新建)

**改动类型**: 新建（477 行）

- `HistorySnipManager` 结构体 + 4 种压缩策略实现
- `HistorySnipConfig` 配置结构 + `Default` 实现
- `SnipStrategy` 枚举：KeepRecent / PriorityBased / Smart / Hybrid
- `SnippedSegment` 压缩段记录结构
- `SnipStats` 统计结构
- 本地启发式摘要生成（无需 LLM 调用）

### `src/memory/mod.rs`

**改动类型**: 修改

- 新增 `pub mod history_snip;` 模块声明
- 新增 `pub use history_snip::{HistorySnipManager, HistorySnipConfig, SnipStrategy};` 导出

### `src/services/compact/mod.rs` (新建)

**改动类型**: 新建（476 行）

- `CompactService` 结构体 + 三级压缩管线实现
- `CompactConfig` 配置结构 + `Default` 实现
- `CompactResult` / `CompactLevel` 压缩结果结构
- Level 1: `microcompact()` — 去空白、截断长回复、截断代码块
- Level 2: `session_compact()` — 分批摘要 + 保留最近消息
- Level 3: `memory_compact()` — 关键词记忆提取 + 最近消息

### `src/services/mod.rs`

**改动类型**: 修改

- 新增 `pub mod compact;` 模块声明

### `src/web/agent/state.rs`

**改动类型**: 修改（重构）

**新增 `SessionContext` 结构体**:

```rust
pub struct SessionContext {
    pub context_mgr: Arc<ContextManager>,       // 128K token 窗口
    pub snip_mgr: Arc<HistorySnipManager>,      // 4 种压缩策略
}
```

**`AgentWebState` 改动**:

| 字段 | 改动前 | 改动后 |
|------|--------|--------|
| `session_contexts` | `HashMap<String, Arc<ContextManager>>` | `HashMap<String, Arc<SessionContext>>` |
| `compact_service` | 不存在 | `Arc<CompactService>`（共享实例） |

**新增方法**:
- `get_or_create_session_context()` — 双重检查锁，返回 `Arc<SessionContext>`
- 保留原有 `get_or_create_context()` 作为向后兼容

### `src/web/agent/handlers.rs`

**改动类型**: 修改（三大分支全部集成）

**改动前**: 每次构造 `[system_prompt] + context_mgr.get_messages()`

**改动后**: 每次构造 `[system_prompt] + 压缩管线处理后的历史消息`

三个分支（Orchestrator / General Purpose / Direct Agent）统一改为：

```rust
// 1. 获取原始上下文条目
let raw_entries = context_mgr_clone.get_entries().await;

// 2. HistorySnip 策略裁剪
let snipped_entries = snip_mgr_clone.snip_if_needed(&raw_entries).await;

// 3. 三级压缩管线
let (compacted_entries, compact_result) = compact_service_clone.compact(&snipped_entries);

// 4. 转换为 ChatMessage
let history_messages: Vec<ChatMessage> = compacted_entries.iter()
    .map(|e| ChatMessage { role: e.role.clone(), content: e.content.clone(), tool_calls: None })
    .collect();

// 5. 拼接并发送给 LLM
target_messages.extend(history_messages);
```

每个分支在压缩实际生效时会输出日志：
```
🗜️  Compact[Orchestrator]: Session applied, 52000 -> 18000 tokens (35% ratio)
```

### `web-frontend/src/hooks/useChat.ts`

**改动类型**: 修改

| 改动 | 说明 |
|------|------|
| 新增 `generateSessionId()` | 生成格式 `session-{timestamp}-{random7}` |
| 新增 `sessionIdRef` | `useRef` 持久化 session_id，跨渲染周期保持 |
| `sendMessage` | 自动携带 `activeSessionId` 发送给后端 |
| `clearMessages` | 重置 `sessionIdRef` → 新对话 = 新上下文窗口 |
| 导出 `sessionId` | 供外部组件使用 |

---

## 数据流与处理管线

### 单次请求完整流程

```
前端 useChat.ts                      后端 chat_handler                          LLM
─────────────                      ──────────────────                        ─────
sendMessage("你好")
  ├─ sessionId: "session-xxx"   →  get_or_create_session_context("session-xxx")
  │                                  ├─ context_mgr (128K window)
  │                                  ├─ snip_mgr (Hybrid strategy)
  │                                  └─ compact_service (shared)
  │
  └─ message: "你好"            →  context_mgr.add_user("你好")
                                   📊 Context[session-]: 1 entries, 2/128000 tokens

                                   raw_entries = context_mgr.get_entries()
                                   snipped = snip_mgr.snip_if_needed(raw_entries)
                                   (compacted, result) = compact_service.compact(snipped)

                                   messages = [system_prompt] + compacted
                                                                            → [system, user:"你好"]
                                                                            ← "你好！有什么..."

                                   context_mgr.add_assistant("你好！有什么...")
                                   💾 Saved assistant response (18 chars)
                                ← SSE stream: "你好！有什么..."
```

### 长对话压缩触发流程

```
第 1~20 轮:
  Context: 5K tokens → 无压缩触发
  HistorySnip: 5K < 96K budget → 原样返回
  CompactService: L1 微压缩（去空白）→ ~4.8K tokens

第 50 轮:
  Context: 55K tokens → 无驱逐
  HistorySnip: 55K < 96K budget → 原样返回
  CompactService: L1 微压缩 → 52K, L2 会话压缩触发 (>50K) → ~18K tokens
  🗜️ Compact[GP]: Session applied, 52000 -> 18000 tokens (35% ratio)

第 100 轮 (极端):
  Context: 95K tokens → 部分驱逐至 ~90K
  HistorySnip: 90K < 96K → Hybrid 裁剪 → ~70K
  CompactService: L1 → 65K, L2 → 45K, L3 记忆压缩触发 (仍>80K前) → ~15K
  🧠 MemoryCompact: 45000 -> 15000 tokens, 8 memories extracted
```

---

## 改动前后对比

| 维度 | 改动前 | 改动后 |
|------|--------|--------|
| **多轮记忆** | ❌ 无，每次只发 `[system, user]` | ✅ 完整多轮对话历史 |
| **Token 管理** | ❌ 无上限保护 | ✅ 128K token 窗口 + 优先级驱逐 |
| **session_id** | 前端传了，后端忽略 | ✅ 按 session 隔离上下文 |
| **Assistant 回复** | 发完就丢 | ✅ 收集完整回复存入上下文 |
| **中文 Token 估算** | `words/3*4` → 中文恒为 0 | ✅ CJK 按字符、英文按词混合估算 |
| **上下文溢出** | ❌ 无保护（会超限报错） | ✅ 三层压缩自动缩减 |
| **历史裁剪** | ❌ 不存在 | ✅ 4 种策略可选（默认 Hybrid） |
| **长对话压缩** | ❌ 不存在 | ✅ L1 微压缩 + L2 会话压缩 + L3 记忆压缩 |
| **关键信息保留** | ❌ 所有信息等价 | ✅ 关键词记忆提取，system 永不驱逐 |

---

## 配置参数参考

### HistorySnipConfig

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `max_tokens` | 100,000 | 历史消息的最大 token 数 |
| `strategy` | `Hybrid` | 裁剪策略：KeepRecent / PriorityBased / Smart / Hybrid |
| `keep_recent_count` | 10 | 始终保留的最近消息数量 |
| `segment_size` | 6 | Smart/Hybrid 策略中每组消息数 |
| `system_prompt_reserve` | 4,000 | 为 system prompt 预留的 token 数 |
| `enabled` | `true` | 是否启用裁剪 |

### CompactConfig

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `enable_microcompact` | `true` | 启用 Level 1 微压缩 |
| `enable_session_compact` | `true` | 启用 Level 2 会话压缩 |
| `enable_memory_compact` | `true` | 启用 Level 3 记忆压缩 |
| `micro_max_assistant_chars` | 2,000 | assistant 回复最大字符数 |
| `micro_max_code_block_chars` | 800 | 代码块最大字符数 |
| `session_keep_recent` | 10 | L2/L3 保留的最近消息数 |
| `session_batch_size` | 6 | L2 摘要的批次大小 |
| `session_compact_threshold` | 50,000 | L2 触发阈值（tokens） |
| `memory_compact_threshold` | 80,000 | L3 触发阈值（tokens） |
| `memory_keywords` | `["important", "key", "remember", "注意", "重要", "记住", "关键", "决定", "结论", "TODO"]` | L3 关键词列表 |

### ContextManager

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `max_tokens` | 128,000 | 上下文窗口最大 token 数 |
| `reserved_tokens` | max_tokens / 10 | 预留 token 数 |

---

> **注**: 所有压缩摘要均使用本地启发式算法生成，不依赖额外的 LLM 调用，因此不会产生额外的 API 费用或延迟。
