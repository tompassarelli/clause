export function renderHostList(hosts: readonly string[]): string {
  return `Hosts (${hosts.length}):\n${hosts.map(host => `  ${host}\n`).join("")}`;
}
