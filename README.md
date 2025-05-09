# Landscape - Linux Router Configuration Tool

Landscape is a Web UI-based tool that allows you to easily configure your favorite Linux distribution as a router.  

> Developed using Rust / eBPF / AF_PACKET.  

[中文文档](./README.zh.md) | [English](./README.md)

## Screenshots
![](docs/images/1.png)

---
## Features
> ✅ Implemented & Tested  
> ⚠ Implemented but Untested  
> ❌ Not Implemented  

### IP Configuration
- **Static IP**
  - ✅ Custom IP assignment
  - ✅ Gateway configuration with default route
- **DHCP Client**
  - ✅ Hostname specification
  - ❌ Custom DHCP options
- **PPPoE**
  - *PPPD Version*
    - ✅ Default route configuration
    - ⚠ Multi-interface dialing
    - ✅ Interface selection
  - *eBPF Version*
    - ✅ Core protocol implementation
    - ❌ MTU issues from GRO/GSO (unresolved)
- **DHCP Server**
  - ✅ IP leasing and renewal
  - ✅ Custom gateway/subnet configuration
  - ✅ MAC-IP binding
  - ✅ IP allocation dashboard
- **IPv6 Support**
  - ✅ DHCPv6-PD prefix delegation
  - ✅ RA prefix advertisement

### Traffic Control
- ✅ QoS-based flow classification
- ✅ Per-flow DNS configuration with caching
- ✅ Traffic handling (direct/drop/redirect)
- ❌ Connection tracking tags
- ✅ GeoIP-based rules (with `geoip.dat` support)
- ✅ DNS action override capability
- ❌ Automatic GeoIP updates

### DNS Management
- ✅ DoH/DoT upstream support
- ✅ Domain-specific DNS routing
- ✅ DNS hijacking (A records)
- ❌ Multi-record hijacking
- ✅ DNS-based traffic tagging
- ✅ GeoSite support
- ❌ Automatic GeoSite updates
- ❌ Docker container name resolution

### NAT (eBPF)
- ✅ Basic NAT functionality
- ⚠ Port forwarding (UI incomplete)
- ✅ NAT hole-punching prevention

### Metrics
- ✅ 5-second connection metrics
- ✅ Live connection monitoring
- ❌ Metrics API export

### Docker Integration
- ✅ Container management
- ⚠ Image pulling
- ✅ TProxy container routing

### WiFi Management
- ✅ Radio control via `iw`
- ✅ AP creation via `hostapd`
- ❌ Client mode connection

### Storage
- ❌ Database-backed configuration
- ❌ External metrics storage

### Miscellaneous
- ✅ Login interface
- ❌ English UI localization
- ❌ NIC XPS/RPS optimization

---

## Getting Started

### System Requirements
- Linux kernel ≥ 6.1
- Dependencies: `iptables` (for PPPoE MSS clamping), `docker`

### Standard Deployment
1. Create config directory:
   ```bash
   mkdir -p ~/.landscape-router
   ```
2. (Optional) Create `landscape_init.toml` (see [Quick Start](https://landscape.whileaway.dev/quick.html))

3. Launch service (default port: 6300):
   ```bash
   ./landscape-webserver
   ```

### Docker Compose Quick Start  
See [Quick Start Guide](https://landscape.whileaway.dev/quick.html)

### Armbian Integration
See [Armbian Integration](https://landscape.whileaway.dev/compilation/armbian.html)

---

## Building from Source
Refer to [Build Instructions](https://landscape.whileaway.dev/compilation/) or [Cross-Compilation Guide](https://landscape.whileaway.dev/compilation/cross.html)

## License

- `landscape-ebpf`: [GPL-2.0](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html)
- Other components: [GPL-3.0](https://www.gnu.org/licenses/gpl-3.0.html)

---

If you have any suggestions or issues, feel free to submit feedback on the [issues](./issues/new) page.  
