# Vendored skill catalogs

Most skills in this directory are maintained here. Three whole catalogs are
vendored from upstream, copied file-for-file so every harness sees the same
content. Edit them here only to fix breakage; otherwise re-sync from upstream so
local changes are not lost on the next copy.

| Set | Upstream | Commit |
| --- | --- | --- |
| Matt Pocock skills (27) | https://github.com/mattpocock/skills | `5863806dd4e15c63174bee4e7515a4256792ecfa` (2026-11-12) |
| pstack skills (58) | https://github.com/backnotprop/pstack | `9cd6939b766cb920f1c44970d7c7ee1ec28bd0d6` (2026-11-14) |
| Addy Osmani skills (25) | https://github.com/addyosmani/agent-skills | `39b723ceebf81474d0557c1fecd01eddab585563` (2026-11-14) |

All three are MIT licensed; the license texts sit beside this file as
`LICENSE-mattpocock-skills`, `LICENSE-pstack`, and
`LICENSE-addyosmani-agent-skills`.

## What was copied

- Matt Pocock: `skills/engineering` (20) and `skills/productivity` (7), including
  their `agents/openai.yaml` files and bundled scripts.
- pstack: all of `skills/` (58), plus `agents/poteto-agent.md`, which is placed
  at `poteto-mode/agents/poteto-agent.md` because pstack's subagent is only
  meaningful alongside `poteto-mode`.
- Addy Osmani: all of `skills/` (25), plus `references/`, which is vendored at
  `harnesses/shared/references/` so its citations resolve unchanged.

Not copied: each repo's README files, `skills/deprecated`,
`skills/in-progress`, `skills/misc`, pstack's `automations/`, `docs/`, and
`assets/`, and Addy's `agents/`, `commands/`, `hooks/`, `evals/`, and `docs/`.
Those are repository furniture, unstable drafts, or host-specific material
(personas, slash-command TOMLs, hooks), not installable skills.

## Shared references outside this directory

Addy Osmani's skills reach for seven checklists that live one level above any
single skill, with paths like `../../references/definition-of-done.md`. Those
files are vendored unchanged at `harnesses/shared/references/`, which is exactly
where `../../` lands from `harnesses/shared/skills/<skill>/`. Both the real path
and the symlinks that `setup.sh` creates resolve there, so no relative path had
to be rewritten. Editing those checklists edits them for every skill that cites
them.

pstack's `../references/...` paths are unrelated: they point at
`poteto-mode/references/`, not at this directory, and only look similar.

## Local deviations

- pstack's `tdd` and `teach` collide with the Matt Pocock skills of the same
  name. Matt Pocock's keep the bare names; pstack's are installed as
  `pstack-tdd` and `pstack-teach`. Front matter and inbound references in
  `poteto-help/SKILL.md` and `poteto-mode/playbooks/bug-fix.md` were updated to
  match.
- Addy Osmani's set needed no renames: no skill name collides with the other two
  catalogs or with the skills maintained here.
- Internal path repairs inside the vendored sets, all of the form "a reference
  written for the upstream root or skill root did not resolve from the file that
  contains it". In pstack, playbook cross-links (`playbooks/x.md` inside
  `poteto-mode/playbooks/`) now resolve within that directory, and
  `why/references/synthesizer-prompt.md` points at its sibling `epistemics.md`. In
  `poteto-help/SKILL.md`, the nine links to upstream `README.md` and `docs/`
  files that were never vendored now point at the source repository on GitHub. In
  Addy Osmani's `spec-driven-development/SKILL.md`, three catalog-root links
  (`skills/<name>/SKILL.md`) became sibling links. No prose was reworded.

Nothing else was changed.

## Re-syncing

Download the tarballs, then re-run the same copy with the two pstack renames
(`tdd` → `pstack-tdd`, `teach` → `pstack-teach`) and re-apply the reference
fixes. Copy `agent-skills/references/` to `harnesses/shared/references/` too; the
citations in Addy's skills already point there, so nothing else changes. Diff
against the previous commit before committing:

```sh
git clone --depth 1 https://github.com/mattpocock/skills /tmp/mp
git clone --depth 1 https://github.com/backnotprop/pstack /tmp/pstack
git clone --depth 1 https://github.com/addyosmani/agent-skills /tmp/agent-skills
```
