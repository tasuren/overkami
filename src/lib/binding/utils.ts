import { message } from "@tauri-apps/plugin-dialog";
import type { ErrorContext } from "./payload_common";

export async function errorMessage(error: ErrorContext) {
  if (typeof error === "string") {
    await message(`不明なエラーが発生しました。\n詳細: ${error}`, {
      kind: "error",
    });
  } else {
    await message(`${error.message}\n詳細: ${error.detail}`, { kind: "error" });
  }
}
