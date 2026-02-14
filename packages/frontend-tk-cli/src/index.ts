/**
 * @aptx/frontend-tk-cli
 *
 * Frontend toolkit CLI entry point.
 * This file assembles the core infrastructure with built-in plugins.
 */

import { createCli } from "@aptx/frontend-tk-core";
import aptxPlugin from "@aptx/frontend-tk-plugin-aptx";
import standardPlugin from "@aptx/frontend-tk-plugin-standard";
import modelPlugin from "@aptx/frontend-tk-plugin-model";
import materalPlugin from "@aptx/frontend-tk-plugin-materal";
import inputPlugin from "@aptx/frontend-tk-plugin-input";
import { createCodegenRunCommand } from "./codegen-run";

// Create CLI instance
const cli = createCli();

// Assemble built-in plugins
cli.use(aptxPlugin);
cli.use(standardPlugin);
cli.use(modelPlugin);
cli.use(materalPlugin);
cli.use(inputPlugin);

// Register CLI-specific convenience command (codegen:run)
cli.registerNamespaceDescription('codegen', 'Code generation (run)');
cli.registerCommand(createCodegenRunCommand());

// Start the CLI
cli.run(process.argv);
