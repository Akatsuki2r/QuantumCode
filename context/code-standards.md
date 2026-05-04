# Code Standards

## Performance First

Quantumn Code is built for budget plans and older hardware. Every feature should reduce one of these costs: prompt tokens, prefill time, generation latency, memory usage, disk churn, or user round trips.

## Prompt Standards

- Keep system prompts compact, direct, and project-aware.
- Avoid persona filler, repeated safety prose, and broad tutorials.
- Spend tokens on operating policy: inspect, edit narrowly, verify, report.
- Route first, then inject only the mode and tools needed for that task.
- Tool descriptions should be short schema hints, not paragraphs.

## Tool Standards

- Tool lists must respect router policy.
- Do not advertise blocked tools in the active prompt.
- Prefer search/glob before broad file reads.
- Prefer targeted verification over full-suite runs unless risk requires it.
- Keep tool result summaries small enough to feed back into the model.

## Local Inference Standards

- llama.cpp optimizations should be opt-in when they download files, start processes, or change persistent config.
- Speculative decoding should use a draft model that is much smaller than the main model and ideally shares tokenizer/model family.
- Defaults should favor stable, documented llama.cpp flags.
- Never silently download models. Prompt first unless the user passes an explicit non-interactive flag such as `--yes`.

## Documentation Standards

- Update README for user-facing setup.
- Update `context/project-state.md` for decisions and outcomes.
- Update `context/trajectory.md` when the architecture direction changes.
