/**
 * Generate class names from a record of classes.
 *
 * @param {Record<string, any>} classes - The record of classes.
 * @returns {string} The generated class names.
 */
export function genClassNames(classes) {
    return Object.entries(classes).map(([_, cls]) => {
        if (typeof cls === "object") {
            return Object.values(cls).join(" ");
        } else if (typeof cls === "string") {
            return cls;
        } else {
            return JSON.stringify(cls);
        }
    }).join(" ");
};
