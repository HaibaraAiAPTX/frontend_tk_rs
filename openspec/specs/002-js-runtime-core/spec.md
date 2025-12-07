# Feature Specification: JS Runtime Core

**Feature Branch**: 02-js-runtime-core
**Created**: 2025-12-07
**Status**: Draft
**Input**: User description: "Build a lib crate that can run js/ts code. Support setTimeout, setInterval, Promise like web enviroment. Support top-await, js module. The core is two methods, one supports running the script directly, and the other supports running a certain method in the script, If the return value is Promise, the result of the Promise needs to be returned"

## Purpose
Run modern JS/TS projects inside Rust by providing a browser-like runtime that supports timers, promises, modules, and the ability to execute entry files or individual exports.

## User Scenarios & Testing

### User Story 1 - Execute Script with Async Support (Priority: P1)

As a developer, I want to execute JS/TS scripts that use Promises and top-level await so that I can run modern JavaScript code.

**Why this priority**: Core functionality required for the runtime.

**Independent Test**: Create a script with top-level await and verify the result is returned correctly.

**Acceptance Scenarios**:

1. **Given** a script with wait Promise.resolve(42), **When** executed via `run_main_file`, **Then** it returns 42.
2. **Given** a script that returns a Promise, **When** executed, **Then** the runtime waits for resolution and returns the value.

---

### User Story 2 - Timer Support (Priority: P1)

As a developer, I want to use setTimeout and setInterval in my scripts so that I can schedule tasks.

**Why this priority**: Essential for web-compatible environment.

**Independent Test**: Run a script that sets a timeout to change a variable, wait, and check the variable.

**Acceptance Scenarios**:

1. **Given** a script using setTimeout, **When** executed, **Then** the callback is invoked after the delay.
2. **Given** a script using setInterval, **When** executed, **Then** the callback is invoked repeatedly until cleared.

---

### User Story 3 - Invoke Specific Method (Priority: P2)

As a developer, I want to call a specific exported function from a script so that I can use the script as a library.

**Why this priority**: Allows granular interaction with script logic.

**Independent Test**: Define a script with multiple functions, call one specifically and verify output.

**Acceptance Scenarios**:

1. **Given** a script exporting function add(a, b) { return a + b; }, **When** calling add with arguments, **Then** it returns the sum.
2. **Given** an async function, **When** called, **Then** it returns the resolved value.

---

### Edge Cases

- What happens when the script throws an exception? (Should return error)
- What happens when a timeout is never cleared? (Should eventually stop or have a mechanism to stop)
- What happens with circular module imports? (Should be handled by module loader)
- What happens if the requested method does not exist? (Should return error)
## Requirements
### Requirement: Script execution API
System SHALL provide an API to execute a JS/TS script file.

#### Scenario: Execute entry script
- **WHEN** `run_main_file` is invoked with a script path
- **THEN** the runtime evaluates the entry module and completes without crashing.

### Requirement: Timer globals
System SHALL support setTimeout and setInterval global functions with standard web behavior.

#### Scenario: Timeout schedules callback
- **WHEN** a script schedules a timeout to update state
- **THEN** the callback runs after the requested delay and the state is observable.

#### Scenario: Interval repeats until cleared
- **WHEN** a script schedules an interval and clears it after multiple ticks
- **THEN** each tick executes until the counter triggers the clear call.

### Requirement: Promise and async support
System SHALL support Promise and sync/await syntax so asynchronous flows resolve predictably.

#### Scenario: Await resolves promise result
- **WHEN** a script uses `await` on a Promise
- **THEN** the runtime delivers the resolved value before continuing.

### Requirement: Top-level await
System SHALL support top-level await expressions within entry scripts.

#### Scenario: Top-level await returns value
- **WHEN** a script uses top-level `await` to resolve a Promise
- **THEN** the runtime waits for completion before finishing script evaluation.

### Requirement: ES modules
System SHALL support ES module imports and exports for scripts loaded by the runtime.

#### Scenario: Import shared helper
- **WHEN** a script imports utilities from another file
- **THEN** the runtime resolves the module graph and exposes the imported bindings.

### Requirement: Named export invocation
System SHALL provide an API to invoke a specific named export from a loaded script.

#### Scenario: Call exported function
- **WHEN** `run_func` requests an exported helper with arguments
- **THEN** the runtime invokes the function and returns the result, respecting Promise resolution.

### Requirement: Promise unwrapping
System SHALL automatically resolve any returned Promise before handing the result back to the host environment.

#### Scenario: Entry script returns Promise
- **WHEN** a script returns a Promise from the entry point or an invoked function
- **THEN** the runtime awaits the Promise and returns the resolved value.

### Requirement: Exception handling
System SHALL surface JavaScript exceptions as strings to the host environment.

#### Scenario: Script throws error
- **WHEN** the script raises an exception during execution
- **THEN** `run_main_file` returns an error containing the exception message.

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

