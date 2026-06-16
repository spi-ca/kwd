---
name: software-role-agents
description: Run a software-writing workflow with Pi subagents for user representative, systems engineer, designer, parallel-capable developer, implementer, QA, and reviewer roles.
---

# Software Role Agents

Use this skill to run software changes through a role-based Pi subagent workflow.

## When to Use

- New features, bug fixes, refactors, or documentation updates that benefit from separated requirements, design, implementation, QA, and review.
- Tasks where user intent, system constraints, implementation, and verification should be reported as distinct outputs.
- Work where kaniko invocation, container runtime behavior, registry/auth mounting, image-build environment assumptions, or deployment constraints matter.
- Situations where you want isolated `subagent` context per role.

## Project-Local Agent Frontmatter Policy

The project-local subagent setup expects these markdown frontmatter fields for role agents:

- `name`: agent name
- `description`: short agent purpose
- `model`: Pi model ID such as `openai-codex/gpt-5.4`
- `thinking`: one of `off`, `minimal`, `low`, `medium`, `high`, `xhigh`
- `tools`: comma-separated built-in tool list

Use a separate `thinking` field instead of encoding reasoning mode into the model ID.

## Role Responsibilities

### `user-representative`
- Preserve the user's original intent and success criteria.
- Summarize acceptance criteria, user-facing scenarios, and blockers.
- Identify ambiguity that prevents completion.

### `software-systems-engineer`
- Review OS, runtime, dependency, permission, process lifecycle, deployment, and operational constraints.
- Identify risks around performance, resource usage, portability, observability, and failure recovery.
- Ask for exact commands or missing environment evidence when feasibility depends on them.

### `software-designer`
- Turn requirements and system constraints into an implementable design.
- Define scope, boundaries, error semantics, and verification strategy.
- Produce a plan the implementer can execute.

### `software-developer`
- Implement one approved independent work package.
- Respect allowed files, ownership boundaries, and focused validation.
- Report blockers instead of guessing across package boundaries.
- Use edit-capable tools and focused validation appropriate for the assigned package.

### `software-implementer`
- Apply approved changes.
- Integrate parallel developer lane results when needed.
- Preserve user changes and existing behavior outside the approved scope.
- Avoid unapproved shortcuts, unfinished markers, dead code, duplicated logic, or hidden compatibility shims.

### `software-qa`
- Map acceptance criteria to commands, tests, smoke checks, or artifact inspection.
- Validate happy path, edge cases, and failure paths that matter to the task.
- Record reproducible findings and separate blocking from non-blocking issues.

### `software-reviewer`
- Review the diff, QA evidence, and requirement mapping.
- Check correctness, maintainability, security, performance, and documentation consistency.
- Perform the final completion audit.

## Role Input/Output Contract

| Role | Primary inputs | Primary outputs |
| --- | --- | --- |
| `user-representative` | User request, docs, current behavior evidence | Intent, acceptance criteria, user-facing scenarios, blockers |
| `software-systems-engineer` | Requirements output, repo/environment evidence, dependency/runtime info | Constraints, feasibility, operational risks, system-level verification |
| `software-designer` | Requirements and system outputs, code/doc structure | Design decisions, scope, implementation plan, verification strategy |
| `software-developer` | One approved package, allowed files, acceptance criteria, repo rules | Package changes, focused validation, parallel-safety notes, blockers |
| `software-implementer` | Approved plan, developer outputs, current files, repo rules | Integrated changes, summary, validation, blockers |
| `software-qa` | Acceptance criteria, diff, verification commands/environment | QA matrix, command/test/smoke evidence, findings |
| `software-reviewer` | User requirements, design, diff, QA results | Review findings, completion audit, ready/not-ready verdict |

## Recommended Subagent Chain

