---
name: /openspec-to-beads
id: openspec-to-beads
category: OpenSpec
description: Use this command after an OpenSpec change is approved.
---

## What to do

1. Ask the user (if not provided) which OpenSpec change ID to use.
2. Read:
   - `openspec/changes/<id>/proposal.md`
   - `openspec/changes/<id>/tasks.md`
   - any `openspec/changes/<id>/specs/**/spec.md`
3. Based on these files:
   - Create a Beads epic with `bd create "Implement <feature-name>" --type epic --priority 0 --description "<epic description>"`.
   - For each concrete implementation step in `tasks.md`, create a child task:
     - `bd create "<task title>" --type task --parent <epic-id> --priority 0 or 1 --description "<task description>"`.
   - Add dependencies using `bd dep add <child-id> <parent-id>`:
     - migrations & infra → block backend
     - backend → block UI
     - implementation → block release/docs
4. Run `bd ready` and summarize which tasks are now ready to start.
5. Print:
   - epic ID
   - all created task IDs
   - a short explanation of the dependency graph.
