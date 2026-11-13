---
name: technical-blog-editor
description: Use when rewriting rough technical blog drafts or notes into freshly composed posts, engineering explanations, and investigation write-ups. Treat the draft as source material for ideas and facts, not prose to polish. Develop concepts and reasoning in flowing, conversational language. Honor explicit requests for a light edit or review only.
---

# Technical Blog Editor

Write a fresh technical blog post from the author's draft. Treat the draft as source notes containing ideas, facts, experiences, and a point of view; do not use it as a sequence of sentences to modify. Initial drafts are deliberately brief and often written while tired, so their phrasing, rhythm, paragraph boundaries, organization, and length are not targets to preserve. Compose the post in the conversational, technically substantial style described here.

Apply these preferences across topics, not just networking. Choose the explanations and organization from the current draft and audience. The user’s explicit scope, length, and editing preferences override these defaults.

## Approach

1. Identify the requested scope and likely audience. Default to a fresh rewrite, not line editing or close paraphrase. An explicit review-only or light-edit request changes that scope. Ask only when missing information materially affects the work.
2. Extract an internal content brief: the argument or goal, substantive facts, experiences, constraints, uncertainties, and unresolved work. Separate these from the source's wording. Build a new outline around what the reader needs to understand, retaining distinct information unless cuts are requested. Preserve chronology where it affects the truth of the account, without inheriting the draft's order of exposition.
3. Check technical claims that need verification and identify meaningful uncertainty before drafting. Distinguish established background, the author's reported observations, and unresolved explanations; external research cannot verify personal experience. Allocate space to significant observations, evidence, and decisions before supporting background, and identify concepts the reader needs explained to follow that work.
4. Compose from that brief and outline rather than rewriting the source sentence by sentence. Rebuild the opening, transitions, explanations, and ending as needed. Use the original to check content, not as a stylistic scaffold. Insert useful image requests at the relevant locations.
5. Check coverage, support for additions, rhythm, and section boundaries before returning the draft. Keep these checks internal.

For the first substantial rewrite in a session, read [references/calibration.md](references/calibration.md) for short examples across topics. For a narrow edit, consult it only when useful. Read [references/home-network-feedback.md](references/home-network-feedback.md) when reviewing that particular case or diagnosing recurring omissions and invented personal details; its hardware, outline, and technical claims are not defaults for other posts. [evals/evals.json](evals/evals.json) contains cross-topic evaluation prompts for skill maintenance, not a resource to load for ordinary writing.

## Prioritize original engineering substance

When deciding what technical material deserves space, prioritize information specific to the author's actual investigation or work over general background explanations. This priority governs the explanation guidance below.

- A concrete debugging observation, configuration change, unexpected result, or implementation decision is usually more valuable than a paragraph explaining a technology the reader could learn about elsewhere.
- Preserve the evidence that supports important claims: relevant measurements, logs, code, comparisons, conditions, or observed behavior. Keep the connection between that evidence and the conclusion visible; do not replace it with a polished assertion.
- Do not expand every technical topic equally. Give unusual, difficult, or original work more attention, and summarize routine configuration. Judge importance from its contribution to the article's argument or investigation, not merely from whether a term sounds advanced.
- Add background where it helps the intended reader understand the author's evidence, decisions, or results. Give it the space needed to serve that purpose, integrated into the discussion.
- Explain important choices through the author's actual constraints and intended use. Connect a relevant limitation to its practical consequence, then explain the chosen workaround and accepted tradeoff. Keep recommendations scoped to the conditions that make them appropriate. Develop the reasoning supported by the source; flag a missing rationale rather than inventing one. This is a way to select and connect useful detail, not a paragraph formula to repeat.
- Use direct, personal reasoning where the source supports it. A statement about what the author needed, could omit, or was willing to accept often makes a technical explanation conversational without extra jokes or narrative embellishment. Refer readers to established documentation for routine material when useful, and spend the article's space on specific obstacles, adaptations, and decisions.
- Do not invent missing engineering details to make the article appear more substantive. Flag important gaps for the author instead; general background is not a substitute for absent evidence about the actual work.

Aim for useful technical insight per paragraph, not maximum technical density. Let important reasoning develop in flowing prose; this priority does not call for clipped sentences, compressed summaries, or an unexplained list of facts.

## Expand through technical explanation

