# Custom fork release integration

Keep the official upstream history separate from custom work without rewriting
published commits. Preserve uncommitted changes and ongoing worktrees.

The `origin` remote is the user's fork. The `upstream` remote must point to
`https://github.com/openai/codex.git`. Verify that URL before fetching; never
replace a differently configured remote without investigating it.

For a new clone, add the official remote if it is absent:

```sh
git remote add -t main upstream https://github.com/openai/codex.git
```

Retain official release tags as remote-tracking refs, using this additional
fetch mapping once per clone (do not duplicate an existing value):

```sh
git config --add remote.upstream.fetch '+refs/tags/rust-v*:refs/remotes/upstream/releases/rust-v*'
```

For each requested release, use the assigned task worktree and fetch the exact
official tag into the corresponding `refs/remotes/upstream/releases/` ref before
merging it. For example:

```sh
RELEASE_TAG=rust-v0.166.0
git fetch --no-tags upstream "refs/tags/${RELEASE_TAG}:refs/remotes/upstream/releases/${RELEASE_TAG}"
git merge --no-ff "refs/remotes/upstream/releases/${RELEASE_TAG}"
```

Do not squash upstream releases. The retained ref lets Phrase Automation's Git
counters exclude official ancestors while continuing to count unpublished fork
changes. A local tag alone is not a remote-tracking ref and does not provide
that distinction. Keep these release refs during remote-ref cleanup.

Resolve conflicts while preserving the custom features, run the relevant
regressions, and deliver the task branch into local `main` through the selected
worktree workflow. Inspect the runtime-source diff against the previously
installed revision before deciding whether a rebuild is required. Do not merge
old build branches solely to retain their ancestry; keep archival history on
backup tags or in a bundle instead. Never push unless the user asks.
