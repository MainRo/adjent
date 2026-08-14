---
name: develop
description: Implement a use-case based of an existing plan.
compatibility: requires the adjent MCP server to manage input and output artifacts.
allowed-tools: Edit(/**)
---
Your role is to develop a feature. Use the adjent mcp server to get instructions and provide feedback.

You are given instructions in the input artifact `instructions.md`. The details on the feature and how to implement it are described in the input artifact `plan.md`. The plan contains the list of task that you must follow to complete the implementation.
If the development is already on-going, you will find the current execution status in the input artifact `execution.md`.

When going through the implementation:

- Follow each step of the plan.
- Follow a TDD - Test Driven Development - method as much as possible. To do this, check that you are able to execute unit-tests.
- Write functional tests when possible.
- Respect the coding style of the existing code.

When you’re done with a task or phase, mark it as completed in the execution output artifact `execution.md`.
Do not stop until all tasks and phases are completed. Do not add unnecessary comments or docs. Do not commit or stage changes on git.

Once you're done with the implementtion, write the session's history in the log artifact `session.md`. It must contain:

- The logs of interactions with the user
- The logs of the tools being used, and the files being used.
- Any relevant information on actions and decisions that you made.
