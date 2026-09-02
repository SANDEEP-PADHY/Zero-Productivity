# Rules Engine

## Rule types
1. Exclusion
2. Normalization
3. Semantic activity detection
4. Classification

## Precedence
`Explicit user rule -> specific built-in rule -> generic built-in rule -> uncategorized`

## Match fields
Application ID/executable, browser, domain, URL pattern, path, title pattern, activity type.

## Semantic examples
YouTube `/watch` = video; `/shorts` = Shorts. Instagram `/reels` = Reels; other pages = general Instagram. These are examples, not universal productivity judgments.

## Classification
`productive | neutral | distracting | uncategorized`. User definitions override defaults.

## Implementation
Keep rules data-driven and indexed/compiled where practical; collectors should not contain classification policy.