Use this chain for medium or larger tasks. The agent names below refer to project-local agents; if the active Pi subagent runtime exposes a project/both agent-scope selector, select it, but do not add unsupported fields to the JSON shape when the callable tool schema does not expose such a selector. Run the `parallel-development` stage only when the designer split the work into non-overlapping packages. If there is only one independent package, run a single `software-developer` lane. If the work cannot be split safely, skip that stage and let `software-implementer` handle the changes sequentially.

```json
{
  "chain": [
    {
      "label": "requirements",
      "agent": "user-representative",
      "task": "Summarize user intent, acceptance criteria, user-facing scenarios, constraints, and blockers for: <task>"
    },
    {
      "label": "system-constraints",
      "agent": "software-systems-engineer",
      "task": "Using the requirements output, inspect repository/environment constraints and provide system-level feasibility, risks, and verification for: <task>"
    },
    {
      "label": "design",
      "agent": "software-designer",
      "task": "Using requirements and system constraints, design the implementation plan and verification strategy for: <task>"
    },
    {
      "type": "parallel",
      "label": "parallel-development",
      "tasks": [
        {
          "agent": "software-developer",
          "task": "Implement independent work package A for: <task>. Include allowed files, acceptance criteria, preserved behavior, and focused validation."
        },
        {
          "agent": "software-developer",
          "task": "Implement independent work package B for: <task>. Include allowed files, acceptance criteria, preserved behavior, and focused validation."
        }
      ]
    },
    {
      "label": "implementation-merge",
      "agent": "software-implementer",
      "task": "Integrate developer lane results, resolve approved shared-file work, and run focused validation for: <task>"
    },
    {
      "type": "parallel",
      "label": "verification-review",
      "tasks": [
        {
          "agent": "software-qa",
          "task": "Validate the implementation against acceptance criteria for: <task>"
        },
        {
          "agent": "software-reviewer",
          "task": "Review the implementation, evidence, maintainability, and completion readiness for: <task>"
        }
      ]
    }
  ],
  "mode": "spawn"
}
```

## Single-Agent Fallback

If `subagent` execution is unavailable or the task is small, the root agent should perform the same sequence directly:

1. `user-representative`
2. `software-systems-engineer`
3. `software-designer`
4. `software-developer` when there is an independent package
5. `software-implementer`
6. `software-qa`
7. `software-reviewer`

## Quality Gates

Before finishing, confirm all of the following:

- Every applicable role has output, or the root agent reported that role's perspective for a small task.
- `software-developer` is used only for real independent work packages. If used, each lane reports allowed files, changed files, validation, and conflicts. If skipped, the reason is documented.
- Every explicit requirement is mapped to current evidence from files, command output, diffs, tests, logs, or artifacts.
- System constraints and operational risks were reviewed.
- Existing behavior and user changes outside the approved scope were preserved.
- Validation matched the change type. For `.pi`-only or documentation-only work, use focused checks such as frontmatter inspection, targeted grep, JSON parsing, and `git diff --check`. For code changes, add relevant build/test/lint verification.
- There are no unapproved shortcuts, unfinished markers, dead code, duplicated logic, hidden assumptions, or undocumented behavior changes.
- QA and reviewer outputs show no blocking issues.

## Output Template

```markdown
## Role Results

### user-representative
- Goal:
- Acceptance criteria:
- Blockers:

### software-systems-engineer
- System constraints:
- Operational risks:
- System verification:

### software-designer
- Design decisions:
- Files/scope:
- Verification strategy:

### software-developer
- Work packages:
- Parallel safety:
- Changed files:

### software-implementer
- Changed files:
- Key edits:
- Integration result:

### software-qa
- Commands:
- Results:

### software-reviewer
- Audit:
- Remaining issues:
- Completion status:
```

## Pitfalls

- Do not omit roles just to match a smaller output.
- Do not merge systems-engineer and designer responsibilities; the first provides environment constraints, the second chooses implementation structure.
- Do not merge QA and reviewer responsibilities; they may run in parallel, but their outputs stay separate.
- Do not stop at planning when implementation and verification are still pending.
- If validation fails, fix the cause when possible and re-run the relevant checks instead of reporting stale failures as complete.
