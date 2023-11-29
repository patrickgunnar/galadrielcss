const transformIntoClassNames = require("./utils/transformIntoClassNames");

/**
 * Callback function type returning CraftClassesType.
 * @typedef {function(): CraftClassesType} CallbackType
 */

/**
 * Process a callback to generate class names using transformIntoClassNames.
 *
 * @param {CallbackType} callback - The callback function.
 * @returns {string} The generated class names.
 */
module.exports = function craftingStyles(callback) {
    return transformIntoClassNames(callback());
}
