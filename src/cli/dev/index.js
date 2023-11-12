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
                    // get the starting time
                    const startTime = new Date();

                    // console log the processing path 
                    logger.now(`processing the path: ${logger.makeBold(__path)}`);
                    // process the path content
                    processContent(path.resolve(__path));

                    // get the ending time
                    const endTime = new Date();

                    // log the successful completion with elapsed time
                    logger.now(`CSS generated successfully in ${logger.makeBold(endTime - startTime)} ms`);
                } catch (error) {
                    console.error("An error occurred:", error);
                }
            }
        });

        // starting log
        logger.now(logger.makeBold("Galadriel.CSS just started"));
    } catch (error) {
        console.error("An error occurred:", error);
    }
}

module.exports = { spectraScribe };
