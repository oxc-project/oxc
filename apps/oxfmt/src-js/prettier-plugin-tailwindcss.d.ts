declare module "prettier-plugin-tailwindcss" {
  export interface BatchSortOptions {
    // Config options for Tailwind sorter
  }

  export interface BatchSortContext {
    sortClasses(classes: string[]): string[];
  }

  export function createBatchSorter(
    options?: BatchSortOptions
  ): Promise<BatchSortContext>;
}
