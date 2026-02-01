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
  content: string,
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

export function mask_string(value: string | undefined | null): string {
  if (!value) return "***";

  const length = value.length;

  if (length <= 4) {
    return value.substring(0, 1) + "*****";
  } else if (length <= 10) {
    return value.substring(0, 3) + "*****";
  } else {
    const start = Math.floor((length - 5) / 2);
    return "*****" + value.substring(start, start + 5) + "*****";
  }
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
