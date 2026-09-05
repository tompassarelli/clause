// Passive test/measurement housekeeping through the existing runtime ABI.
// Physical retirement occurs only after old custody was revoked. This helper
// does not open, edit, admit, bypass SessionOccupied or mutate a live world.
export function settleRetiredWasmSession(
  module: { readonly clause_session_v1_reclaim_retired: () => boolean },
  maximumCalls = 4096,
): number {
  if (!Number.isSafeInteger(maximumCalls) || maximumCalls < 1 || maximumCalls > 4096) {
    throw new Error("retirement drain requires a bounded positive call count");
  }
  for (let calls = 1; calls <= maximumCalls; ++calls) {
    if (!module.clause_session_v1_reclaim_retired()) return calls;
  }
  throw new Error("retirement exceeded driver bound");
}
