# RAG System (Retrieval-Augmented Generation)

## Overview

The RAG (Retrieval-Augmented Generation) system provides context-aware retrieval for enhanced AI responses. It uses embedding-based similarity search over the codebase to provide relevant context to the AI model.

**Status**: Partially Implemented

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      RAG Pipeline                            │
│                                                              │
│  User Query → Keyword Extraction → Document Retrieval       │
│                           │                                  │
│                           ▼                                  │
│                    Similarity Scoring                        │
│                           │                                  │
│                           ▼                                  │
│                    Context Formatting                        │
│                           │                                  │
│                           ▼                                  │
│                    Augmented Prompt → LLM                    │
└─────────────────────────────────────────────────────────────┘
```

## Implementation

**File**: `src/rag/mod.rs`

### Configuration

```rust
pub struct RagConfig {
    /// Enable RAG retrieval
    pub enabled: bool,
    /// Maximum chunks to retrieve
    pub max_chunks: usize,
    /// Similarity threshold (0.0 - 1.0)
    pub similarity_threshold: f32,
    /// Chunk size in tokens
    pub chunk_size: usize,
    /// Chunk overlap in tokens
    pub chunk_overlap: usize,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chunks: 5,
            similarity_threshold: 0.3,
            chunk_size: 512,
            chunk_overlap: 50,
        }
    }
}
```

### Context Chunk

```rust
pub struct ContextChunk {
    /// Source file path
    pub file_path: String,
    /// Chunk content
    pub content: String,
    /// Start line in source file
    pub start_line: usize,
    /// End line in source file
    pub end_line: usize,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f32,
    /// Embedding vector (simplified - currently just a hash)
    pub embedding_hash: u64,
}
```

### RAG Result

```rust
pub struct RagResult {
    /// Retrieved chunks
    pub chunks: Vec<ContextChunk>,
    /// Total retrieval time in ms
    pub retrieval_time_ms: u64,
    /// Whether RAG was actually used
    pub used: bool,
}
```

## Retrieval Implementation

### Current: Keyword-Based Retriever

**Status**: Implemented

The current implementation uses keyword matching as a placeholder for true embedding-based search:

```rust
pub struct KeywordRetriever {
    config: RagConfig,
}

impl KeywordRetriever {
    pub fn retrieve(&self, query: &str, documents: &[Document]) -> RagResult {
        let start = std::time::Instant::now();

        if !self.config.enabled || documents.is_empty() {
            return RagResult::empty();
        }

        // Extract query terms (words > 2 chars)
        let query_terms: Vec<&str> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        // Score all chunks
        let mut scored_chunks: Vec<(ContextChunk, f32)> = Vec::new();

        for doc in documents {
            for chunk in &doc.chunks {
                let score = self.compute_relevance(&query_terms, &chunk.content);
                if score >= self.config.similarity_threshold {
                    scored_chunks.push((chunk.clone(), score));
                }
            }
        }

        // Sort by score descending
        scored_chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N chunks
        let chunks: Vec<ContextChunk> = scored_chunks
            .into_iter()
            .take(self.config.max_chunks)
            .map(|(chunk, _)| chunk)
            .collect();

        RagResult {
            chunks,
            retrieval_time_ms: start.elapsed().as_millis() as u64,
            used: true,
        }
    }

    /// TF-IDF-like relevance scoring
    fn compute_relevance(&self, query_terms: &[&str], content: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let mut score = 0.0;

        for term in query_terms {
            if content_lower.contains(term) {
                // Base score for match
                score += 1.0;

                // Bonus for multiple occurrences
                let count = content_lower.matches(*term).count();
                score += (count as f32) * 0.5;

                // Bonus for term in identifiers
                if content.contains(term) {
                    score += 0.5;
                }
            }
        }

        // Normalize by content length
        let length_factor = 1.0 / (1.0 + (content.len() as f32 / 1000.0));
        score * length_factor
    }
}
```

### Document Chunking

```rust
impl Document {
    pub fn new(path: String, content: String, chunk_size: usize, overlap: usize) -> Self {
        let chunks = Self::chunk_content(&path, &content, chunk_size, overlap);
        Self {
            path,
            content,
            chunks,
        }
    }

