import { ChildProcess } from "@figliolia/child-process";

/**
 * Compress
 *
 * Given an absolute path to a directory, generates gzip,
 * zstandard, deflate, and brotli file variants for all
 * files recursively
 */
export const compress = async (directory: string) => {
  return new ChildProcess(`ui-build-compression ${directory}`).handler;
};
