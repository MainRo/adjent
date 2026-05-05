

# CLI commands

The CLI commands get the current context of action, and call the web-server to do the actual actions.
The context is retrieved from the execution context of the CLI, based on the current directory.

By comparing the current directory against the base directory of adjent, the CLI knows what is the 
current project/task/round. For example, if the base directory of adjent is "/home/alice", and the current directory where the CLI is executed is "/home/alice/state/projects/foo/tasks/20260606-update-docs/rounds/0/", then:

- The project is "foo"
- The task is "20260606-update-docs"
- The round is "0"

If the CLI is executed outside of the base directory, then explicit parameters or active values must be available. for the command to be accepted.

## Project management

### List projects

```
adjent project list
'''

This command lists all available projects.

### Create a new project

```
adjent project create [project-id]
```

This command creates a new project named "project-id"

### Activate a project

```
adjent project activate [project-id]
```

This command sets the project "project-id" as the default project.


## Task management

```
adjent task -p [project-id] list
adjent task -p [project-id] activate [task-id]
adjent task -p [project-id] create [task-id] 
```

## Round management

```
adjent round -p [project-id] -t [task-id] activate [round-id]
````

### bump to a new round

```
adjent round -p [project-id] -t [task-id] bump --from [round-id]
```

The --from argument is optional, and used only to start a new branch from a specific round.
This command automatically activates the new created round.

```
adjent round -p [project-id] -t [task-id]  -r [round-id] add input [artifact]
```

When the state database is not local, this adds an artifact to the inputs of the round.


### Schedule the round for work

```
adjent round -p [project-id] -t [task-id]  -r [round-id] do [action]
```

action is one of the supported skill of the agent listening for incoming work:

- plan
- implement
- review



Mark the round as ready to be processed by an agent. 

## server

### Start the server

```
adjent server start -p port
```

starts the adjent HTTP and MCP server. 
The server default configuration is read from ADJENT_HOME/adjent-config.toml

### Stop the server

```
adjent server stop
```
