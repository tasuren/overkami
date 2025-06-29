import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";
import type { ApplicationWindow } from "./payload_os";

export async function getApplicationWindows(): Promise<ApplicationWindow[]> {
  return await invoke("get_application_windows");
}

export async function setDocumentEdited(edited: boolean): Promise<void> {
  if (platform() !== "macos")
    throw new Error("This function is only available on macOS.");

  await invoke("set_document_edited", { edited });
}
