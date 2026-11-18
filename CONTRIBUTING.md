# Contributing to Relay

This checkout is being prepared for a source preview; publication is not yet approved. Read README.md, SECURITY.md, LICENSING.md and THIRD_PARTY_NOTICES.md first.

For changes to approved MPL-covered source, contribute material you have rights to provide under MPL-2.0, retain SPDX/copyright notices, and identify any third-party material and its terms. Contributors retain copyright; no assignment or additional contributor agreement is implied. Do not copy code or artwork without permission. Disclose relevant provenance, including generated material and references, so it can be reviewed. Separate branding assets are not covered merely because source that renders them is MPL.

Keep changes focused. Include the problem, implementation and relevant test results in a proposed change. Use the locked Cargo dependency graph; explain dependency/license changes. Follow the build and test commands in README.md, including Android builds when affected and `python3 tools/check_licenses.py`. Never bypass OS capture consent, certificate verification or secure pairing for a test. Device instrumentation tests can change trust state; use a dedicated test setup.

Do not include private keys, real pairing links/PINs, personal screenshots, production logs, local device identifiers, or generated binaries. Use controlled sample content. The public development signing password is not a production signing policy; keystores always stay private. Review the exact files before staging rather than adding the entire worktree blindly.

Review Git author and committer metadata before contributing publicly. Maintainer commits require the owner's explicitly approved public identity and GitHub noreply email; do not infer or configure that identity without approval. Do not rewrite another contributor's history without approval.

Report security problems privately as described in SECURITY.md. If a private reporting channel is not available, request one without posting vulnerability details or credentials publicly.
