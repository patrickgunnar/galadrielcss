import { CraftClassesType } from "./src/types/typeManifest";

interface CallbackType {
    (): CraftClassesType;
}

export function craftingStyles(callback: CallbackType): string;
