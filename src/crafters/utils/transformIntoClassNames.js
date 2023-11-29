const generateCSSClassName = require("./generateCSSClassName");

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
                    // collects the nested value from the property
                    var nestedValue = value[nestedKey];

                    if (!nestedValue.includes("$")) {
                        var cls = generateCSSClassName(`${nestedKey}:${nestedValue}`);

                        // append the generated class name to the class name variable
                        classNames += `galadriel_${cls} `;
                    } else {
                        // append the generated class name to the class name variable
                        classNames += `${nestedValue.replace("$", "")} `;
                    }
                }
            } else if (!value.includes("$")) {
                var cls = generateCSSClassName(`${key}:${value}`);

                // append the generated class name to the class name variable
                classNames += `galadriel_${cls} `;
            } else {
                // append the generated class name to the class name variable
                classNames += `${value.replace("$", "")} `;
            }
        }
    } catch (_unused) {}

    return classNames.trim();
};
