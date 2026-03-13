import type { CallerIdentityResponse } from "@landscape-router/types/api/schemas";

import { get_client_caller } from "@/api/client";

export type IfaceDisableRiskCaller = CallerIdentityResponse;

export async function get_iface_disable_risk_caller(
  ifaceName: string,
): Promise<IfaceDisableRiskCaller | null> {
  try {
    const caller = await get_client_caller();
    if (caller.iface_name !== ifaceName) {
      return null;
    }

    return caller;
  } catch {
    return null;
  }
}
