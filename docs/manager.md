# Adjent Manager

The manager is responsible for spawning an agent each time a new action is triggered.
The manager is started on a specific project. It then waits for actions via the HTTP server.

Each time a new action is received, the manager spawns the agent (for example claude code or gemini cli). The agent is provided as an argument of the `adjent manage` command. Once an agent is spawned, the manager waits until its completion. Then it waits again for another action to be available.

## Architecture

```mermaid
sequenceDiagram
    participant CLI
    participant Server
    participant Manager
    participant Agent

    CLI->>Server: POST /projects/:p/tasks/:t/rounds/:r/do {action: "plan"}
    Server->>Server: Create Action (status: "pending")
    Server-->>CLI: 202 Accepted

    loop Infinite Manager Loop
        Manager->>Server: GET /projects/:p/actions/next
        Note over Server: Find "pending" action
        Note over Server: Update status to "assigned" (Atomic)
        Server-->>Manager: Action {id, task_id, round_id, action: "plan"}
        
        Manager->>Server: POST /projects/:p/actions/:id/status {status: "running"}
        Server-->>Manager: 200 OK

        Manager->>Agent: Spawn process (with ADJENT_ACTION_ID)
        Agent-->>Manager: (stdout/stderr)
        Manager-->>Manager: Print to stdout
        Agent-->>Manager: Exit Code

        alt Success
            Manager->>Server: POST /projects/:p/actions/:id/status {status: "completed"}
        else Failure
            Manager->>Server: POST /projects/:p/actions/:id/status {status: "failed"}
        end
        Server-->>Manager: 200 OK
    end
```

## Agent Lifecycle

The manager must spawn the agent in the same environment that it is executed on. It forwards all environment variables and injects the following specific ones:

- `ADJENT_PROJECT_ID`: The current project ID.
- `ADJENT_TASK_ID`: The current task ID.
- `ADJENT_ROUND_ID`: The current round ID.
- `ADJENT_ACTION_ID`: The ID of the action being processed.
- `ADJENT_ACTION`: The name of the action (e.g., `plan`, `implement`).

The spawned agent uses these variables to interact with the project state, typically via the MCP server resources.

## Usage

```bash
adjent manage --project my-project --agent "gemini-cli"
```
