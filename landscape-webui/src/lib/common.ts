import { MessageApi } from "naive-ui";

export class Range {
  start: number;
  end: number;

  constructor(start: number, end: number) {
    this.start = start;
    this.end = end;
  }
}

export type KeyValuePair = { key: string; value: string };

export class SimpleResult {
  success: boolean;

  constructor(obj?: { success?: boolean }) {
    this.success = obj?.success ?? false;
  }
}

export const LANDSCAPE_TOKEN_KEY = "LANDSCAPE_TOKEN";

export async function copy_context_to_clipboard(
  message: MessageApi,
  content: string
) {
  try {
    await navigator.clipboard.writeText(content);
    message.success("copy success");
  } catch (e) {
    message.error("copy fail");
  }
}

export async function read_context_from_clipboard(): Promise<string> {
  return await navigator.clipboard.readText();
}
