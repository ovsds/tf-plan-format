# Don't wait for the Release PR check by default

- Context: [AGENTS.md § Pull requests](../../AGENTS.md#pull-requests)
- Source: conversation 2026-10-19 - "You don't need to wait for release PR by default, only if you want to get builded
  artefact or ensure build. Save to repos context", then "tf-plan-format only"
- Supersedes: -

A PR is green without the Release PR check; agents wait for it only to use its beta binaries or image, or to verify the
build.

Why: the check builds the release binaries and image on every PR touching the build inputs, which is slow, and its
result rarely matters to a PR beyond the build itself.
