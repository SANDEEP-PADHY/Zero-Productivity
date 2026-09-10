# Rules Engine

## Rule types
1. Exclusion
2. Normalization
3. Semantic activity detection
4. Classification

## Historical Semantics

Rules apply **only** at the time of classification when an activity session is finalized. 

Changing a rule does **NOT** retroactively reclassify previously finalized sessions. This guarantees immutable temporal truth and prevents historical drift of finalized productivity scores. For example, if `youtube.com` was classified as `Neutral` on January 1st, and the user changes it to `Distracting` on January 2nd, the sessions from January 1st remain `Neutral`.

## Default Fallback Semantics

If no rules match an observation, the `RulesEngine` assigns a default `ActivityType` based on the normalized identity:
- `NormalizedIdentity::Application`: Defaults to `ActivityType::Application`
- `NormalizedIdentity::Browser`: 
  - If a domain or URL is present (indicating an active web tab), it defaults to `ActivityType::Website`.
  - If no domain or URL is present (indicating a bare browser window, e.g. when tracking is disabled or during startup), it defaults to `ActivityType::Browser`.

The `Classification` defaults to `Neutral`.

## Precedence

Rules are applied with a strict deterministic evaluation loop. Precedence is resolved based on the following order: `Explicit user rule -> specific built-in rule -> generic built-in rule -> uncategorized`

Implementation specifically resolves ties and orders evaluation by:
1. **Scope**: `Device > Account > System` (Device/Account = Explicit user rules, System = Built-in rules)
2. **Priority**: Higher integer wins (Specific > Generic within the same scope)
3. **Tie-breaker**: Lexicographical sort by Rule ID (`a.id.cmp(&b.id)`) to guarantee determinism.

## Match fields
Application ID/executable, browser, domain, URL pattern, path, title pattern, activity type.

## Semantic examples
YouTube `/watch` = video; `/shorts` = Shorts. Instagram `/reels` = Reels; other pages = general Instagram. These are examples, not universal productivity judgments.

## Classification
`productive | neutral | distracting | uncategorized`. User definitions override defaults.

## Implementation
Keep rules data-driven and indexed/compiled where practical; collectors should not contain classification policy.