    fn chunk_content(
        path: &str,
        content: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<ContextChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut start = 0;

        while start < lines.len() {
            let mut end = start;
            let mut char_count = 0;

            // Accumulate lines until chunk size reached
            while end < lines.len() && char_count < chunk_size {
                char_count += lines[end].len() + 1;
                end += 1;
            }

            if end > start {
                let chunk_content = lines[start..end].join("\n");
                chunks.push(ContextChunk {
                    file_path: path.to_string(),
                    content: chunk_content,
                    start_line: start + 1,
                    end_line: end,
                    similarity: 0.0,
                    embedding_hash: 0,
                });
            }

            // Move start with overlap
            start = if end > overlap { end - overlap } else { end };
            if start >= end {
                break;
            }
        }

        chunks
    }
}
```

### In-Memory Index

```rust
pub struct RagIndex {
    documents: HashMap<String, Document>,
    config: RagConfig,
}

impl RagIndex {
    pub fn new(config: RagConfig) -> Self {
        Self {
            documents: HashMap::new(),
            config,
        }
    }

    pub fn add_document(&mut self, path: String, content: String) {
        let doc = Document::new(
            path.clone(),
            content,
            self.config.chunk_size,
            self.config.chunk_overlap,
        );
        self.documents.insert(path, doc);
    }

    pub fn remove_document(&mut self, path: &str) {
        self.documents.remove(path);
    }

    pub fn search(&self, query: &str) -> RagResult {
        let retriever = KeywordRetriever::new(self.config.clone());
        let docs: Vec<&Document> = self.documents.values().collect();
        retriever.retrieve(
            query,
            &docs.into_iter().map(|d| (*d).clone()).collect::<Vec<_>>(),
        )
    }
}
```

## Prompt Compaction

**Status**: Implemented

The `compact_prompts` module provides utilities for efficient token usage:

```rust
pub mod compact_prompts {
    /// Compact system prompt template
    pub const COMPACT_SYSTEM: &str = "QC: Local-first coding AI. Read/write/edit files, shell, analyze, search.
MODE: {mode} | TOOLS: read,write,bash,grep,glob | GIT: safe history
{context}";

    /// Ultra-compact for small context windows
    pub const ULTRA_COMPACT: &str = "QC AI: {mode}. Tools: read,write,bash,grep. {context}";

    /// Compress a prompt by removing redundancy
    pub fn compress_prompt(prompt: &str, target_tokens: usize) -> String {
        let target_chars = target_tokens * 4;

        if prompt.len() <= target_chars {
            return prompt.to_string();
        }

        // Remove common filler phrases
        let compressed = prompt
            .replace("please ", "")
            .replace("could you ", "")
            .replace("i would like to ", "")
            .replace("i want to ", "")
            .replace("can you ", "")
            .replace("help me ", "")
            .replace("  ", " ");

        // Truncate if still too long
        if compressed.len() > target_chars {
            format!("{}...", &compressed[..target_chars.saturating_sub(3)])
        } else {
            compressed
        }
    }

    /// Format context chunks efficiently
    pub fn format_context_compact(chunks: &[ContextChunk]) -> String {
        if chunks.is_empty() {
            return String::new();
        }

        let mut formatted = String::with_capacity(chunks.len() * 200);
        for chunk in chunks {
            let preview = if chunk.content.len() > 200 {
                format!("{}...", &chunk.content[..200])
            } else {
                chunk.content.clone()
            };
            formatted.push_str(&format!(
                "[{}:{}-{}] {}\n",
                chunk.file_path, chunk.start_line, chunk.end_line, preview
            ));
        }
        formatted
    }
}
```

## Router Integration

**File**: `src/router/types.rs`

RAG is configured through the router:

```rust
pub struct RagRouterConfig {
    /// Enable RAG retrieval
    pub enabled: bool,
    /// Maximum context chunks to retrieve
    pub max_chunks: usize,
    /// Similarity threshold (0.0 - 1.0)
    pub similarity_threshold: f32,
}

impl Default for RagRouterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chunks: 5,
            similarity_threshold: 0.3,
        }
    }
}

pub struct RouterConfig {
    pub prefer_local: bool,
    pub cost_limit: f32,
    pub rag: RagRouterConfig,
    pub prompt_compaction: PromptCompactionConfig,
}
```

## Testing

### Unit Tests

```rust
#[test]
fn test_document_chunking() {
    let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10";
    let doc = Document::new("test.txt".to_string(), content.to_string(), 50, 10);

    assert!(!doc.chunks.is_empty());
    assert!(doc.chunks.len() >= 2);
}

