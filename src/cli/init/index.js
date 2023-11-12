const fs = require("fs");
const path = require("path");
const { Logger } = require("../../../scripts/logger");

const galadrielConfig = `{
    "module": true,
    "exclude": ["node_modules"]
}
`;

function galadrielInit() {
    const logger = new Logger();
    const rootPath = process.cwd();
    const galadrielConfigPath = path.join(rootPath, "galadriel.json");

    fs.writeFileSync(galadrielConfigPath, galadrielConfig);
    logger.now("Configuration generated successfully!");
}

module.exports = { galadrielInit };
