---
name: iterative-findings-loop
description: Repeat implementation or documentation updates through subagent QA and review until findings and blockers are cleared with current evidence.
---

# Iterative Findings Loop

Use this skill when implementation or documentation work needs one or more QA/review/fix cycles before it is actually complete.

## When to Use

- A subagent workflow produced QA or review findings that need follow-up changes.
- The task is likely to require more than one pass across code, prompts, skills, docs, or tests.
- Completion requires every explicit requirement to be mapped to current evidence.

## Procedure

1. Restate the completion criteria from the current request and latest approved plan.
2. Classify findings as blocking, non-blocking, or unclear.
3. Keep implementation, QA, and review outputs separate so fresh evidence is easy to compare.
4. Split required fixes into work packages again.
   - If file scopes are independent, rerun targeted `software-developer` lanes.
   - If the issue is in shared files or integration, use a sequential `software-implementer` pass.
5. Run focused validation immediately after each fix.
6. Rerun `software-qa` and `software-reviewer`, preferably in parallel.
7. If new findings appear, update the root cause and impact scope, then repeat only the needed stages.
8. Stop only when all blockers are cleared and any remaining non-blocking notes are explicitly acceptable.
9. Before final reporting, map every explicit requirement to current evidence from files, diffs, commands, tests, or review output.

## Parallel Guidance

- Use parallel execution only when file scopes and decisions do not overlap.
- QA and review should run in parallel whenever possible.
- Documentation and code fixes may run in separate lanes only if they do not touch the same files.

## Pitfalls

- Do not list findings without feeding them back into a fix loop.
- Do not mark the task complete while validation is still failing.
- Do not assign the same shared file to multiple fix lanes.
- Do not treat stale QA/review results as current evidence.
- Do not hide unresolved blockers by softening the wording.

## Verification

- The latest QA result has no blocking issues.
- The latest review result does not request changes.
- Every explicit requirement maps to current evidence.
- Required code/document/test changes are present in the current diff.
