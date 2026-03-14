# review-pr — Fetch & Fix PR Review Comments

Fetches all review comments (inline code comments) and general PR comments from
GitHub for the **current branch's PR**, then evaluates and fixes the valid ones.

## How to use

```
/review-pr          # auto-detect PR from current branch
/review-pr 18       # explicit PR number
```

## GitHub repo

`dodoflix/tacoshell` on `api.github.com`

## Step-by-step instructions

### 1. Determine the PR number

If an argument is given, use it directly.

Otherwise, run:

```bash
git rev-parse --abbrev-ref HEAD
```

Then find the PR number from the remote PR refs:

```bash
git ls-remote origin 'refs/pull/*/head' | grep $(git rev-parse HEAD) | grep -oP 'refs/pull/\K[0-9]+'
```

If that doesn't match (e.g. new commits since last push), list all open PR refs
and match on branch name via:

```bash
git ls-remote origin 'refs/pull/*/head'
```

Cross-reference with local branch name using:

```bash
git branch -r | grep <branch-name>
```

### 2. Fetch inline code review comments

```bash
curl -s "https://api.github.com/repos/dodoflix/tacoshell/pulls/<PR>/comments" \
  | python3 -c "
import sys, json
comments = json.load(sys.stdin)
for c in comments:
    user = c['user']['login']
    path = c.get('path', '')
    line = c.get('line') or c.get('original_line', '')
    body = c.get('body', '')
    cid  = c.get('id', '')
    print(f'ID:{cid} | {user} | {path}:{line}')
    print(body)
    print('---')
"
```

### 3. Fetch general PR (issue) comments

```bash
curl -s "https://api.github.com/repos/dodoflix/tacoshell/issues/<PR>/comments" \
  | python3 -c "
import sys, json
comments = json.load(sys.stdin)
for c in comments:
    user = c['user']['login']
    body = c.get('body', '')
    cid  = c.get('id', '')
    print(f'ID:{cid} | {user} | (general comment)')
    print(body)
    print('---')
"
```

### 4. Evaluate each comment

For every comment, decide:

| Verdict | Criteria | Action |
|---------|----------|--------|
| **Fix** | Technically correct — real bug, missing dep, wrong type, security issue, logic error | Read the referenced file, apply the fix |
| **Skip** | Style preference with no correctness impact, already addressed, or out of date | Note it, move on |
| **Discuss** | Ambiguous — needs user input | Surface it to the user |

Key rule: **never suppress a warning to silence a comment** — always fix the
underlying issue.

### 5. Apply fixes

For each comment being fixed:
- Read the file at the referenced path
- Make the minimal, correct change
- Do not reformat unrelated lines

### 6. Commit and push

After all fixes are applied:

```bash
git add <changed files>
git commit -m "fix: address PR review comments\n\n<bullet list of what was fixed>\n\nhttps://claude.ai/code/session_..."
git push -u origin <branch>
```

### 7. Report

Print a summary table:

```
FIXED:
  - packages/ui/package.json — replaced happy-dom with jsdom devDependency (Copilot)

SKIPPED:
  - (none)
```

## Notes

- The GitHub API for public repos doesn't require auth for read-only access —
  no token is needed unless the repo is private or rate-limited.
- Both endpoint types must be checked: `/pulls/<n>/comments` (inline) and
  `/issues/<n>/comments` (general thread).
- Always re-read the file before editing — never edit blind.
