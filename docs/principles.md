
This document explains the principles of the different design elements of adjent.


# Local state

The path of the local state is normative. It determines the existing projects, tasks, and rounds.
Each round is a directory containing different artifacts:

```
state/projects/[project-id]/tasks/[task-id]/rounds/[round-id]/
                                                             /inputs/
                                                             /inputs/input.md
                                                             /inputs/plan.md
                                                             /outputs/
                                                             /outputs/plan.md
                                                             /logs/
```

The local state contains all the artifacts referenced by the tasks, categorized into `inputs`, `outputs`, and `logs`. 

While the CLI focuses on human-provided `inputs`, the Web API supports management of all artifact types to allow agents to update `outputs` and `logs` via the MCP server.
