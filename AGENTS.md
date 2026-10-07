# Working on concat-song

This is an independent fork of `jub0t/concat`, not an official Concat build.
The fork targets an agent-only engine/API. Preserve useful engine and API
extension hooks for later presentation integrations; do not restore or promise
a conventional GUI, or claim UI testing that was not performed.

## Make the requested change

- Respect the current task's scope and other contributors' uncommitted work.
  Inspect affected code and tests before editing; do not revert unrelated edits.
- Keep the change as small as the real behavior permits. Test the affected
  headless behavior and report unavailable tools or unrun tests plainly.
- Do not infer permission to commit, push, publish, install, deploy, or contact
  people from these instructions. Follow the current user's authorization.
- Keep private environment, account, project, and local-path details out of
  public source and documentation.

## Preserve and record notices

Read [FORK-NOTICE.md](FORK-NOTICE.md) and
[docs/licensing-process.md](docs/licensing-process.md) before changing licensed
source. Preserve upstream copyright, SPDX, license, warranty, third-party, and
trademark notices. `LICENSE` must remain byte-identical. Do not introduce a CLA,
copyright-assignment, or signing mandate, or imply upstream endorsement.

Add a dated, syntax-valid `Modified for concat-song on YYYY-MM-DD; see
FORK-NOTICE.md.` comment near the top of hand-edited commentable source/config
files, keeping existing legal headers. Generated, binary, deleted, and
non-commentable files are accounted for centrally, not with invalid comments.
After all intended edits, use:

```sh
python scripts/check_modification_notices.py record --date YYYY-MM-DD --summary "Describe this change"
python scripts/check_modification_notices.py check
python -m unittest discover -s scripts/tests -p 'test_modification_notices.py'
```

Supply the actual relevant date and review the index diff; do not invent dates
or treat the summary as proof. Recording only updates the index. A passing check
means the scoped accounting checks passed, not legal compliance, a complete
Corresponding Source offer, trademark clearance, or authorization to release.
Raise an actual conflict or missing decision with the task owner and preserve
the evidence instead of silently widening the work.
