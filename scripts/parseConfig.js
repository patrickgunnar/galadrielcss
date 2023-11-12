const path = require("path");
const fs = require("fs");

/**
 * Parse Galadriel configuration file.
 *
 * This function attempts to locate and parse Galadriel configuration file, "galadriel.json", in the project directory.
 * It retrieves relevant configuration exclude and returns it as array.
 *
 * @returns {object} An object containing the exclude and module configuration.
 */
function parseConfig() {
    try {// resolve the full path of the config
        const fullPath = path.resolve("galadriel.json");

        if (fs.existsSync(fullPath)) { // if the file exists
            // reads the config file
            const configJson = fs.readFileSync(fullPath);
            // parse the config content
            const config = JSON.parse(configJson);
            // collects the exclude content
            const { exclude = [], module = false } = config;

            return { exclude, module };
        }
    } catch (error) {
        console.error("An error occurred:", error);
    }

    return [];
}

module.exports = { parseConfig };
