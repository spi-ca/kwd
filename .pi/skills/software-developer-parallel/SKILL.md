---
name: software-developer-parallel
description: Split approved software work into independent packages and run parallel software-developer subagents safely.
---

# Software Developer Parallel

Use this skill to split approved implementation work into independent packages and run multiple `software-developer` subagents in parallel when it is actually safe.

## When to Use

- The implementation spans separate files, modules, prompts, skills, tests, or docs with clear ownership boundaries.
- Each lane can have explicit allowed files, acceptance criteria, and focused validation.
- You want to extend the `software-role-agents` workflow with parallel implementation after design approval.

Do not use it when:

- Most changes touch the same files.
- Requirements or design decisions are still unclear.
- One package depends on another package's output.

## Agent Configuration

`software-developer` lanes should use:

- Model: `openai-codex/gpt-5.4`
- Thinking: `high`
- Tools: `read`, `grep`, `find`, `ls`, `bash`, `edit`, `write`

Use the full edit-capable developer configuration because merge conflicts, validation failures, and package-boundary decisions are often more expensive than the initial code change.

## Procedure

1. Read the request, approved design, repository instructions, and current user changes.
2. Split candidate work by file/module/responsibility.
3. For each package, define:
   - goal and acceptance criteria
   - allowed files or directories
   - behavior that must stay unchanged
   - focused validation commands
   - files that must not be shared with other lanes
4. If file scopes overlap, do not parallelize that overlap. Move it into a sequential merge step.
5. Run only independent packages through one `subagent` parallel call. The `software-developer` lanes are project-local agents; if the active Pi subagent runtime exposes a project/both agent-scope selector, select it, but do not add unsupported fields to the JSON shape when the callable tool schema does not expose such a selector.
6. Collect lane diffs, validation, blockers, and conflict reports.
7. Perform shared-file integration and final cleanup in the root agent or a sequential `software-implementer` step.
8. Run `software-qa` and `software-reviewer` for verification.
9. If findings remain, update the package split or fix plan and rerun only the needed lanes.
10. Finish only when every requirement is mapped to current evidence and no blocking findings remain.

## Parallel Subagent Template

```json
{
  "tasks": [
    {
      "agent": "software-developer",
      "task": "Implement work package A for <task>. Allowed files: <paths>. Acceptance criteria: <criteria>. Preserve existing behavior outside this package. Run focused validation: <commands>. Report blockers and parallel-safety conflicts."
    },
    {
      "agent": "software-developer",
      "task": "Implement work package B for <task>. Allowed files: <paths>. Acceptance criteria: <criteria>. Preserve existing behavior outside this package. Run focused validation: <commands>. Report blockers and parallel-safety conflicts."
    }
  ],
  "mode": "spawn"
}
```

## Integration with the Role Workflow

1. `user-representative`: summarize goals and acceptance criteria.
2. `software-systems-engineer`: review runtime and operational constraints.
3. `software-designer`: define independent work packages.
4. `software-developer` lanes: implement those packages in parallel.
5. Root merge step or `software-implementer`: integrate shared files and rerun focused validation.
6. `software-qa` and `software-reviewer`: verify the result.

## Quality Gates

- Each lane's actual changed files stay within its allowed files.
- No two lanes modify the same file.
- Every change is traceable to an approved package goal.
- Focused validation and repo-level validation have fresh command output.
- QA and review have no blocking findings.
- There are no unapproved shortcuts, unfinished markers, dead code, duplicated logic, compatibility shims, hidden assumptions, or undocumented behavior changes.

## Output Template

```markdown
## Parallel Developer Plan

| Package | Agent | Allowed files | Acceptance criteria | Validation |
| --- | --- | --- | --- | --- |
| ... | `software-developer` | ... | ... | ... |

## Lane Results
- Package:
  - Files changed:
  - Validation:
  - Blockers/conflicts:

## Merge and Review
- Integrated files:
- QA evidence:
- Review verdict:
- Remaining findings/blockers:
```

## Pitfalls

- Do not force parallelization by inventing weak boundaries.
- Do not assign the same file to multiple lanes.
- Do not treat one lane's success as proof that all lanes are valid.
- Do not bypass project-local subagent confirmation requirements.
- If validation fails, fix and rerun when possible instead of stopping at a failure report.
