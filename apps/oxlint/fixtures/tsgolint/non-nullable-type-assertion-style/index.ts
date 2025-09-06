// Examples of incorrect code for non-nullable-type-assertion-style rule

declare const value: string | null;

// Type assertion when non-null assertion would be clearer
const result1 = value as string;

declare const maybe: number | undefined;
const result2 = maybe as number;