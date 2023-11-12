import { CraftClassesType } from "../types/typeManifest";

/**
 * The type of a callback function returning CraftClassesType.
 */
interface CallbackType {
    (): CraftClassesType;
}

/**
 * Process a callback to generate class names using genClassNames.
 *
 * @param callback - The callback function that returns CraftClassesType.
 * @returns The generated class names as a string.
 */
export function craftingStyles(callback: CallbackType): string;
