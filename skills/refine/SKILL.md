---
name: refine
description: Refines a use-case by writing a plan. Deep-dive into the implications of the use-case and how to implement it.
compatibility: requires the adjent MCP server to manage input and output artifacts.
allowed-tools: Read(/**)
---

Your role is to build a plan to implement a feature. Use the adjent mcp server to get instruction and provide feedback.

You are given instructions in the input artifact `instructions.md`. If the use-case is already under refinement, you may find these additional input information:

- The current plan in the input artifact `plan.md`.
- The current list of decisions in the input artifact `decisions.md`.

You always update the plan in the output artifact `plan.md`
You always update the decisions in the output artifact `decisions.md`

When going through the refinement:

- Write a detailed plan document outlining how to achieve the use-case.
- Base the plan on the actual codebase.
- Include code snippets.
- Include a TODO list, with all the phases and individual tasks necessary to complete the plan.
- Include diagrams - in mermaid format - to add in the docs whenever this can clarify the behavior.

The output plan MUST be self-contained. So keep any information from the input plan that is still applicable. When a candidate solution is rejected, add it to the list of decisions.

The team follows a TDD - Test Driven Development - method as much as possible. Take this into account in the TODO list so that tests are written before the implementation.

Once you're done writing the plan, write the session's history in the log artifact `session.md`. It must contain:

- The logs of interactions with the user
- The logs of the tools being used, and the files being used.
- Any relevant information on actions and decisions that you made.

ONLY build the plan. Do NOT develop yet.
