# Analytics Specification

## Weekly metrics
Total tracked, foreground, interaction, idle, media, productive, neutral, distracting, uncategorized time; top apps/sites; context switches; longest sessions; deep-work sessions; week-over-week changes.

## Definitions
Context switch = transition between normalized activities. Deep work is configurable and should not use an arbitrary universal threshold.

## Calendar Boundaries
Because the tracking engine records continuous sessions (e.g., spanning 23:30 to 01:30) without splitting at midnight, the Analytics layer must split durations across calendar days, weeks, and months dynamically when generating reports.

## Browser analytics
Keep semantic categories such as YouTube video/Shorts and Instagram Reels/general Instagram separate when rules can identify them.

## Cross-device
Never merge raw activity sessions from different devices during ingestion. The dashboard must show device-specific time and clearly label aggregate views. The analytics layer calculates optional deduplicated human-time estimates, but must never silently double-count or destructively deduplicate the underlying overlapping sessions.
