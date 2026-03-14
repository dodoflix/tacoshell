# ci-status — Fetch GitHub Actions Results & Diagnose Failures

Fetches the latest GitHub Actions workflow runs for the **current branch** (or
a specified PR/branch/run), shows pass/fail status per job, and pulls the
relevant log lines for any failures so you can diagnose and fix them.

## How to use

```
/ci-status                  # latest run on current branch
/ci-status 18               # latest run for PR #18's branch
/ci-status run 15234567890  # specific run ID
```

## GitHub repo

`dodoflix/tacoshell` on `api.github.com`

---

## Step-by-step instructions

### 1. Determine the branch

If an explicit run ID was given, skip to step 3.

If a PR number was given, resolve its head branch:

```bash
curl -s "https://api.github.com/repos/dodoflix/tacoshell/pulls/<PR>" \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['head']['ref'])"
```

Otherwise use the current branch:

```bash
git rev-parse --abbrev-ref HEAD
```

### 2. Find the latest workflow run for that branch

```bash
BRANCH=<branch>
curl -s "https://api.github.com/repos/dodoflix/tacoshell/actions/runs?branch=${BRANCH}&per_page=5" \
  | python3 -c "
import sys, json
data = json.load(sys.stdin)
for r in data.get('workflow_runs', []):
    print(r['id'], r['name'], r['status'], r['conclusion'], r['html_url'])
"
```

Pick the most recent run (first entry). Note its `id`.

### 3. List jobs for that run

```bash
RUN_ID=<run_id>
curl -s "https://api.github.com/repos/dodoflix/tacoshell/actions/runs/${RUN_ID}/jobs?per_page=50" \
  | python3 -c "
import sys, json
data = json.load(sys.stdin)
for j in data.get('jobs', []):
    status    = j['status']
    conclusion= j.get('conclusion') or 'in_progress'
    name      = j['name']
    jid       = j['id']
    icon      = '✓' if conclusion == 'success' else ('✗' if conclusion == 'failure' else '·')
    print(f'{icon} [{jid}] {name} — {conclusion}')
    if conclusion == 'failure':
        for step in j.get('steps', []):
            if step.get('conclusion') == 'failure':
                print(f'      FAILED STEP: {step[\"name\"]}')
"
```

### 4. Fetch logs for each failed job

For every job with `conclusion == 'failure'`:

```bash
JOB_ID=<job_id>
curl -sL "https://api.github.com/repos/dodoflix/tacoshell/actions/jobs/${JOB_ID}/logs" 2>&1 \
  | grep -E "(error|Error|FAILED|fatal|warning|Warning|##\[)" \
  | tail -80
```

If the log redirect requires following, add `-L` (already included above).

Note: GitHub returns a redirect to a pre-signed Azure Blob URL for logs. `curl -sL` follows it automatically.

### 5. Diagnose

Print a structured report:

```
RUN #<id> — <workflow name> — <conclusion>
Branch: <branch>
Triggered: <created_at>

JOBS:
  ✓ Lint
  ✗ Rust Tests (ubuntu-latest)
      FAILED STEP: cargo test
  ✓ TypeScript Tests
  · Build Desktop (skipped)

FAILURE DETAILS — Rust Tests (ubuntu-latest):
  <relevant log lines>
```

### 6. Fix (if asked or obviously actionable)

If the failure is in this repo's code (not a flaky network issue or missing
secret), read the relevant file(s) and apply a fix following the project's
TDD and no-suppression rules.

If the failure needs a secret or external config, report it clearly and stop.

### 7. Commit & push (if fixes were made)

```bash
git add <changed files>
git commit -m "fix: resolve CI failure in <job name>\n\n<what was wrong and how it was fixed>\n\nhttps://claude.ai/code/session_..."
git push -u origin <branch>
```

---

## Useful API reference

| What | Endpoint |
|------|----------|
| List runs for branch | `GET /repos/dodoflix/tacoshell/actions/runs?branch=<branch>&per_page=5` |
| Single run | `GET /repos/dodoflix/tacoshell/actions/runs/<run_id>` |
| Jobs for run | `GET /repos/dodoflix/tacoshell/actions/runs/<run_id>/jobs?per_page=50` |
| Job logs (redirects) | `GET /repos/dodoflix/tacoshell/actions/jobs/<job_id>/logs` |
| PR head branch | `GET /repos/dodoflix/tacoshell/pulls/<pr>` → `.head.ref` |

All endpoints are unauthenticated for public repos. If rate-limited (HTTP 403
with `X-RateLimit-Remaining: 0`), wait and retry or add `-H "Authorization:
token <GITHUB_TOKEN>"` using a token from the environment.

## Notes

- Always check **all failed jobs**, not just the first one.
- Log output from `curl -sL` on the jobs/logs endpoint may be plain text or
  URL-encoded — pipe through `python3 -c "import sys; print(sys.stdin.read())"` if needed.
- Flaky failures (network timeouts, Docker pull failures) should be noted but
  not "fixed" in code — report them to the user instead.
