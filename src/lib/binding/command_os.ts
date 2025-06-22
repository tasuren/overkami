import { invoke } from "@tauri-apps/api/core";
import type { ApplicationWindow } from "./payload_os";

export async function getApplicationWindows(): Promise<ApplicationWindow[]> {
  return await invoke("get_application_windows");
}
