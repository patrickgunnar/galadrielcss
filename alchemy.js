const { resolve } = require("path");
const { cloneDeep } = require("lodash");
const { parseConfig } = require("./scripts/parseConfig.js");
const { alchemyProcessing, generatesHashingHex } = require("./index.js");

// used objects and CSS rules controls
const usedObjects = [];
const transformedNodes = {};

/**
 * Generates transformations for properties within an object expression.
 *
 * @param {any} t - The t object used for AST transformation.
 * @param {any} node - The node to generate transformations for.
 * @param {boolean} isModular - A flag indicating if the transformations are related to a module.
 * @param {string | undefined} filePath - The file path associated with the transformations, if applicable.
 */
function generatesAlchemy(t, node, isModular, filePath) {
    try {
        // if the current node is not an object expression
        if (!t.isObjectExpression(node)) return;

        // loops through the node properties
        for (const property of node.properties) {
            // if the property is not an identifier
            if (!t.isIdentifier(property.key)) continue;

            // if the property's value type is a string literal
            if (property.value.type === "StringLiteral") {
                // transform the property's value
                const className = alchemyProcessing(property.key.name, JSON.stringify(property.value.value), isModular, filePath || "", "");

                // if class name is not empty
                if (className) property.value.value = className;

                // if the property's value is an object expression
            } else if (property.value.type === "ObjectExpression") {
                // loops through the nested properties
                for (const nestedProperty of property.value.properties) {
                    // if the nested property's value type is a string literal
                    if (nestedProperty.value.type === "StringLiteral") {
                        // transform the property's value
                        const className = alchemyProcessing(
                            nestedProperty.key.name, JSON.stringify(nestedProperty.value.value), isModular, filePath || "", property.key.name
                        );

                        // if class name is not empty
                        if (className) nestedProperty.value.value = className;
                    }
                }
            }
        }
    } catch (error) {
        console.error("An error occurred:", error);
    }
}

/**
 * Transforms an AST node and generates transformations for properties within it and its nested nodes.
 *
 * @param {any} t - The t object used for AST transformation.
 * @param {any} rootNode - The root node of the AST to transform.
 * @param {boolean} module - A flag indicating if the transformations are related to a module.
 * @param {string | undefined} filePath - The file path associated with the transformations, if applicable.
 */
function walkThroughOutAst(t, rootNode, isModular, filePath) {
    try {
        // loop stack
        const stack = [rootNode];

        // loops while exists the stack
        while (stack.length > 0) {
            // get the current node
            const node = stack.pop();

            // generates the transformations
            generatesAlchemy(t, node, isModular, filePath);

            // recursively process nested properties
            for (const nestedProperty in node) {
                // if current node element and type of the current node element is object and 
                // current node is not of the type of object expression and not current node
                // element an array
                if (
                    node[nestedProperty] && 
                    typeof node[nestedProperty] === 'object' && 
                    !(node.type === "ObjectExpression" && Array.isArray(node[nestedProperty]))
                ) {
                    // push the current node element into the stack
                    stack.push(node[nestedProperty]);
                }
            }
        }
    } catch (error) {
        console.error("An error occurred:", error);
    }
}

/**
 * Exported default function to process a Babel plugin.
 *
 * @param {Object} param - The parameters for the function.
 * @param {any} param.types - The types object for node analysis.
 * @returns {PluginObj} The Babel plugin object.
 */
module.exports = function ({types}) {
    const { exclude, module } = parseConfig();

    return {
        visitor: {
            CallExpression(path, state) {
                try {
                    // get the file path and check for exclusion
                    const filePath = state.filename;
                    const excludePath = exclude.some(__path => filePath?.includes(resolve(__path)));

                    // checks for an exclusion
                    if (excludePath) return;

                    // collects the callee
                    const callee = path.get("callee");

                    // if the callee is not "craftingStyles"
                    if (!callee.isIdentifier({ name: "craftingStyles" })) return;

                    // collects the argument's body - callback function
                    const callback = path.node.arguments[0];

                    // if not the callback function
                    if (!callback) return;

                    // collects the type of the callback function
                    const callbackType = callback.type;

                    // if the callback type is a function or arrow function
                    if (callbackType === "FunctionExpression" || callbackType === "ArrowFunctionExpression") {
                        // hash the callback body function body
                        const hashedNode = generatesHashingHex(JSON.stringify(callback.body).replace(/\s+/g, ""), true, false);

                        // if current exists in the control array
                        if (usedObjects.includes(hashedNode)) {
                            // collects the node from the
                            const collectedNode = transformedNodes[hashedNode];

                            // if not the current node
                            if (!collectedNode) {
                                // transform the current node
                                walkThroughOutAst(types, callback.body, module, filePath);
                                // save the transformed node
                                transformedNodes[hashedNode] = cloneDeep(callback.body);
                                // save the used objects
                                usedObjects.push(hashedNode);
                            } else {
                                // clone the collected node into the ast node
                                callback.body = cloneDeep(collectedNode);
                            }
                        } else {
                            // transform the current node
                            walkThroughOutAst(types, callback.body, module, filePath);
                            // save the transformed node
                            transformedNodes[hashedNode] = cloneDeep(callback.body);
                            // save the used objects
                            usedObjects.push(hashedNode);
                        }
                    }
                } catch (error) {
                    console.error("An error occurred:", error);
                }
            }
        }
    }
}
