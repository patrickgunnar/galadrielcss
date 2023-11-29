/**
 * An object representing class names and their values.
 */
interface ClassesObject {
  [key: string]: string | Record<string, string>;
}

/**
 * Transforms the CSS class names based on the provided classes object.
 *
 * @param classes - An object containing class names and their values.
 * @returns The generated CSS class names.
 */
declare function transformIntoClassNames(classes: ClassesObject): string;

export = transformIntoClassNames;
