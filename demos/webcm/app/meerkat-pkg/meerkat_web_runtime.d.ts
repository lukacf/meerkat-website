/* tslint:disable */
/* eslint-disable */
/**
 * The `ReadableStreamType` enum.
 *
 * *This API requires the following crate features to be activated: `ReadableStreamType`*
 */

type ReadableStreamType = "bytes";

export class IntoUnderlyingByteSource {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    cancel(): void;
    pull(controller: ReadableByteStreamController): Promise<any>;
    start(controller: ReadableByteStreamController): void;
    readonly autoAllocateChunkSize: number;
    readonly type: ReadableStreamType;
}

export class IntoUnderlyingSink {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    abort(reason: any): Promise<any>;
    close(): Promise<any>;
    write(chunk: any): Promise<any>;
}

export class IntoUnderlyingSource {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    cancel(): void;
    pull(controller: ReadableStreamDefaultController): Promise<any>;
}

/**
 * Clear all registered tool callbacks.
 */
export function clear_tool_callbacks(): void;

/**
 * Close a subscription and free resources.
 */
export function close_subscription(handle: number): void;

/**
 * Create a session from a mobpack + config.
 *
 * Routes through `AgentFactory::build_agent()` with override-first
 * resource injection — same pipeline as all other meerkat surfaces.
 *
 * `config_json`: `{ "model": "...", "api_key": "sk-...", "max_tokens"?: N,
 *                    "comms_name"?: "...", "host_mode"?: true }`
 */
export function create_session(mobpack_bytes: Uint8Array, config_json: string): number;

/**
 * Create a session without a mobpack — for standalone agent use.
 *
 * `config_json`: `{ "model": "...", "api_key": "sk-...", "system_prompt"?: "...",
 *                    "max_tokens"?: N, "additional_instructions"?: ["..."] }`
 *
 * Uses `register_tool_callback` tools if any were registered before this call.
 */
export function create_session_simple(config_json: string): number;

/**
 * Remove a session.
 */
export function destroy_session(handle: number): void;

/**
 * Get current session state.
 */
export function get_session_state(handle: number): string;

/**
 * Primary bootstrap: parse a mobpack and create service infrastructure.
 *
 * `mobpack_bytes`: tar.gz mobpack archive.
 * `credentials_json`: `{ "api_key": "sk-...", "anthropic_api_key"?: "...", "openai_api_key"?: "...", "gemini_api_key"?: "...", "model"?: "claude-sonnet-4-5" }`
 *
 * Stores an `EphemeralSessionService<FactoryAgentBuilder>` and a `MobMcpState`
 * in a `thread_local! RuntimeState` for subsequent mob/comms calls.
 */
export function init_runtime(mobpack_bytes: Uint8Array, credentials_json: string): any;

/**
 * Advanced bare-bones bootstrap without a mobpack.
 *
 * `config_json`: `{ "api_key"?: "sk-...", "anthropic_api_key"?: "...", "openai_api_key"?: "...", "gemini_api_key"?: "...", "model"?: "claude-sonnet-4-5", "max_sessions"?: 64 }`
 */
export function init_runtime_from_config(config_json: string): any;

/**
 * Inspect a mobpack without creating a session.
 */
export function inspect_mobpack(mobpack_bytes: Uint8Array): string;

/**
 * Cancel an in-flight flow run.
 */
export function mob_cancel_flow(mob_id: string, run_id: string): Promise<void>;

/**
 * Create a new mob from a definition JSON.
 *
 * Returns the mob_id as a string.
 */
export function mob_create(definition_json: string): Promise<any>;

/**
 * Fetch mob events.
 *
 * Returns JSON array of mob events.
 *
 * Note: `after_cursor` is u32 at the JS boundary (wasm_bindgen limitation),
 * internally widened to u64. Cursors beyond 4B are not supported via this export.
 */
export function mob_events(mob_id: string, after_cursor: number, limit: number): Promise<any>;

/**
 * Read flow run status.
 *
 * Returns JSON with run state and ledgers, or null if not found.
 */
export function mob_flow_status(mob_id: string, run_id: string): Promise<any>;

/**
 * Inject a message and subscribe for interaction-scoped events.
 *
 * Returns JSON: `{ "interaction_id": "..." }`
 */
export function mob_inject_and_subscribe(mob_id: string, meerkat_id: string, message: string): Promise<any>;

