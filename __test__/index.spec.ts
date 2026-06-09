import { cwd } from "node:process";
import { join } from "node:path";
import { cp, readdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";

import test from "ava";

import { compress } from "../dist";

const FIXTURE_PATH = join(cwd(), "fixture");
const TARGET_FIXTURE_PATH = join(cwd(), "__test__", "fixture");

test.before(async () => {
  await cp(FIXTURE_PATH, TARGET_FIXTURE_PATH, { recursive: true, force: true });
});

test.after(async () => {
  await rm(TARGET_FIXTURE_PATH, { force: true, recursive: true });
});

test("Test compression", async t => {
  await compress(TARGET_FIXTURE_PATH);
  const variants = ["br", "gz", "zstd", "deflate"];
  const entries = await readdir(FIXTURE_PATH, {
    withFileTypes: true,
    recursive: true,
  });
  for (const entry of entries) {
    if (entry.isFile()) {
      for (const variant of variants) {
        t.is(
          existsSync(
            join(
              TARGET_FIXTURE_PATH,
              entry.parentPath.replace(FIXTURE_PATH, ""),
              `${entry.name}.${variant}`,
            ),
          ),
          true,
        );
      }
    }
  }
});

[join(cwd(), "path/to/nowhere")].forEach(path => {
  test(`Invalid Path: ${path}`, async t => {
    await t.throwsAsync(() => compress(path));
  });
});
