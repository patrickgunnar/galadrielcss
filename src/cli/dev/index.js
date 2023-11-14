const path = require("path");
const chokidar = require("chokidar");
const { Logger } = require("../../../scripts/logger");
const { parseConfig } = require("../../../scripts/parseConfig.js");
const { processContent } = require("../../../index.js");

// extensions to be watched
const extensions = ["**/*.js", "**/*.jsx", "**/*.ts", "**/*.tsx"];

/**
 * Begin monitoring and processing code changes with Galadriel.
 *
 * This function initializes Galadriel's code monitoring and processing system. 
 * It watches for changes in specified files, processes them, 
 * and generates CSS files.
 */
function spectraScribe() {
    try {
        const logger = new Logger();
        // collects the exclude content from the config
        const { exclude } = parseConfig();

        // if ignore and output do not exist
        if (!exclude) {
            return logger.message(
                "The galadriel.json file should have the fields ('exclude' and 'output') or ('exclude' and 'module - (enabled)')", true
            );
        }

        // instantiate the watcher
        const watcher = chokidar.watch(extensions, {
            persistent: true,
            ignoreInitial: true,
            ignored: [...exclude, /(^|[/\\])\../],
        });

        // watch all changes on the application
        watcher.on("change", (__path) => {
            // if current path does not include a starting dot
            if (__path[0] !== ".") {
                try {
                    // process the path content
                    processContent(path.resolve(__path));
                } catch (error) {
                    console.error("An error occurred:", error);
                }
            }
        });

        // starting log
        logger.message(logger.makeBold(
            "\n----------------------------------------------\n   Galadriel.CSS just started\n----------------------------------------------"
        ));
    } catch (error) {
        console.error("An error occurred:", error);
    }
}

module.exports = { spectraScribe };
