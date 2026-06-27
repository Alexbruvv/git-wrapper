#!/usr/bin/env bun
// Cross-compile standalone `gw` binaries for every supported platform into
// dist/. Run with `bun run build:all`.
import { $ } from "bun";

const ENTRY = "./src/cli.ts";

const targets = [
    { name: "linux-x64", target: "bun-linux-x64" },
    { name: "linux-arm64", target: "bun-linux-arm64" },
    { name: "darwin-x64", target: "bun-darwin-x64" },
    { name: "darwin-arm64", target: "bun-darwin-arm64" },
    { name: "windows-x64", target: "bun-windows-x64" },
];

for (const { name, target } of targets) {
    const ext = name.startsWith("windows") ? ".exe" : "";
    const outfile = `dist/gw-${name}${ext}`;
    console.log(`building ${outfile} (${target})`);
    await $`bun build ${ENTRY} --compile --target=${target} --outfile ${outfile}`;
}

console.log("done");
