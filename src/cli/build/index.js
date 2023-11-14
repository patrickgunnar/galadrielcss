const path = require("path");
const { glob } = require("glob");
const { Logger } = require("../../../scripts/logger.js");
const { parseConfig } = require("../../../scripts/parseConfig.js");
const { processContent } = require("../../../index.js");

/**
 * Assembles application styles by processing JavaScript and TypeScript files to generate CSS rules and files.
 * @async
 * @returns {Promise<void>} A promise that resolves when the process is complete.
 */
async function assembleApplicationStyles() {
    try {
        const logger = new Logger();

        // collects the exclude from config file
        const { exclude } = parseConfig();
        // get all the js, jsx, ts, tsx files
        const files = await glob("**/*.{js,jsx,ts,tsx}", {
            ignore: [
                "node_modules/**",
                "**/*.{config.js,config.ts}",
                "**/*.{.}",
                ...exclude.map((item) => {
                    // if item is the name of a folder
                    if (
                        !item.includes("./") &&
                        !item.includes("/") &&
                        !item.includes(".")
                    )
                        return `${item}/**`;
                    else return item;
                }),
            ],
        });

        // starting log
        logger.message(logger.makeBold(
            "\n----------------------------------------------\n   Galadriel.CSS build process just started\n----------------------------------------------"
        ));

        // loops through the array of files
        for (const __path of files) {
            // if current path does not include a starting dot
            if (__path[0] !== ".") {
                // process the path content
                processContent(path.resolve(__path));
            }
        }

        // ending log
        logger.message(logger.makeBold(
            "----------------------------------------------\n   Galadriel.CSS build ended successfully\n"
        ));
    } catch (error) {
        console.error("An error occurred:", error);
    }

    return;
}

module.exports = { assembleApplicationStyles };
