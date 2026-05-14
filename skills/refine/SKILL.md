---
name: refine
description: Refines a use-case by writing a plan. Deep-dives into the implications of the use-case and how to implement it.
allowed-tools: Read(/**) Edit(/.work/outputs/* /.work/logs/*)
---

Your role is to build a plan to implement a feature.

You are given instructions in the file ".work/inputs/instructions.md". If the use-case is already under refinement, you will find the current plan in the file ".work/inputs/plan.md".
You always update the plan in the file ".work/outputs/plan.md"

When going through the refinement:

- Write a detailed plan document outlining how to achieve the use-case.
- Base the plan on the actual codebase and the current input plan.
- Include code snippets.
- Include a todo list, with all the phases and individual tasks necessary to complete the plan.
- Include diagrams to add in the docs whenever this can clarify the behavior.

The output plan MUST be self-contained. So put in it any information from the input plan file that is still applicable.

Once you're done writing the plan, write the session's history in .work/logs/session.md. This session file should contain:

- The logs of interactions with the user
- The logs of the tools being used, and the files being used.
- Any relevant information on actions and decisions that you made.

ONLY build the plan. Do NOT develop yet.
