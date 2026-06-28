/**
 * Low-level IPC bridge — the ONLY place `invoke` is called. React components
 * and stores import the typed `api` from ./api, never `invoke` directly.
 */
import { invoke } from "@tauri-apps/api/core";

/** True when running inside the Tauri shell (vs. a plain browser/dev preview). */
export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Raised when a service command fails; carries the original message. */
export class ServiceError extends Error {
  constructor(
    public readonly command: string,
    message: string
  ) {
    super(message);
    this.name = "ServiceError";
  }
}

/** Invoke a service command, normalizing errors to `ServiceError`. */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    throw new ServiceError(command, message);
  }
}
