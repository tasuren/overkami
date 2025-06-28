import { invoke } from "@tauri-apps/api/core";
import type { ErrorContext } from "./payload_common";
import type { Config } from "./payload_config";
import { errorMessage } from "./utils";

export async function getConfig(): Promise<Config> {
  return await invoke("get_config");
}

export async function saveConfig(config: Config): Promise<void> {
  console.log(config);
  try {
    return await invoke("save_config", { config });
  } catch (error) {
    errorMessage(error as ErrorContext);
  }
}
