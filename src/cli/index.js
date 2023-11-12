#!/usr/bin/env node

const yargs = require("yargs");
const { galadrielInit } = require("./init");
const { assembleApplicationStyles } = require("./build");
const { spectraScribe } = require("./dev");
const { wizardBabel } = require("./babel");

yargs
    .command({
        command: "init",
        describe: "Configure the Galadriel.CSS environment",
        handler: (_) => {
            galadrielInit();
        },
    })
    .command({
        command: "build",
        describe: "Galadriel.CSS build process",
        handler: (_) => {
            assembleApplicationStyles();
        },
    })
    .command({
        command: "dev",
        describe: "Galadriel.CSS development process",
        handler: (_) => {
            spectraScribe();
        },
    })
    .command({
        command: "babel",
        describe: "Generates Babel's configuration ('.babelrc') with the necessary plugin to handle the build process",
        handler: (_) => {
            wizardBabel();
        }
    });

yargs.parse();
