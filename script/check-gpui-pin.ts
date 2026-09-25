#!/usr/bin/env bun
/** Check that every direct GPUI/Zed dependency uses one immutable upstream revision. */
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const manifestPath = process.argv[2] ?? join(resolve(import.meta.dir, ".."), "Cargo.toml");
const manifest = Bun.TOML.parse(readFileSync(manifestPath, "utf8")) as {
  workspace?: { dependencies?: Record<string, unknown> };
};
const source = "https://github.com/zed-industries/zed";
const dependencies = Object.entries(manifest.workspace?.dependencies ?? {})
  .filter(([, spec]) => typeof spec === "object" && spec !== null && (spec as { git?: string }).git === source)
  .map(([name, spec]) => ({ name, rev: (spec as { rev?: string }).rev }));
const gpui = dependencies.find(({ name }) => name === "gpui");
const problems: string[] = [];
if (!gpui?.rev || !/^[0-9a-f]{40}$/.test(gpui.rev)) {
  problems.push("gpui must pin a full upstream commit SHA");
}
for (const { name, rev } of dependencies) {
  if (rev !== gpui?.rev) problems.push(`${name} does not use the workspace gpui revision`);
}
if (problems.length) {
  for (const problem of problems) console.error(`::error file=${manifestPath}::${problem}`);
  process.exit(1);
}
console.log(`GPUI/Zed dependencies agree on ${gpui!.rev}`);
