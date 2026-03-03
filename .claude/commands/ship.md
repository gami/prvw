# /ship — Commit, Push, and Create PR

Perform the following steps in order. Stop immediately if any step fails and report the error.

## 1. Self-review

Run `git diff` (staged + unstaged) and review all changes:
- Look for bugs, typos, leftover debug code, security issues
- Check that no secrets (.env, credentials) are included
- If you find issues, fix them before proceeding

## 2. Lint & Format

Run `npm run check` (biome + tsc + clippy + rustfmt). Fix any errors automatically and re-run until clean.

## 3. Test

Run `npm run test` (cargo test + vitest). All tests must pass. If tests fail, fix the issue and re-run.

## 4. Commit

- Stage all relevant files (avoid secrets/credentials)
- Write a concise commit message in English describing the changes
- Use a HEREDOC for the commit message

## 5. Push

- If on `main`, create a new branch first with a descriptive name
- Push with `-u` to set upstream

## 6. Create PR

- Create a PR with `gh pr create`
- Title: short (under 70 chars), English
- Body: Japanese, with `## Summary` (bullet points) and `## Test plan` (checklist)
- Return the PR URL when done

If the argument `$ARGUMENTS` is provided, use it as context for the commit message and PR description.
