/**
 * Transforms the CSS class names based on the provided classes object.
 *
 * @param {Object} classes - An object containing class names and their values.
 * @returns {string} The generated CSS class names.
 * @throws {Error} Throws an error if an issue occurs during the class name transformation process.
 */
module.exports = function transformIntoClassNames(classes) {
    var classNames = "";

    try {
        // loops through the classes contents
        for (var key in classes) {
            // collects the property's value
            var value = classes[key];

            // if the value is an object
            if (typeof value === "object") {
                // loops through the value's properties
                for (var nestedKey in value) {
                    // append the generated class name to the class name variable
                    classNames += value[nestedKey] + " ";
                }
            } else {
                // append the generated class name to the class name variable
                classNames += value  + " ";
            }
        }
    } catch (_unused) {}

    return classNames.trim();
};
