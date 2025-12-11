import { join } from "node:path";

/**
 * Resolve Prettier configuration from a directory path.
 * Uses a fake file path to trigger Prettier's config resolution.
 *
 * @param dirPath - Directory path to search from (current working directory)
 * @returns Prettier config as JSON string, or null if not found
 */
export async function getPrettierConfig(dirPath: string): Promise<string | null> {
  const prettier = await import("prettier");

  try {
    // Use a fake file path - Prettier will search upward from this directory
    // The file doesn't need to exist; Prettier just uses it as a starting point
    const fakePath = join(dirPath, "fake.js");
    const config = await prettier.resolveConfig(fakePath);
    return config ? JSON.stringify(config) : null;
  } catch (error) {
    console.error("Error resolving Prettier config:", error);
    return null;
  }
}
