

# CLI commands

## Project management

```
adjent project list
adjent project create [project-id]
adjent project activate [project-id]
```


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

## HTTP server

### Start the server

```
adjent server start -p port
```

starts the adjent HTTP server. 
The server default configuration is read from ADJENT_HOME/adjent-config.toml

### Stop the server

```
adjent server stop
```

## MCP server

```
adjent mcp start -u [adjent-endpoint] -p [project-id]
```

Starts the MCP server in stdio mode. The server must be provided with the HTTP API endpoint of the adjent server, and the project-id to listen to.

