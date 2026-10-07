# Fork modification notices and source delivery

This is the concat-song maintenance process, introduced on 2026-10-07. It
supports the obligations in the actual [AGPL license](../LICENSE); it is not
legal advice or a substitute for that license. [FORK-NOTICE.md](../FORK-NOTICE.md)
is the prominent fork-level notice and [MODIFICATIONS.json](../MODIFICATIONS.json)
is the dated file inventory. The pinned upstream identity is recorded in both.

## Keep the underlying notices

AGPL section 4 requires preservation of the applicable copyright, license,
additional-term, and no-warranty notices and provision of a license copy.
Section 5(a) requires prominent modification notices with a relevant date;
5(b) requires prominent notice of this license and applicable section 7 terms.
The license does not prescribe this JSON format or a comment on every file:
those are the fork's checkable maintenance conventions.

Do not replace or erase upstream SPDX/copyright headers when adding a fork
notice. Keep `LICENSE` byte-identical. Retain `LICENSE-EXCEPTIONS.md`,
`TRADEMARK.md`, `THIRD_PARTY_NOTICES.md`, and applicable component notices; adding
a notice is not relicensing. The plugin exception is retained on its existing
terms, including its independent-module and public-interface boundaries.
It is not a general exception for copied engine code or internal-crate links.

The upstream commercial-licensing and CLA statements preserved in upstream
documents describe upstream's arrangement; they do not establish a fork CLA,
copyright assignment, signing mandate, or authority to commercially relicense
other contributors' work. Follow this fork's `CONTRIBUTING.md` for contributions.

## Record an actual change

1. Make and review the intended edits. Preserve existing SPDX/copyright lines.
   In hand-edited commentable source/configuration, add a syntax-valid comment
   within the first 40 lines, after the shebang and existing legal header:

   ```text
   // Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.
   ```

   Use `#` or another valid comment delimiter for the file's language. When
   changing that file again, update the notice's date to the relevant date.
   A descriptive form is also accepted: `Modified by the concat-song fork on
   2026-10-07: describe the change.` The date must match that file's index entry.
   Do not append comments to JSON, binary data, generated output, or LICENSE.

2. From the repository root, record the reviewed working-tree diff:

   ```sh
   python scripts/check_modification_notices.py record --date 2026-10-07 --summary "Describe the reviewed change"
   python scripts/check_modification_notices.py check
   python -m unittest discover -s scripts/tests -p 'test_modification_notices.py'
   ```

   Use the actual relevant modification date, not the upstream date or a
   guessed publication date. The recorder cannot establish that historical
   fact for you. It writes only the index and refuses missing inline notices
   or lost SPDX/copyright lines; it does not edit source, stage, commit, fetch,
   install, or publish. Unchanged entries keep their dates and descriptions.
   If several paths need different descriptions/dates, record each logical
   batch before editing the next. Inspect the resulting diff before committing.

3. Keep `FORK-NOTICE.md` accurate when the fork's scope changes, then record
   that edit too. Include both notice files in source archives and releases.

The checkout needs the pinned upstream commit locally; CI should fetch full
history (`fetch-depth: 0`) or explicitly fetch that commit. A missing base is
an unavailable check, never a pass. The tool uses local Git only.

## What the checker can establish

Every current added, modified, mode-changed, or deleted path relative to the
pinned base has an entry. Renames are represented as deletion plus addition.
This includes staged and unstaged tracked changes and non-ignored untracked
files, so remove unrelated scratch files before recording. Ignored untracked
files are outside this inventory; do not distribute them merely because this
check passes. Submodules are rejected rather than silently omitted.

Each normal entry has a checkout-normalized Git blob fingerprint, file mode,
upstream blob/mode when applicable, a relevant ISO date, a description, and the
notice location. Git's configured clean/text conversion makes line endings
consistent with versioned blobs. Custom clean filters are therefore part of
the accounting environment and must be reviewed if introduced.

Deleted entries have no current blob. The index's own entry has no Git blob;
instead `index_sha256` hashes the canonical UTF-8, sorted-key JSON with only
`index_sha256` replaced by `null`. This avoids a recursive self-hash while
covering its dates, descriptions, entries, and upstream identity. The hash
detects accidental staleness, not malicious rewriting or authenticity.

Binary/non-UTF-8 content, symlinks, prose/non-commentable formats, and generated
files use the central notice/index. Byte-for-byte upstream copies relocated to
another path are also recorded centrally without changing their original
headers; any subsequent content edit follows the ordinary inline-notice rule.
`Cargo.lock` and files marked generated near
their start are treated as generated. Review that classification; a generator
marker is not permission to call hand-written source generated. Other listed
source/configuration suffixes require the dated inline sentence above.

The checker catches missing/stale entries, mismatched contents or modes, invalid
or missing dates, missing inline notices, and removal of existing SPDX/copyright
lines in retained changed files. It accepts only the exact upstream license
bytes in LF or CRLF checkout form, since Git may convert line endings across
hosts; leave the existing checkout file untouched.
It does not understand every legal notice, validate comment syntax, prove the
truth of a date/description, check trademark similarity, or audit third-party
dependencies. Review deleted components' notices and distributed assets by hand.

## Distribution and remote operation remain separate work

Before distributing object code, select and satisfy an applicable AGPL section
6 route for Corresponding Source. For a network download under 6(d), provide
equivalent source access and clear directions next to the binary download;
include needed build/control materials and any applicable Installation
Information. A generic repository link is not evidence that it matches the
binary, includes all required material, or remains available as required.

Before allowing users to interact remotely with a modified version, address
section 13: prominently offer those users no-charge access to the Corresponding
Source of the version actually running. Choose a reachable source-offer surface
for the real API/service deployment and verify it. Local stdio tooling and a
public repository do not automatically prove a remote deployment's obligations
are fulfilled. Retained extension hooks do not promise GUI behavior or UI tests.

Before a branded release, review the preserved trademark policy, use a distinct
product name/icon, and make fork status visible. The repository handle is not
an upstream endorsement. A passing accounting check is only one release input;
source-delivery, notice packaging, dependency, and branding review remain owned
by whoever publishes or operates that particular version.
