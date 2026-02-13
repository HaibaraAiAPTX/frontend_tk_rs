/**
 * Plugin descriptor - identifies a plugin module
 */
export interface PluginDescriptor {
  /** Unique name for the plugin */
  name: string;
  /** Plugin version string */
  version: string;
}

/**
 * Context provided to plugin commands and renderers
 */
export interface PluginContext {
  /** Access to the native Rust binding */
  binding: typeof import('@aptx/frontend-tk-binding');
  /** Logging function for output */
  log: (msg: string) => void;
}

/**
 * Command handler function type
 */
export type CommandHandler = (
  ctx: PluginContext,
  args: Record<string, unknown>,
) => Promise<void> | void;

/**
 * Option descriptor for command options
 */
export interface OptionDescriptor {
  /** Option flags (e.g., "-o, --output <path>") */
  flags: string;
  /** Description of the option */
  description: string;
  /** Default value for the option */
  defaultValue?: string | boolean;
  /** Whether the option is required */
  required?: boolean;
}

/**
 * Command descriptor - defines a CLI command
 */
export interface CommandDescriptor {
  /** Command name (e.g., "my:command") */
  name: string;
  /** Brief summary of the command */
  summary: string;
  /** Detailed description (optional) */
  description?: string;
  /** Command options */
  options: OptionDescriptor[];
  /** Usage examples (optional) */
  examples?: string[];
  /** Handler function */
  handler: CommandHandler;
}

/**
 * Renderer descriptor - defines a code generator renderer
 */
export interface RendererDescriptor {
  /** Unique renderer ID */
  id: string;
  /** Render function */
  render: (
    ctx: PluginContext,
    options: Record<string, unknown>,
  ) => Promise<void> | void;
}

/**
 * Plugin interface - defines a loadable plugin module
 */
export interface Plugin {
  /** Plugin metadata */
  descriptor: PluginDescriptor;
  /** Commands provided by this plugin */
  commands: CommandDescriptor[];
  /** Optional renderers provided by this plugin */
  renderers?: RendererDescriptor[];
  /** Optional initialization callback */
  init?(context: PluginContext): void | Promise<void>;
}
