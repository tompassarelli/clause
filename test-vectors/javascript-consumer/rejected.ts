import { dispatch } from "./generated/foreign-cli.js";

// @ts-expect-error Arguments must contain text, not numbers.
dispatch([42]);
// @ts-expect-error The argument sequence is required.
dispatch();

const outcome = dispatch(["module", "add"]);
// @ts-expect-error The message is text, not a number.
outcome.message.toFixed(0);
// @ts-expect-error The status is a number, not text.
outcome.status.trim();
// @ts-expect-error Result fields are readonly.
outcome.status = 0;
