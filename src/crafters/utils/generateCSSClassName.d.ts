/**
 * Map containing unique IDs for characters.
 */
interface CharsMap {
  [key: string]: string;
}

/**
 * Generates a CSS class name based on the given string.
 *
 * @param str - The input string for which the CSS class name is generated.
 * @param is32Bits - If true, returns the last 4 characters of the generated hash.
 * @param is96Bits - If true, returns the last 12 characters of the generated hash.
 * @returns The generated CSS class name based on the input string.
 */
declare function generateCSSClassName(
  str: string,
  is32Bits?: boolean,
  is96Bits?: boolean
): string;

export = generateCSSClassName;
