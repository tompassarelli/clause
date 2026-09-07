# JavaScript declaration consumer

Replace `LANE` with the Clause worktree name. Install the checker with
`bun install --cwd ~/code/clause/worktrees/LANE/test-vectors/javascript-consumer`,
then run
`bash ~/code/clause/worktrees/LANE/test-vectors/javascript-consumer/check.sh /absolute/path/to/clause-workbench`.
Set `BUN` if Bun is not on `PATH`.

The check emits the existing `clause:test-vectors/authoring/foreign-cli.clause`
module, checks a TypeScript consumer against its generated declarations, and
executes that consumer with Bun. It accepts readonly argv and uses inferred
message/status types. `clause:test-vectors/javascript-consumer/rejected.ts`
requires rejection of wrong arguments, wrong result operations, and result
mutation; unused `@ts-expect-error` directives fail the check.
