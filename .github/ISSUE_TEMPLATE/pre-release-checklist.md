---
name: Pre-release-checklist
about: Checklist to ensure new release works without corrupting the chain
title: "Pre-release Checklist: CHAIN VERSION"
---

## Client Checklist

- [ ] Verify client `Cargo.toml` version has been incremented since the last release.
  - Current version: XXX
  - Last version: XXX

## Runtime Checklist

- [ ] Verify runtime `Cargo.toml` version has been incremented since the last release.
  - Current version: XXX
  - Last version: XXX
- [ ] Verify runtime `spec_version` has been incremented since the last release.
  - Current version: XXX
  - Last version: XXX
- [ ] Compatible with previous client.
- [ ] Use `try-runtime` to test migrations.
- [ ] Use `fork-off-substrate` to ensure the chain is not bricked after the upgrade

## Ecosystem

- [ ] Bholdus.js