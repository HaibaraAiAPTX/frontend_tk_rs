## ADDED Requirements
### Requirement: Meaningful `run_main_file` result
The system SHALL return the most meaningful value that the executed script produces. `run_main_file` SHALL inspect the evaluated module’s namespace for a `default` export and, when present, use that as the return value, invoking it if it is a function and awaiting any promise it produces. When no `default` export is defined or its value is `undefined`, `run_main_file` SHALL fall back to metrics exposed through `globalThis.result` so that imperative scripts can still share their result with the host environment.

#### Scenario: Global result is set
- **WHEN** a module sets `globalThis.result = "hello world"` (without exporting anything else) and the entry file is executed through `run_main_file`
- **THEN** the function returns `"hello world"`

#### Scenario: Default export value is returned
- **WHEN** a module exports a constant `default` value such as `"hello world"` and the entry file is executed through `run_main_file`
- **THEN** the function returns the exported value without invoking it as a function

#### Scenario: Default export function is invoked
- **WHEN** a module exports a synchronous function as the `default` export that returns `"hello world"`, and `run_main_file` executes the entry file
- **THEN** the function is called, and `run_main_file` returns the string produced by the invoked function

#### Scenario: Default export async function is awaited
- **WHEN** a module exports an async function as the `default` export that resolves to `"hello async"`, and `run_main_file` executes the entry file
- **THEN** the function is called, the resulting promise is awaited, and `run_main_file` returns `"hello async"`
