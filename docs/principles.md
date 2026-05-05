
This document explains the principles of the different design elements of adjent.



# Local state

```
state/projects/[project-id]/tasks/[task-id]/rounds/[round-id]/
                                                             /inputs/
                                                             /inputs/input.md
                                                             /inputs/plan.md
                                                             /outputs/
                                                             /outputs/plan.md
                                                             /logs/
```


# Remote State

The state database contains the list of projects, tasks, and rounds. The state database supports several backends:

- A local file storage. Its location is in the ADJENT_HOME/state
- An SQL database, like postgresql


## Local file storage



# Artifacts store

The artifacts store contains all the files referenced by the tasks, categorized into `inputs`, `outputs`, and `logs`. It supports several backends:

- A local file storage when the state database is also a local storage.
- An object storage like s3.

While the CLI focuses on human-provided `inputs`, the Web API supports management of all artifact types to allow agents (via the MCP server) to update `outputs` and `logs`.
