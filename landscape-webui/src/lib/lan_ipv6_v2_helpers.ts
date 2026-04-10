import type { PrefixParentSource } from "@landscape-router/types/api/schemas";

export type SourceType = "static" | "pd";
export type SourceKind = "ra" | "na" | "pd";

export function sourceTypeFromParent(parent: PrefixParentSource): SourceType {
  return parent.t === "static" ? "static" : "pd";
}

export function groupParentLabel(parent: PrefixParentSource) {
  if (parent.t === "static") {
    return `${parent.base_prefix}/${parent.parent_prefix_len}`;
  }
  return parent.depend_iface;
}
