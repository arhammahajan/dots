# Vendored skill catalog

One skill catalog is vendored from upstream, copied file-for-file so every
harness sees the same content. Edit it here only to fix breakage; otherwise
re-sync from upstream so local changes are not lost on the next copy.

| Set | Upstream | Commit |
| --- | --- | --- |
| pstack skills (58) | https://github.com/backnotprop/pstack | `9cd6939b766cb920f1c44970d7c7ee1ec28bd0d6` (2026-11-14) |

pstack is MIT licensed; the license text sits beside this file as
`LICENSE-pstack`.

## What was copied

- pstack: all of `skills/` (58), plus `agents/poteto-agent.md`, which is placed
  at `poteto-mode/agents/poteto-agent.md` because pstack's subagent is only
  meaningful alongside `poteto-mode`.

Not copied: pstack's README, `skills/deprecated`, `skills/in-progress`,
`skills/misc`, `automations/`, `docs/`, and `assets/`. Those are repository
furniture, unstable drafts, or host-specific material, not installable skills.

## Local deviations

- pstack's `tdd` and `teach` were installed as `pstack-tdd` and `pstack-teach`
  to avoid colliding with the Matt Pocock skills of the same name, which have
  since been removed. Front matter and inbound references in
  `poteto-help/SKILL.md` and `poteto-mode/playbooks/bug-fix.md` were updated to
  match.
- Internal path repairs inside the vendored set, of the form "a reference
  written for the upstream root or skill root did not resolve from the file that
  contains it". In pstack, playbook cross-links (`playbooks/x.md` inside
  `poteto-mode/playbooks/`) now resolve within that directory, and
  `why/references/synthesizer-prompt.md` points at its sibling `epistemics.md`.
  In `poteto-help/SKILL.md`, the nine links to upstream `README.md` and `docs/`
  files that were never vendored now point at the source repository on GitHub.
  No prose was reworded.

Nothing else was changed.

## Re-syncing

Download the tarball, then re-run the same copy with the two pstack renames
(`tdd` → `pstack-tdd`, `teach` → `pstack-teach`) and re-apply the reference
fixes. Diff against the previous commit before committing:

```sh
git clone --depth 1 https://github.com/backnotprop/pstack /tmp/pstack
```
