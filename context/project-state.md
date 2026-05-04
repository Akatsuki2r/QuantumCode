# Project State - Prompt and Local Inference Optimization

Date: 2026-05-04

## Current Objective

Quantumn Code must stay usable on budget cloud plans, local-only setups, and older hardware. The guiding target is frontier-level coding quality per token, not maximum prompt verbosity or always choosing the largest model.

## Decisions Made

1. Prompts are optimized, but must stay compact.
   - Why: system prompt tokens are paid on every request and also increase local prefill time.
   - What changed: core, mode, router, and agent prompts were rewritten to be project-aware, terse, and action-oriented.
   - Outcome: prompts now emphasize evidence, small diffs, router-selected modes, minimal context, and verification without long persona filler.

2. Tool metadata is filtered by router policy.
   - Why: advertising every tool wastes tokens and tempts the model to call tools that the router will block.
   - What changed: agent prompt builders can now inject only allowed tools for the current decision.
   - Outcome: plan/review/chat paths carry smaller tool prompts and less behavioral ambiguity.

3. llama.cpp is the first speculative decoding engine.
   - Why: it already supports local GGUF inference and official draft-model speculative decoding flags.
   - What changed: settings support `llama_cpp.speculative_decoding`, `draft_model_path`, `draft_max`, `draft_min`, and `draft_p_min`.
   - Outcome: Quantumn can auto-start llama-server with a draft model when configured.

4. Speculative decoding is opt-in.
   - Why: it downloads a large-ish GGUF file and works best when draft/main tokenizers match.
   - What changed: `quantumn model --enable-speculative` explains the optimization and asks for confirmation before download/config changes.
   - Outcome: users can activate the speed path without surprising network or disk usage.

5. Default draft recommendation is Qwen2.5-Coder 0.5B Instruct GGUF Q5_K_M.
   - Why: it is code-oriented, GGUF-native, in the requested 100M-800M range, and small enough for old hardware compared with the main model.
   - Outcome: Quantumn uses it as a starter draft model for Qwen/Qwen-Coder main models. Users should choose a same-family tiny draft for Llama/Mistral mains.

## Activation Flow

Command:

```bash
quantumn model --enable-speculative
```

Flow:

1. Explain speculative decoding in one short prompt.
2. Tell the user the model to be downloaded and where it will be stored.
3. Ask yes/no.
4. If no: abort with no config changes.
5. If yes: download the draft GGUF, set provider to `llama_cpp`, enable auto-start and speculative decoding, and save config.

Automation:

```bash
quantumn model --enable-speculative --yes
```

## Result

The project now has a clearer performance posture:

- Fewer recurring system/tool tokens.
- Better alignment between router policy and tool prompting.
- A documented and implemented local inference speed path.
- Safer UX for network/download-heavy optimization.
