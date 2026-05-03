/** Tagged union matching the Rust CustomDhcpOption serde format. */
export type CustomDhcpOption =
  | { TFTPServerName: string }
  | { BootfileName: string }
  | { VendorExtensions: string }
  | { RelayAgentInformation: RelayAgentInfo };

export interface RelayAgentInfo {
  /** Sub-options encoded as RelayAgentInformation */
  [key: string]: unknown;
}

export const DHCP_OPTION_LABELS: Record<number, string> = {
  12: "Host Name (12)",
  15: "Domain Name (15)",
  28: "Broadcast Address (28)",
  43: "Vendor Extensions (43)",
  66: "TFTP Server Name (66)",
  67: "Bootfile Name (67)",
  82: "Relay Agent Information (82)",
};

export const DHCP_FILTER_OPTIONS = [12, 15, 28, 43, 66, 67, 82] as const;
