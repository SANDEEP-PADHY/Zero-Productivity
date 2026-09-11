# Analytics Specification

## Calendar Boundaries & Timezones
Because the tracking engine records continuous sessions (e.g., spanning 23:30 to 01:30) without splitting at midnight, the Analytics layer splits durations across calendar days dynamically when generating reports.
- **Timezone Semantics**: Calendar boundaries are calculated exactly against the explicitly requested timezone (e.g. `America/New_York`), rather than system local time or raw UTC, preserving correct semantic days for the user's location.
- **DST Handling**: Daylight Saving Time transitions (e.g. spring forward skipping 02:00, fall back repeating 01:00) must be handled safely without panic. Sessions that cross DST boundaries split smoothly at the correct localized absolute midnight.

## Range-Overlap & Splitting Behavior
- **Range Intersection**: Querying sessions within a `[start, end]` range strictly intersects sessions. Sessions spanning outside the range are retrieved but *must* be clamped explicitly to the `[start, end]` query boundaries.
- **Proportional Component Splitting**: When a session is split across days or clamped to a range, its `duration_ms` and dimensional metrics (`foreground_ms`, `interaction_ms`, `media_ms`, `idle_ms`) must be allocated proportionally to the time intersecting the slice.
- **Rounding Behavior**: Proportional allocations are rounded to the nearest millisecond. To prevent floating point arithmetic from losing or fabricating milliseconds across multiple chunks, the final chunk of a split session MUST be allocated the exact mathematical remainder (`remaining_ms = total_ms - sum(allocated_ms_of_previous_chunks)`).

## Metric Dimension Semantics
- Semantic dimensions (`foreground`, `interaction`, `media`, `idle`) must remain fully independent. The analytics engine must not subtract them from one another (e.g. `foreground` minus `media`) unless explicitly calculating a derived UI metric. They are distinct tracking facets.

## Classification Source of Truth
- **Immutability**: Analytics are purely read-only derived metrics. `get_sessions_in_range` directly parses the immutable string/integer fields persisted in SQLite.
- **Source of Truth**: The analytics engine does *not* rerun the `RulesEngine`. Re-classifying historical sessions requires a separate explicit data migration job; analytics will blindly aggregate the natively stored `Classification` and `ActivityType`.

## Aggregation & Identity
- **Application Aggregation**: Grouping is performed by the deterministic `application_id`. It must not group purely by raw window titles or PIDs, as those fluctuate.
- **Domain Aggregation**: Grouping uses the exact `domain` string. It must not infer domains from raw titles.
- **Multi-Device**: The engine correctly aggregates identical identities natively across devices while retaining the raw components as separate `ClassifiedSession`s. Devices can be explicitly filtered via the `device_ids` parameter in the range query.
- **Deterministic Ranking**: Top applications and domains must be ordered deterministically: `total_ms DESC`, then `identity ASC`. Longest sessions must be ordered by `duration_ms DESC`, then `session_id ASC`.

## Context Switching
- Context switches are counted strictly as the boundaries between distinct continuous sessions (after chronological sorting) where the semantic identity (`application_id` or `domain`) changes from the previous session. Title changes within the same app/domain identity do not constitute a context switch.
