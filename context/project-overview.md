# Project Overview: Quantum Code

## Core Identity
Quantum Code is a high-performance, local-first AI coding assistant built in Rust. It aims to bridge the gap between heavy cloud-based agents and local-first efficiency using an intelligent routing architecture.

## Technical Foundation
- **Language**: Pure Rust (Tokio, Ratatui, Reqwest).
- **Performance**: <1ms routing latency using regex-based intent classification.
- **Architecture**: 7-Layer Intelligent Router (Intent, Complexity, Mode, Model Tier, Tool Policy, Context Budget, Memory Policy).

## Key Features
- **Multi-Provider**: Support for Anthropic, OpenAI, Ollama, LM Studio, and llama.cpp.
- **Token Efficiency**: Aggressive prompt compaction and token budget enforcement (target < 600 system tokens).
- **Agentic Loop**: Multi-turn tool execution capability (Read, Write, Bash, Grep, Glob).
- **RAG System**: In-memory keyword-based retrieval for codebase context.

## Project Status
- **Core Engine**: Fully implemented with 155+ passing tests.
- **Integration**: Router is wired into `AgentExecutor` and `App` state.
- **Current Focus**: Enforcing tool policies and wiring relevance-based memory loading.

## Design Philosophy
- **Local-First**: Prioritizes local inference (Ollama/llama.cpp) whenever complexity allows.
- **Explainability**: Every routing decision includes a reasoning field for transparency.
- **Minimalism**: Reduced dependency footprint and optimized system prompts.