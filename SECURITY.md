# Security policy

## Reporting a vulnerability

Report privately via GitHub's **Security → Report a vulnerability** (private
vulnerability reporting) on this repository. You should receive a response within a
week. Please do not open a public issue for anything you believe is exploitable.

## Supported versions

The latest published release of each crate (`spec-lock`, `boundary-spec-macros`,
`boundary-enforce`, `boundary-spec`) is supported. There are no maintenance branches:
fixes ship forward, through the same certification pipeline as everything else.

## What counts as a vulnerability here

These crates run at **build and test time** in consumers' trees (a build.rs
enforcement pass, a test-time drift gate), not in production data paths. The threat
model is accordingly about the development loop:

- anything that lets a crafted source tree, spec file, register, or export escape the
  parser and execute or exfiltrate at a consumer's build time;
- anything that makes a drift gate pass when it should refuse (a forged "fresh" lock
  is an integrity failure — the gates are the product);
- path traversal through spec/lock/register paths supplied by configuration.

Discovery running arbitrary consumer code in-process is by design (it is the
consumer's own code, in the consumer's own build); that is not a report.

## Verifying what you install

Releases are automatic: every fully-certified default-branch tree publishes itself
(see `.github/release.sh`), and release notes carry the ratified spec-lock diff plus a
fingerprint of it ("signed by change"). The repository state behind a published
package is addressable via its release tag.
