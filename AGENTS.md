# Agent rules

## Pull requests

- A PR counts as green without the Release PR check (`.github/workflows/release-pr.yaml`). Wait for it only when you
  need its build artifacts (the beta binaries or image it comments on the PR) or must verify that the build passes.
