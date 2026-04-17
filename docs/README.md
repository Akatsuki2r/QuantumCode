# Quantum Code Documentation

Welcome to the Quantum Code documentation. This folder contains comprehensive technical documentation about the project architecture, implementation details, and development roadmap.

## Documentation Index

### Core Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System architecture overview, component relationships, data flow |
| [ROUTER_DEEP_DIVE.md](./ROUTER_DEEP_DIVE.md) | Complete router documentation - 7-layer policy engine |
| [PROVIDERS.md](./PROVIDERS.md) | AI provider implementations, API integrations, local model discovery |
| [RAG_SYSTEM.md](./RAG_SYSTEM.md) | RAG (Retrieval-Augmented Generation) implementation details |
| [TOOLS_SYSTEM.md](./TOOLS_SYSTEM.md) | Tool policy system, allowed/disallowed tools per mode |

### Implementation Guides

| Document | Description |
|----------|-------------|
| [MODE_SYSTEM.md](./MODE_SYSTEM.md) | Mode state machine - chat, plan, build, review, debug |
| [LOCAL_LLM_DISCOVERY.md](./LOCAL_LLM_DISCOVERY.md) | Auto-discovery of Ollama, LM Studio, llama.cpp models |
| [PROMPT_COMPACTION.md](./PROMPT_COMPACTION.md) | Prompt compaction for efficient token usage |
| [TUI_WIDGETS.md](./TUI_WIDGETS.md) | Terminal UI widgets - dropdown, tabs, chat interface |

### Project Status

| Document | Description |
|----------|-------------|
| [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) | What's implemented vs what's planned |
| [CHOKE_POINTS.md](./CHOKE_POINTS.md) | Performance bottlenecks, areas needing optimization |
| [ROADMAP.md](./ROADMAP.md) | Development roadmap, upcoming features |

### Research References

| Document | Description |
|----------|-------------|
| [research_01_architecture_and_build.md](../research_01_architecture_and_build.md) | Initial architecture research |
| [research_02_router_deep_dive.md](../research_02_router_deep_dive.md) | Router deep dive - academic reference |
| [research_03_skills_agents_optimization.md](../research_03_skills_agents_optimization.md) | Skills and agents optimization research |
| [research_04_todo_audit_and_roadmap.md](../research_04_todo_audit_and_roadmap.md) | TODO audit and roadmap planning |

## Quick Reference

### Router 7-Layer Pipeline

```
User Prompt → Intent → Complexity → Mode → Model Tier → Tools → Context → Memory → Decision
```

### Provider Support

| Provider | Type | API Key | Status |
|----------|------|---------|--------|
| Anthropic Claude | Cloud | Yes | Implemented |
| OpenAI GPT | Cloud | Yes | Implemented |
| Ollama | Local | No | Implemented |
| LM Studio | Local | No | Implemented |
| llama.cpp | Local | No | Implemented |

### Modes

| Mode | Writes? | Tools | Use Case |
|------|---------|-------|----------|
| Chat | No | Minimal | Quick questions |
| Plan | No | Read-only | Architecture, planning |
| Build | Yes | Full | Implementation |
| Review | No | Read-only | Code review |
| Debug | Limited | Read + Bash | Debugging |

## Getting Started

1. Read [ARCHITECTURE.md](./ARCHITECTURE.md) for system overview
2. Read [ROUTER_DEEP_DIVE.md](./ROUTER_DEEP_DIVE.md) for router details
3. Read [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) for current status
4. Check [ROADMAP.md](./ROADMAP.md) for upcoming work

## Contributing

When adding new features:
1. Update relevant documentation in this folder
2. Add tests for new functionality
3. Update IMPLEMENTATION_STATUS.md if status changes
4. Note any new choke points or optimizations in CHOKE_POINTS.md
