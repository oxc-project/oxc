declare module "prettier-plugin-tailwindcss" {
  export interface PluginOptions {
    tailwindConfig?: string;
    tailwindStylesheet?: string;
    tailwindFunctions?: string[];
    tailwindAttributes?: string[];
    tailwindPreserveWhitespace?: boolean;
    tailwindPreserveDuplicates?: boolean;
  }

  export interface TailwindContext {
    getClassOrder(classes: string[]): [string, bigint | null][];
  }

  export interface TransformerEnv {
    context: TailwindContext;
    options: Partial<PluginOptions>;
  }

  export interface SortClassesOptions {
    env: TransformerEnv;
    ignoreFirst?: boolean;
    ignoreLast?: boolean;
    removeDuplicates?: boolean;
    collapseWhitespace?: boolean | { start?: boolean; end?: boolean };
  }

  export function getTailwindConfig(
    options: Partial<PluginOptions & { filepath?: string }>,
  ): Promise<TailwindContext>;

  export function sortClasses(classString: string, options: SortClassesOptions): string;
}
