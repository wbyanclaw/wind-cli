# Release Process

This document describes the correct workflow for releasing a new version of wind-cli.

## Prerequisites

- Write access to the `wbyanclaw/wind-cli` repository
- Git configured with your credentials (`gh auth login`)
- Rust toolchain installed (`rustup default stable`)

## Release Workflow

### Step 1: Ensure main branch is ready

```bash
git checkout main
git pull origin main
```

Make sure all desired changes are merged. The CI will automatically run the full test suite on tag push — running locally first is optional but recommended to catch obvious issues:

```bash
cargo test --all-targets
cargo clippy --all-targets
cargo build --release
```

All tests should pass before tagging.

### Step 2: Tag the release

**Important**: Create an **annotated tag** (not a lightweight tag), and push the **tag** (not a branch).

```bash
# Example: releasing v0.2.2
git tag -a v0.2.2 -m "Release v0.2.2"
git push origin v0.2.2
```

**Do NOT create a branch named `v0.2.x`**. The CI workflow triggers on tag pushes (`refs/tags/v*`), not branch pushes.

### Step 3: Watch the CI

The GitHub Actions workflow `.github/workflows/windows-release.yml` will automatically:

1. Trigger on the tag push
2. Build `windcli.exe` for Windows x86_64
3. Run smoke tests and unit tests
4. Compute SHA256 of the built artifacts
5. Create/update the GitHub Release
6. Upload `windcli.exe`, `windcli-windows-x86_64.zip`, `install.ps1`, and `SHA256SUMS.txt`

Monitor the run at:
```
https://github.com/wbyanclaw/wind-cli/actions
```

### Step 4: Verify the release

After CI completes:

1. Check all 4 assets are present in the release:
   - `windcli.exe`
   - `windcli-windows-x86_64.zip`
   - `install.ps1`
   - `SHA256SUMS.txt`

2. Verify SHA256SUMS.txt contains correct filenames:
   ```
   <hash>  windcli.exe
   <hash>  windcli-windows-x86_64.zip
   ```

3. Run the install script on a Windows machine:
   ```powershell
   irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 -OutFile $env:TEMP\windcli-install.ps1
   powershell -NoProfile -ExecutionPolicy Bypass -File $env:TEMP\windcli-install.ps1
   windcli --version
   ```

4. Update `docs/release-notes-windows.md` with changes for this release.

## Common Mistakes

### ❌ Wrong: Creating a branch instead of a tag

```bash
# WRONG — this creates a branch, not a tag
git checkout -b v0.2.2
git push origin v0.2.2
```

This will trigger the CI on a branch push, but the release upload step (`gh release upload`) requires a release to already exist. The binary will be built but not properly attached to a release.

### ❌ Wrong: Manually creating the release via GitHub UI

If you create the release manually via GitHub's web UI **before** CI runs, CI will try to `gh release edit` it. The binary will be uploaded correctly, but the SHA256 will be correct **only if** you let CI compute it. Always let CI handle the full release creation.

### ✅ Correct: Tag → push → CI handles everything

```bash
git tag -a v0.2.2 -m "Release v0.2.2"
git push origin v0.2.2
```

The CI workflow detects the tag, creates the release, computes SHA256, and uploads all assets automatically.

## Hotfix Process

For an urgent hotfix (e.g., v0.2.1 → v0.2.2):

1. Create a fix branch from the tag: `git checkout -b fix/description v0.2.1`
2. Apply the fix and commit
3. Tag and push following Step 2 above

## Release cadence

- Aim for small, incremental releases
- Each release should have a clear changelog entry in `docs/release-notes-windows.md`
- Version numbers follow [Semantic Versioning](https://semver.org/)
