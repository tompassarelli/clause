import { dispatch } from "./generated/foreign-cli.js";

const argv: readonly string[] = ["module", "add"];
const outcome = dispatch(argv);
const message = outcome.message.trimEnd();
const status = outcome.status.toFixed(0);

if (message !== "firn: 'module add' requires a leaf node\n"
  + "Usage: firn module add <name>\n"
  + "  scaffold a minimal module (.bnix + .nix)"
  || status !== "1") {
  throw new Error(`Unexpected dispatch result: ${message} (${status})`);
}
console.log("typed dispatch consumer passed");