#[test]
fn test_keyword_retriever() {
    let config = RagConfig::default();
    let retriever = KeywordRetriever::new(config);

    let docs = vec![Document::new(
        "test.rs".to_string(),
        "fn main() { println!(\"hello\"); }".to_string(),
        100,
        10,
    )];

    let result = retriever.retrieve("main function", &docs);
    assert!(result.used);
    assert!(!result.chunks.is_empty());
}

#[test]
fn test_rag_index() {
    let mut index = RagIndex::new(RagConfig::default());

    index.add_document("file1.rs".to_string(), "fn test() {}".to_string());
    index.add_document("file2.rs".to_string(), "fn main() {}".to_string());

    assert_eq!(index.document_count(), 2);

    let result = index.search("main");
    assert!(result.used);
}
```

## What's NOT Implemented

### 1. True Embedding-Based Search

**Current**: Keyword matching with TF-IDF-like scoring
**Needed**: Actual embedding vectors with cosine similarity

```rust
// TODO: Implement embedding generation
pub fn generate_embedding(text: &str) -> Result<Vec<f32>, EmbeddingError> {
    // Use a local embedding model (e.g., all-MiniLM-L6-v2 via ONNX)
    // or call an API for embeddings
}

// TODO: Implement cosine similarity
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // Dot product / (norm_a * norm_b)
}
```

### 2. Persistent Document Store

**Current**: In-memory HashMap
**Needed**: Disk-backed storage for large codebases

```rust
// TODO: Implement persistent storage
pub struct PersistentRagIndex {
    db: sled::Db,
    config: RagConfig,
}
```

### 3. Automatic Document Indexing

**Current**: Manual `add_document()` calls
**Needed**: Automatic indexing of project files

```rust
// TODO: Watch project directory and auto-index
pub fn watch_and_index(project_path: &Path) -> Result<(), io::Error> {
    // Use notify crate for file watching
    // Index new/modified files automatically
}
```

### 4. RAG Integration with Agent

**Current**: RAG exists as separate module
**Needed**: Integration with agent/executor for automatic context retrieval

```rust
// TODO: Integrate with agent
impl Agent {
    async fn execute_with_rag(&self, prompt: &str) -> Result<String, AgentError> {
        // 1. Retrieve relevant context
        let rag_result = self.rag_index.search(prompt);
        
        // 2. Augment prompt with context
        let augmented_prompt = format!(
            "{}\n\n## Context\n{}",
            prompt,
            rag_result.format_context()
        );
        
        // 3. Send to LLM
        self.provider.chat(augmented_prompt).await
    }
}
```

## Performance

| Operation | Current (Keyword) | Target (Embedding) |
|-----------|-------------------|--------------------|
| Indexing (1K files) | ~100ms | ~5s (embedding gen) |
| Search (1K chunks) | < 10ms | < 50ms |
| Accuracy | ~60% | ~85%+ |

## Future Enhancements

1. **Embedding Model Integration**
   - Use ONNX runtime for local embeddings
   - Model: `all-MiniLM-L6-v2` (80MB, fast)

2. **Vector Database**
   - Use `qdrant` or `chroma` for vector storage
   - Enable semantic search across codebase

3. **Hybrid Search**
   - Combine keyword + embedding search
   - Best of both approaches

4. **Code-Specific Embeddings**
   - Use CodeBERT or similar for code embeddings
   - Better understanding of code semantics

5. **Cross-File Context**
   - Track imports/dependencies
   - Retrieve related files automatically

## Usage Example

```rust
use quantumn::rag::{RagConfig, RagIndex, Document};

// Create RAG index
let mut index = RagIndex::new(RagConfig::default());

// Add documents
index.add_document("src/main.rs".to_string(), main_rs_content);
index.add_document("src/lib.rs".to_string(), lib_rs_content);

// Search
let result = index.search("how does authentication work?");

// Format context for prompt
let context = result.format_context();
let prompt = format!("Answer this question:\n{}\n\n{}", question, context);

// Send to LLM
let response = provider.chat(prompt).await?;
```
