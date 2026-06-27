import { describe, it, expect } from "vitest";
import { parseJsonStatus, parseTextStatus } from "../src/gh.js";

const JSON_OUT = JSON.stringify({
  hosts: {
    "github.com": [
      { login: "akc-barton", host: "github.com", active: true },
      { login: "Alexbruvv", host: "github.com", active: false },
    ],
  },
});

const TEXT_OUT = `github.com
  ✓ Logged in to github.com account akc-barton (keyring)
  - Active account: true
  - Git operations protocol: https

  ✓ Logged in to github.com account Alexbruvv (keyring)
  - Active account: false
  - Git operations protocol: https
`;

describe("gh status parsing", () => {
  it("parses --json hosts output", () => {
    const accounts = parseJsonStatus(JSON_OUT);
    expect(accounts).toEqual([
      { user: "akc-barton", host: "github.com", active: true },
      { user: "Alexbruvv", host: "github.com", active: false },
    ]);
  });

  it("parses the human-readable fallback output", () => {
    const accounts = parseTextStatus(TEXT_OUT);
    expect(accounts).toEqual([
      { user: "akc-barton", host: "github.com", active: true },
      { user: "Alexbruvv", host: "github.com", active: false },
    ]);
  });

  it("returns empty for unparseable json", () => {
    expect(parseJsonStatus("not json")).toEqual([]);
  });
});
