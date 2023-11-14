const fs = require("fs");
const path = require("path");
const { Logger } = require("../../../scripts/logger");

function wizardBabel() {
    const logger = new Logger();
    
    // babel configuration
    const babelConfig = `{\n\t"plugins": ["galadrielcss/alchemy"]\n}\n`;
    // collects the root directory
    const rootPath = process.cwd();
    // creates babel path to the root directory
    const babelConfigPath = path.join(rootPath, ".babelrc");

    // generates babel config file
    fs.writeFileSync(babelConfigPath, babelConfig);
    logger.now("Babel configuration generated successfully!");
}

module.exports = { wizardBabel };
