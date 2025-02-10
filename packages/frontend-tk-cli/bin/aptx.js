#!/usr/bin/env node
const nodeModule = require("node:module");

// enable on-disk code caching of all modules loaded by Node.js
// requires Nodejs >= 22.8.0
const { enableCompileCache } = nodeModule;
if (enableCompileCache) {
	try {
		enableCompileCache();
	} catch {
		// ignore errors
	}
}

process.title = "aptx-frontend-tk-node"

const { run } = require("../dist/index")
run(process.argv.slice(2))