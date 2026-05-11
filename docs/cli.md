
# CLI commands

The CLI commands get the current context of action, and call the web-server to do the actual actions.
The context is retrieved from the execution context of the CLI, based on the current directory.

By comparing the current directory against the base directory of adjent, the CLI knows what is the 
current project/task/round. For example, if the base directory of adjent is "/home/alice/.adjent", and the current directory where the CLI is executed is "/home/alice/.adjent/state/projects/foo/tasks/20260606-update-docs/rounds/0/", then:

- The project is "foo"
- The task is "20260606-update-docs"
- The round is "0"

If the CLI is executed outside of the base directory, then explicit parameters or active values must be available for the command to be accepted.

## Project management

### List projects

```bash
adjent project list
```

This command lists all available projects.

### Create a new project

```bash
adjent project create [project-id]
```

This command creates a new project named `project-id`.

### Activate a project

```bash
adjent project activate [project-id]
```

This command sets the project `project-id` as the default project.


## Task management

```bash
adjent task -p [project-id] list
adjent task -p [project-id] activate [task-id]
adjent task -p [project-id] create [task-id] 
```

## Round management

```bash
adjent round -p [project-id] -t [task-id] activate [round-id]
```

### Bumps round to next one

```bash
adjent round -p [project-id] -t [task-id] bump --from [round-id]
```

The `--from` argument is optional. When not provided, the current round is used as the source.
When no round exists, then round 0 is created.

Bumping a round creates a new round where the id is increased by 1. 
Moreover:

- The "inputs", "outputs", and "logs" artifacts directories are created in the new round.
- The input artifacts of the current round are copied as input artifacts of the new round.
- The outputs artifacts of the current round are copied as input artifacts of the new round, overriding any existing file from the previous step.
- An empty input artifact named "instructions.md" is created (or overridden when it already exists).

### Add an input artifact

```bash
adjent round -p [project-id] -t [task-id] -r [round-id] add input [artifact]
```


### Schedule the round for work

```bash
adjent round -p [project-id] -t [task-id] -r [round-id] do [action]
```

`action` is one of the supported skills of the agent listening for incoming work:

- `refine-use-case`
- `develop`

This schedules the round to be processed by a manager listening to the project.

## Manager

### Start the manager

```bash
adjent manage --project [project-id] --agent "[agent-command]"
```

The manager is a long-running process that polls the server for pending actions in a project.
When an action is available, it:
1. Marks the action as `assigned` and then `running`.
2. Spawns the `agent-command` as a sub-process.
3. Injects the following environment variables into the agent's process:
    - `ADJENT_PROJECT_ID`: The project ID.
    - `ADJENT_TASK_ID`: The task ID.
    - `ADJENT_ROUND_ID`: The round ID.
    - `ADJENT_ACTION_ID`: The unique ID of the action.
    - `ADJENT_ACTION`: The name of the action to perform.
4. Waits for the agent to finish and updates the action status to `completed` or `failed` based on the exit code.

## Server

### Start the server

```bash
adjent server start -p [port]
```

Starts the adjent HTTP and MCP server. 
The server default configuration is read from `ADJENT_HOME/adjent-config.toml`.

### Stop the server

```bash
adjent server stop
```
