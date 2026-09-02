# Error Handling

Errors must be observable, recoverable when possible, non-destructive, and explicit when user action is required.

Collector errors should degrade gracefully. DB errors must not discard finalized sessions. Sync errors retry with backoff. Auth errors request reauthentication. Permission errors explain required access. Schema errors should stop endless retries.

Never fabricate timestamps or silently discard activity.
