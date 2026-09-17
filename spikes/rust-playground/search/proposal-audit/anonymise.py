#!/usr/bin/env python3
"""Regenerate search/designs/design-{a,b}.md from the two proposals: the
provenance paragraph, the titled heading and the `## Repairs` table removed,
nothing else changed.
Input: brainstorming-notes/24-*, 25-*. Run from spikes/rust-playground,
after the repair pass and before the reviews. The A/B mapping is a coin
flip recorded in note 23; reviewers never read this directory."""
import glob, os, re

def strip(path, letter, keep_title):
    lines = open(path).read().split("\n")
    out, i = [], 0
    while i < len(lines):
        if lines[i].startswith("Provenance:"):
            while i < len(lines) and lines[i].strip():
                i += 1
            continue
        m = re.match(r"# \d+ — (.*)", lines[i])
        if m:
            title = m.group(1) if keep_title else ""
            out.append(f"# Design {letter.upper()}" + (f" — {title}" if title else ""))
            i += 1
            continue
        out.append(lines[i]); i += 1
    text = re.sub(r"\n{3,}", "\n\n", "\n".join(out)).lstrip("\n")
    # The repair table is the audit's trace; reviewers never see the audit.
    text = text.split("\n## Repairs")[0].rstrip("\n") + "\n"
    for word in ("Fable", "Astra", "Claude", "Codex", "OpenAI", "Anthropic"):
        assert word.lower() not in text.lower(), (path, word)
    os.makedirs("search/designs", exist_ok=True)
    open(f"search/designs/design-{letter}.md", "w").write(text)

strip(glob.glob("brainstorming-notes/25-*.md")[0], "a", keep_title=True)
strip(glob.glob("brainstorming-notes/24-*.md")[0], "b", keep_title=False)
print("wrote search/designs/design-a.md, design-b.md")
