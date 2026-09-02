# Development Workflow

## Repository
One Git monorepo. Keep `main` stable; use `development`, `feature/*`, `fix/*`, and `docs/*` as needed.

## AI workflow
1. Read docs.
2. Inspect existing code.
3. Identify applicable decisions.
4. Propose the smallest change.
5. Implement only requested scope.
6. Test.
7. Update docs if behavior changed.

Do not ask Antigravity to build the entire product in one prompt.

## Stitch/Antigravity
Stitch = UI authority. Antigravity = implementation authority.

## Done
Implementation + tests + documentation + security/privacy review + migrations where required + no secrets.
