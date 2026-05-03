


# State database

The state database contains the list of projects, tasks, and rounds. The state database supports several backends:

- A local file storage. Its location is in the ADJENT_HOME/state
- An SQL database, like postgresql


## Local file storage

```
state/projects/[project-id]/tasks/[task-id]/rounds/[round-id]/
                                                             /inputs/
                                                             /inputs/input.md
                                                             /inputs/plan.md
                                                             /outputs/
                                                             /outputs/plan.md
                                                             /logs/
```

# Artifacts store

The artifacts store contains all the files referenced by the tasks. It supports several backends:

- A local file storage when the state database is also a local storage.
- An object storage like s3.