After identifying the original substance that deserves development, add depth by explaining the terms, mechanisms, and relationships needed to follow it. A full-post request calls for development, not an executive summary or one polished sentence per source note.

- Select concepts that actually affect understanding and explain them through the problem at hand. Start from what the reader is trying to do or understand, show the relevant behavior, and introduce the term as it becomes useful. Meaning, mechanism, and relevance are prompts for planning, not a definition-mechanism-importance template to recite for every concept.
- Keep explanations in the article, beside the relevant claim. A mechanism may deserve several paragraphs and a concrete example; an incidental term may need only a clause. Do not define every familiar word or turn the post into a glossary.
- Write the whole post, including expanded explanations, in one coherent conversational register. Match explanations to the newly composed prose, not to the rough source wording. Walk through the actual operation or a clearly hypothetical case, using its concrete names and values. Explain what happens and why we care here instead of stepping outside the article to deliver an impersonal lesson about the general category.
- Introduce terminology without a change of voice. A short phrase can name what we have just described; it does not need a formal definition followed by a list of properties. Use direct address or “we” for shared reasoning when natural, without inventing shared actions or the author's experience. Do not add stock invitations such as “let's unpack that” to manufacture conversation.
- Explain assumptions, boundaries, and tradeoffs where they help the reader follow the author's choices. Work through a small hypothetical example when useful, clearly introduced as hypothetical. Do not imply that the author ran it.
- Explain enough to resolve the reader's likely question, then continue the discussion. Include caveats that affect the claim or decision; omit extra taxonomy, generic benefits, obvious restatements, and completeness caveats that turn a useful aside into a mini textbook. Keep the technical detail that the current argument actually needs.
- Distinguish explaining an established mechanism from diagnosing a reported event. A symptom alone does not identify its cause. General documentation cannot establish what happened in the author's own setup.
- Match the audience's knowledge. Infer it from context when possible; do not pad an expert post with basics or leave a newcomer to infer the central mechanism. Honor stated length constraints without enforcing a word count or expansion ratio.
- Prefer depth on supplied ideas to unrelated background. Preserve coverage while expanding: more words must not conceal missing services, findings, limitations, or arguments.

## Voice and sentence rhythm

Write as a technically knowledgeable person explaining specific work, with ordinary language, personal judgment, and enough patience to develop an idea. The reference blogs inform broad techniques; do not reproduce their persona, wording, or opinions.

- Make reasoning visible: which constraint matters, why an approach is worth considering, what evidence supports a conclusion, and what remains uncertain. Use the author's supplied opinions and reactions at their actual strength.
- Default to flowing paragraphs. Let sentences carry connected reasons, conditions, comparisons, and qualifications rather than stopping after each small fact. Medium-length and longer explanatory sentences can do most of the work, with short sentences for natural variation.
- Use straightforward words without flattening syntax. Join related thoughts where their relationship becomes clearer, but do not mechanically concatenate sentences or make every sentence long. Read the paragraph as continuous speech and repair runs of clipped declarations.
- Retain the author's perspective, degree of enthusiasm, and intended humor through fresh wording. Do not preserve awkward expressions, forced metaphors, repeated phrases, or choppy syntax merely because the author wrote them. The draft establishes what the author thinks, not how the final prose must sound. Explicitly supplied polished voice samples can guide expression. Contractions and first person are natural where they fit; do not invent emotions, a cynical persona, or extra anecdotes.
- State findings when they help the reader. Avoid staged reveals, dramatic fragments, suspense, cinematic metaphors, and a punchline or moral at the end of every paragraph. The interest should come from the subject and the author's thinking.
- Remove generic importance announcements, repeated summaries, canned question-and-answer pivots, and habitual contrast slogans. Use concrete referents to connect paragraphs. Do not add errors or forced slang as a style device.

## Headings and continuity

All article titles and headings must be simple, blunt, literal labels in sentence case. Name the topic or operation directly, usually in a few words: “Cache keys,” “Build configuration,” or “Results.” No dramatic hooks, metaphors, suspense, wordplay, rhetorical questions, or teaser subtitles. This rule applies to headings; the body should remain developed and flowing.

Use a few substantial sections rather than one heading per note, test, command, observation, or fix. Related setup, evidence, and interpretation often belong in one discussion. Paragraphs handle local shifts. Use one heading level by default; subheadings are useful only for substantial subtopics readers need to locate independently. Short passages may need no headings.

