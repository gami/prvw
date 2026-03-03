# /release — Bump version, tag, and trigger release build

Perform the following steps in order. Stop immediately if any step fails and report the error.

## 1. Determine version

- Read the current version from `package.json`
- If `$ARGUMENTS` is provided (e.g. `patch`, `minor`, `major`, or an explicit version like `0.3.0`), use it to determine the new version
- If no argument, default to `patch` bump
- Confirm the new version with the user before proceeding

## 2. Lint & Test

Run `npm run check` and `npm run test`. All must pass.

## 3. Bump version in all 3 files

Update the version string in:
- `package.json` (`"version"` field)
- `src-tauri/Cargo.toml` (`version` field under `[package]`)
- `src-tauri/tauri.conf.json` (`"version"` field)

Then run `npm install --package-lock-only` to sync `package-lock.json`.

## 4. Commit

Stage the 4 changed files and commit with message: `Bump version to vX.Y.Z`

## 5. Tag and push

- Create an annotated git tag: `git tag -a vX.Y.Z -m "vX.Y.Z"`
- Push the commit and tag: `git push && git push origin vX.Y.Z`

This triggers the `release.yml` workflow which builds for macOS (aarch64 + x86_64), Linux, and Windows.

## 6. Verify

Run `gh run list --workflow=release.yml -L 1` to confirm the release build started, and show the user the Actions URL.
