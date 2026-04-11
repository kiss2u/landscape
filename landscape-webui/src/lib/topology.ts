import { NetDev, WLANTypeTag } from "./dev";
import { ZoneType } from "./service_ipconfig";

export type BridgeAttachIssue =
  | "device_not_found"
  | "bridge_connection_rule"
  | "device_has_parent"
  | "connect_unavailable"
  | "wifi_client_mode_warning";

export function canManageBridgeAttachment(dev: NetDev) {
  if (dev.dev_kind === "bridge") {
    return false;
  }

  return dev.zone_type !== ZoneType.Wan;
}

export function getBridgeAttachIssue(
  controller: NetDev | undefined,
  child: NetDev | undefined,
): BridgeAttachIssue | undefined {
  if (!controller || !child) {
    return "device_not_found";
  }

  if (controller.index === child.index) {
    return "bridge_connection_rule";
  }

  if (controller.dev_kind !== "bridge" || child.dev_kind === "bridge") {
    return "bridge_connection_rule";
  }

  if (child.controller_id !== undefined) {
    return "device_has_parent";
  }

  if (child.zone_type !== ZoneType.Undefined) {
    return "connect_unavailable";
  }

  if (child.wifi_info && child.wifi_info.wifi_type.t !== WLANTypeTag.Ap) {
    return "wifi_client_mode_warning";
  }

  return undefined;
}
