/**
 * @aptx/frontend-tk-cli
 *
 * Frontend toolkit CLI entry point.
 * This file assembles the core infrastructure with built-in plugins.
 */

import { createCli } from "@aptx/frontend-tk-core";
import aptxPlugin from "@aptx/frontend-tk-plugin-aptx";
import modelPlugin from "@aptx/frontend-tk-plugin-model";
import materalPlugin from "@aptx/frontend-tk-plugin-materal";
import inputPlugin from "@aptx/frontend-tk-plugin-input";

// Create CLI instance
const cli = createCli();

// Assemble built-in plugins
cli.use(aptxPlugin);
cli.use(modelPlugin);
cli.use(materalPlugin);
cli.use(inputPlugin);

// Start the CLI
cli.run(process.argv);
