const path = require("path");
const fs = require("fs");
const { coreStaticStyles } = require(path.join(__dirname, "..", "build", "src", "kernel", "coreStaticStyles"));
const { coreDynamicProperties } = require(path.join(__dirname, "..", "build", "src", "kernel", "coreDynamicProperties"));

const pseudoClasses = [
    "hover",
    "active",
    "focus",
    "firstChild",
    "lastChild",
    "firstOfType",
    "lastOfType",
    "visited",
    "checked",
    "onlyChild",
    "onlyOfType",
    "targetPseudoClass",
    "disabled",
    "enabled",
    "readOnly",
    "readWrite",
    "placeholderShown",
    "valid",
    "invalid",
    "required",
    "optional",
    "fullscreen",
    "focusWithin",
    "firstLine",
    "firstLetter",
    "before",
    "after",
    "outOfRange",
    "root",
    "firstPage",
    "leftPage",
    "rightPage",
    "empty",
    "minLargeDesktops",
    "minStandardDesktops",
    "minPortraitTablets",
    "minLargeSmartphones",
    "minStandardSmartphones",
    "maxLargeDesktops",
    "maxStandardDesktops",
    "maxPortraitTablets",
    "maxLargeSmartphones",
    "maxStandardSmartphones",
];

/**
 * Generates type definitions and configuration for galadriel based on the available keys in the cores.
 *
 * @returns {{ types: string, config: string } | null} An object containing type definitions and configuration, or null if an error occurs.
 */
function dynamicObjectManager() {
    try {
        // config array
        const config = [
            "module?: boolean;",
            "output?: string;",
            "exclude?: string[];",
            "include?: string[];",
            "craftStyles?: {",
        ];

        // collects the keys inside the cores
        const keys = new Set([
            ...Object.keys(coreStaticStyles),
            ...Object.keys(coreDynamicProperties),
        ]);

        // loop through keys content
        const types = Array.from(keys).map((key) => {
            config.push(`${key}?: Record<string, string>;`);

            if (pseudoClasses.includes(key)) {
                return `${key}?: Record<string, string>;`;
            }

            return `${key}?: string;`;
        });

        // push a config close curly brackets
        config.push("}");

        // return the generated types and config
        return {
            types: types.join(" "),
            config: config.join(" "),
        };
    } catch (error) {
        console.error("An error occurred:", error);
    }

    return null;
}

/**
 * Generates and writes TypeScript type definitions to "typeManifest.ts" and "config.ts" based on the data collected by dynamicObjectManager.
 */
function typeWeaver() {
    // collects the objects data containing the config anf types
    const objectsData = dynamicObjectManager();

    if (objectsData) {
        try {
            const { types, config } = objectsData;

            if (types) {
                try {
                    // write the file with the types
                    fs.writeFileSync(
                        path.join(__dirname, "..", "src", "types", "typeManifest.ts"),
                        `export type CraftClassesType = { ${types} }`
                    );
                } catch (error) {
                    console.error("An error occurred:", error);
                }
            }

            if (config) {
                try {
                    // write the file with the config
                    fs.writeFileSync(
                        path.join(__dirname, "..", "src", "types", "config.ts"),
                        `export type Config = { ${config} }`
                    );
                } catch (error) {
                    console.error("An error occurred:", error);
                }
            }
        } catch (error) {
            console.error("An error occurred:", error);
        }
    }
}

typeWeaver();