/**
 * Perform a lifecycle action on a mob.
 *
 * `action`: one of "stop", "resume", "complete", "destroy".
 */
export function mob_lifecycle(mob_id: string, action: string): Promise<void>;

/**
 * List all mobs.
 *
 * Returns JSON array of `{ mob_id, state }`.
 */
export function mob_list(): Promise<any>;

/**
 * List all members in a mob.
 *
 * Returns JSON array of roster entries.
 */
export function mob_list_members(mob_id: string): Promise<any>;

/**
 * Subscribe to a mob member's session event stream.
 *
 * Returns a subscription handle. Use `poll_subscription(handle)` to drain
 * buffered events. Each call returns all events since the last poll.
 *
 * The subscription captures ALL agent activity: text deltas, tool calls
 * (including comms send_message/peers), turn completions, etc.
 */
export function mob_member_subscribe(mob_id: string, meerkat_id: string): Promise<number>;

/**
 * Retire and re-spawn a meerkat with the same profile.
 *
 * Returns JSON: `{ "meerkat_id", "status": "respawn_enqueued" }`
 */
export function mob_respawn(mob_id: string, meerkat_id: string, initial_message?: string | null): Promise<any>;

/**
 * Retire a meerkat from a mob.
 */
export function mob_retire(mob_id: string, meerkat_id: string): Promise<void>;

/**
 * Start a configured flow run.
 *
 * Returns the run_id as a string.
 */
export function mob_run_flow(mob_id: string, flow_id: string, params_json: string): Promise<any>;

/**
 * Send an external message to a spawned meerkat.
 */
export function mob_send_message(mob_id: string, meerkat_id: string, message: string): Promise<void>;

/**
 * Spawn one or more meerkats in a mob.
 *
 * `specs_json`: JSON array of `{ "profile": "...", "meerkat_id": "...", "initial_message"?: "...",
 *                "runtime_mode"?: "autonomous_host"|"turn_driven", "backend"?: "subagent"|"external" }`
 *
 * Returns JSON array of results per spec.
 */
export function mob_spawn(mob_id: string, specs_json: string): Promise<any>;

/**
 * Get the status of a mob.
 *
 * Returns JSON with the mob state.
 */
export function mob_status(mob_id: string): Promise<any>;

/**
 * Subscribe to mob-wide events (all members, continuously updated).
 *
 * Returns a subscription handle. Use `poll_subscription(handle)` to drain
 * buffered events. Each call returns all events since the last poll.
 *
 * Unlike `mob_member_subscribe` which streams a single member's agent events,
 * this streams [`AttributedEvent`]s tagged with source meerkat_id and profile
 * for every member in the mob, automatically tracking roster changes.
 */
export function mob_subscribe_events(mob_id: string): Promise<number>;

/**
 * Unwire bidirectional trust between two meerkats.
 */
export function mob_unwire(mob_id: string, a: string, b: string): Promise<void>;

/**
 * Wire bidirectional trust between two meerkats.
 */
export function mob_wire(mob_id: string, a: string, b: string): Promise<void>;

/**
 * Drain and return all pending agent events from the last turn(s).
 *
 * Returns a JSON array of `AgentEvent` objects. Each call drains the buffer;
 * subsequent calls return `[]` until the next `start_turn` produces more events.
 */
export function poll_events(handle: number): string;

/**
 * Poll a subscription for new events.
 *
 * Returns a JSON array of event objects. Drains all buffered events
 * since the last poll. Non-blocking: returns `[]` if no new events.
 *
 * For per-member subscriptions (`mob_member_subscribe`), returns
 * `EventEnvelope<AgentEvent>` objects. For mob-wide subscriptions
 * (`mob_subscribe_events`), returns `AttributedEvent` objects with
 * `source`, `profile`, and `envelope` fields.
 */
export function poll_subscription(handle: number): string;

/**
 * Register a tool implementation from JavaScript.
 *
 * Call this BEFORE `init_runtime` or `init_runtime_from_config`.
 * The `callback` receives a JSON string of tool arguments and must return
 * a `Promise<string>` resolving to JSON `{"content": "...", "is_error": false}`.
 *
 * Example (JS):
 * ```js
 * register_tool_callback("shell", '{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}', shellFn);
 * ```
 */
export function register_tool_callback(name: string, description: string, schema_json: string, callback: any): void;

