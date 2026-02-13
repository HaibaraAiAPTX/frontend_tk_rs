import { Command, Option } from 'commander';
import * as binding from '@aptx/frontend-tk-binding';
import type {
  Plugin,
  PluginContext,
  CommandDescriptor,
  RendererDescriptor,
} from './types';

/**
 * Core CLI interface
 */
export interface Cli {
  use(plugin: Plugin): Cli;
  registerCommand(command: CommandDescriptor): Cli;
  run(argv: string[]): Promise<void>;
  findRenderer(id: string): RendererDescriptor | undefined;
  getRegisteredCommands(): CommandDescriptor[];
}

/**
 * Internal state for the CLI instance
 */
interface CliState {
  plugins: Plugin[];
  commands: CommandDescriptor[];
  renderers: Map<string, RendererDescriptor>;
  program: Command;
  context: PluginContext;
}

/**
 * Create a plugin context with binding and logging
 */
function createPluginContext(): PluginContext {
  return {
    binding,
    log: (msg: string) => console.log(msg),
  };
}

/**
 * Parse colon-separated command name into namespace and command
 * e.g., "model:gen" -> { namespace: "model", commandName: "gen" }
 * e.g., "build" -> { namespace: undefined, commandName: "build" }
 */
function parseCommandName(name: string): { namespace: string | undefined; commandName: string } {
  const colonIndex = name.indexOf(':');
  if (colonIndex === -1) {
    return { namespace: undefined, commandName: name };
  }
  return {
    namespace: name.slice(0, colonIndex),
    commandName: name.slice(colonIndex + 1),
  };
}

/**
 * CLI implementation class
 */
class CliImpl implements Cli {
  private state: CliState;
  private subcommandGroups: Map<string, Command>;

  constructor() {
    this.state = {
      plugins: [],
      commands: [],
      renderers: new Map(),
      program: new Command(),
      context: createPluginContext(),
    };
    this.subcommandGroups = new Map();

    // Configure the base program
    this.state.program
      .name('aptx-ft')
      .description('Frontend toolkit CLI for OpenAPI code generation.')
      .version('0.1.0')
      .exitOverride()
      .allowUnknownOption(true)
      .addOption(new Option('-i, --input <path>', 'Override input OpenAPI path/url'))
      .addOption(new Option('-p, --plugin <paths...>', 'Extra plugin dll paths'));

    // Add help text
    this.state.program.addHelpText(
      'after',
      `
Commands are organized by namespace:
  aptx       - @aptx ecosystem (functions, react-query, vue-query)
  std        - Standard library (axios-ts, axios-js, uniapp)
  model      - Model generation (gen, ir, enum-plan, enum-apply)
  input      - Input handling (download)
  codegen    - Code generation (run)

Use 'aptx-ft <namespace> <command> --help' for details.
`,
    );
  }

  /**
   * Register a plugin with the CLI
   */
  use(plugin: Plugin): Cli {
    this.state.plugins.push(plugin);

    // Register plugin's commands
    for (const command of plugin.commands) {
      this.registerCommand(command);
    }

    // Register plugin's renderers
    if (plugin.renderers) {
      for (const renderer of plugin.renderers) {
        this.state.renderers.set(renderer.id, renderer);
      }
    }

    // Call plugin init if provided
    if (plugin.init) {
      Promise.resolve().then(() => plugin.init!(this.state.context));
    }

    return this;
  }

  /**
   * Get or create a subcommand group for a namespace
   */
  private getOrCreateNamespace(namespace: string): Command {
    if (this.subcommandGroups.has(namespace)) {
      return this.subcommandGroups.get(namespace)!;
    }

    const group = new Command(namespace);
    this.state.program.addCommand(group);
    this.subcommandGroups.set(namespace, group);
    return group;
  }

  /**
   * Register a single command
   */
  registerCommand(command: CommandDescriptor): Cli {
    this.state.commands.push(command);

    const parsed = parseCommandName(command.name);

    // Create Commander command with just the command name (without namespace prefix)
    const cmd = new Command(parsed.commandName);
    cmd.description(command.summary);

    if (command.description) {
      cmd.description(command.summary + '\n\n' + command.description);
    }

    // Add options
    for (const opt of command.options) {
      cmd.option(opt.flags, opt.description, opt.defaultValue);
    }

    // Add examples as help text
    if (command.examples && command.examples.length > 0) {
      cmd.addHelpText(
        'after',
        '\nExamples:\n  ' + command.examples.join('\n  '),
      );
    }

    // Set up action handler
    cmd.action(async (options: Record<string, unknown>) => {
      try {
        // Merge global options with command options
        const globalOpts = this.state.program.opts();
        const mergedOptions = { ...globalOpts, ...options };
        await command.handler(this.state.context, mergedOptions);
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        console.error(`Error: ${message}`);
        process.exitCode = 1;
      }
    });

    // Add command to namespace group or directly to program
    if (parsed.namespace) {
      const group = this.getOrCreateNamespace(parsed.namespace);
      group.addCommand(cmd);
    } else {
      this.state.program.addCommand(cmd);
    }

    return this;
  }

  /**
   * Find a registered renderer by ID
   */
  findRenderer(id: string): RendererDescriptor | undefined {
    return this.state.renderers.get(id);
  }

  /**
   * Get all registered commands
   */
  getRegisteredCommands(): CommandDescriptor[] {
    return [...this.state.commands];
  }

  /**
   * Run the CLI with provided arguments
   */
  async run(argv: string[]): Promise<void> {
    try {
      await this.state.program.parseAsync(argv, { from: 'user' });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      console.error(`Error: ${message}`);
      process.exitCode = 1;
    }
  }
}

/**
 * Create a new CLI instance
 */
export function createCli(): Cli {
  return new CliImpl();
}

/**
 * Export types for external use
 */
export type { Plugin, PluginContext, CommandDescriptor, RendererDescriptor } from './types';