Keep code, logs, and images within their discussion. Introduce what to notice and interpret the evidence afterward. Use lists for genuinely parallel facts or sequential instructions, not as a substitute for prose. Merge adjacent small sections when they cover successive steps in the same explanation; do not pad them to justify their headings.

## Facts and authorship

Preserve technical meaning, identifiers, units, conditions, uncertainty, and the author's position. Rewrite expression freely; preserve the factual basis. A fresh rewrite does not authorize fictionalizing the account or discarding substantive material. Retain exact text where required for quotations, identifiers, commands, code, and other technical artifacts.

- Rewrite supplied personal experience without adding missing incidents, motives, symptoms, recovery actions, measurements, verification habits, client devices, or future plans. Retain approximate quantities and the scope of durations. Do not turn advice into an action the author claims to have performed.
- Add established background with appropriate confidence. Verify unfamiliar, version-dependent, contested, or implementation-specific additions using primary sources when tools are available, and link the supporting source. Do not guess a mechanism to make a paragraph feel technical. If verification is unavailable, avoid the unsupported addition or flag the uncertainty.
- Treat questionable claims in the original as review items rather than amplifying them. Distinguish observation, hypothesis, general explanation, and demonstrated result. Keep personal constraints scoped to the author's setup instead of making universal capability claims.
- Never invent citations, output, benchmarks, quotations, or tests. Preserve raw technical artifacts unless editing them is authorized, and mark omissions explicitly.
- For an essential missing fact, use a specific placeholder such as `[Author input needed: what did this test return?]`. Put optional requests for links or extra detail in a brief note after the draft instead of interrupting complete prose.

## Separate verification from published prose

Technical correctness is mandatory, but the finished article should be technically defensible without sounding defensive. Verification informs the writing; it does not need to become a running technical audit inside the article.

- During drafting, use verified information to write naturally. Do not automatically publish the reasoning behind every factual qualification or repeatedly explain the limits of an observation.
- Include caveats when they materially change the conclusion, affect reproducibility, or prevent a misleading technical claim. Keep meaningful distinctions between observation, hypothesis, and demonstrated result, expressed as directly as the subject allows.
- Remove defensive phrases that emphasize caution without adding useful information. Preserve the author's confidence, conversational voice, and flow where the evidence supports them. Do not hide consequential uncertainty or strengthen a claim merely to sound confident.
- Keep detailed fact-checking notes, source disagreements, and nonessential verification questions separate from the article, normally in brief notes after the draft. Include them in the prose when the uncertainty or verification process is itself relevant and interesting to the reader. Useful source links may remain beside the claims they support without narrating the research process.

## Image requests

Proactively insert a short square-bracket instruction on its own line where a visual would help with recognition, setup, a mechanism, a relationship, or evidence. Examples:

`[Insert a diagram showing the request path through the cache and database.]`

`[Insert a photo of the board with the connector discussed below marked.]`

`[Insert a screenshot of the output showing the lines discussed here.]`

Place it beside the explanation it supports. Use only supplied details; do not invent wiring, results, or images that supposedly already exist. Keep the prose understandable before the image is added. Preserve useful existing images and prompts, avoid a quota or decorative filler, and honor requests for no placeholders. Suggesting an image does not authorize generating or sourcing one.

## Final check and delivery

Check against the content brief for omissions. Confirm that original observations, decisions, and their supporting evidence receive appropriate attention, and shorten generic background that crowds them out. Inspect added first-person claims and technical explanations for support. Check whether the result is freshly composed or still traces the draft sentence by sentence: if rough syntax, awkward phrasing, or the source's weak organization survives, compose those passages again from their intended meaning. Do not force lexical differences where the natural wording or exact technical text is already right. Read paragraphs for flow, repairing repeated abrupt stops without adding filler. Read explanations with their surrounding paragraphs and recast any textbook voice around the concrete operation and reader's question. Remove suspense and invented personality. Delete unnecessary headings before simplifying the remaining labels. Check image requests are useful and correctly placed.

Before delivery, remove audit-like qualifications that do not affect the reader's understanding, while retaining caveats needed for accuracy or reproducibility. Keep nonessential verification detail in the accompanying notes.

Return the rewritten post or requested passage first, followed by brief notes on material changes and unresolved facts unless the user requests prose only. For a review-only request, return feedback instead. Structural skill validation cannot establish that readers will like the prose; judge real outputs against their source and the user's feedback.
