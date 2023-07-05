# Steps to Perform in the Event of a Release

- [ ] Ensure local `main` is up-to-date with `origin/main`
- [ ] Run `cargo update`
- [ ] Run `cargo test`
- [ ] Create a new branch: `release-X-Y-Z`
- [ ] Run `git diff` between current commit and previous tagged release commit
  - [ ] Check which crates have been modified; these will need their versions bumped
  - [ ] If sub-crates, e.g., `serde_json_path_core`, have their version bumped, check their super-crates, e.g., `serde_json_path`, for dependency update
- [ ] Move Unreleased changes into the new version header in `serde_json_path/CHANGELOG.md`
- [ ] Commit changes and push to `origin/main`
- [ ] Open a pull request to merge changes into `main`, and allow CI to run successfully
- [ ] Merge the PR and jump back to `main` locally
- [ ] For each crate, in sub-crate to super-crate order, publish the crates from the workspace that had their versions bumped:
  - [ ] Run `cargo publish -p <crate name> —dry-run`, to check that all is good
  - [ ] Run `cargo publish -p <crate name>`, to do the actual release