/**
 * Run a turn through the real meerkat agent loop via AgentFactory::build_agent().
 *
 * Returns JSON: `{ "text", "usage", "status", "session_id", "turns", "tool_calls" }`
 *
 * Convention: always resolves (Ok). Check `status` field for "completed" vs "failed".
 * Only rejects (Err) for infrastructure errors (session not found, build failure).
 * Agent-level errors (LLM failure, timeout) resolve with `status: "failed"` + `error` field.
 */
export function start_turn(handle: number, prompt: string, options_json: string): Promise<any>;

/**
 * Entry point invoked by JavaScript in a worker.
 */
export function task_worker_entry_point(ptr: number): void;

/**
 * Wire bidirectional comms trust between meerkats in DIFFERENT mobs.
 *
 * Unlike `mob_wire` (which is intra-mob), this establishes peer trust across
 * mob boundaries by accessing each member's comms runtime through the shared
 * session service. Both members must have comms enabled.
 */
export function wire_cross_mob(mob_a: string, meerkat_a: string, mob_b: string, meerkat_b: string): Promise<void>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly clear_tool_callbacks: () => void;
    readonly close_subscription: (a: number) => [number, number];
    readonly create_session: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly create_session_simple: (a: number, b: number) => [number, number, number];
    readonly destroy_session: (a: number) => [number, number];
    readonly get_session_state: (a: number) => [number, number, number, number];
    readonly init_runtime: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly init_runtime_from_config: (a: number, b: number) => [number, number, number];
    readonly inspect_mobpack: (a: number, b: number) => [number, number, number, number];
    readonly mob_cancel_flow: (a: number, b: number, c: number, d: number) => any;
    readonly mob_create: (a: number, b: number) => any;
    readonly mob_events: (a: number, b: number, c: number, d: number) => any;
    readonly mob_flow_status: (a: number, b: number, c: number, d: number) => any;
    readonly mob_inject_and_subscribe: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly mob_lifecycle: (a: number, b: number, c: number, d: number) => any;
    readonly mob_list: () => any;
    readonly mob_list_members: (a: number, b: number) => any;
    readonly mob_member_subscribe: (a: number, b: number, c: number, d: number) => any;
    readonly mob_respawn: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly mob_retire: (a: number, b: number, c: number, d: number) => any;
    readonly mob_run_flow: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly mob_send_message: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly mob_spawn: (a: number, b: number, c: number, d: number) => any;
    readonly mob_status: (a: number, b: number) => any;
    readonly mob_subscribe_events: (a: number, b: number) => any;
    readonly mob_unwire: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly mob_wire: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly poll_events: (a: number) => [number, number, number, number];
    readonly poll_subscription: (a: number) => [number, number, number, number];
    readonly register_tool_callback: (a: number, b: number, c: number, d: number, e: number, f: number, g: any) => [number, number];
    readonly start_turn: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly wire_cross_mob: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => any;
    readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
    readonly intounderlyingsource_cancel: (a: number) => void;
    readonly intounderlyingsource_pull: (a: number, b: any) => any;
    readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
    readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
    readonly intounderlyingbytesource_cancel: (a: number) => void;
    readonly intounderlyingbytesource_pull: (a: number, b: any) => any;
    readonly intounderlyingbytesource_start: (a: number, b: any) => void;
    readonly intounderlyingbytesource_type: (a: number) => number;
    readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
    readonly intounderlyingsink_abort: (a: number, b: any) => any;
    readonly intounderlyingsink_close: (a: number) => any;
    readonly intounderlyingsink_write: (a: number, b: any) => any;
    readonly task_worker_entry_point: (a: number) => [number, number];
    readonly wasm_bindgen__closure__destroy__he7596f27e48685e9: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__hcda34e93732b88fa: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h30cdf69ea633e0ba: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hc19d3fcc17c58b79: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__haaa9bd75819395b6: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hce552e0b4f79163a: (a: number, b: number) => number;
    readonly wasm_bindgen__convert__closures_____invoke__h96d6fed1334963ec: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h403743adb47429fd: (a: number, b: number) => void;
    readonly __wbindgen_malloc_command_export: (a: number, b: number) => number;
    readonly __wbindgen_realloc_command_export: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store_command_export: (a: number) => void;
    readonly __externref_table_alloc_command_export: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free_command_export: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc_command_export: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
