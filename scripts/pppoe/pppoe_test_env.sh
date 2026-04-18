#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

NS_NAME="${PPPOE_TEST_NS:-ld-pppoe-test}"
CLIENT_IFACE="${PPPOE_TEST_CLIENT_IFACE:-ld-pppoe-client}"
SERVER_IFACE="${PPPOE_TEST_SERVER_IFACE:-ld-pppoe-server}"
CLIENT_MAC="${PPPOE_TEST_CLIENT_MAC:-02:00:00:00:00:11}"
SERVER_MAC="${PPPOE_TEST_SERVER_MAC:-02:00:00:00:00:22}"
RUNTIME_DIR="${PPPOE_TEST_RUNTIME_DIR:-/tmp/landscape-pppoe-test}"
ETC_NETNS_DIR="/etc/netns/${NS_NAME}"
PPPOE_OPTIONS_FILE="${ETC_NETNS_DIR}/ppp/pppoe-server-options"
PAP_SECRETS_FILE="${ETC_NETNS_DIR}/ppp/pap-secrets"
CHAP_SECRETS_FILE="${ETC_NETNS_DIR}/ppp/chap-secrets"
SERVER_LOG_FILE="${RUNTIME_DIR}/pppoe-server.log"
PPPD_LOG_FILE="${RUNTIME_DIR}/pppd.log"
SERVER_PID_FILE="${RUNTIME_DIR}/pppoe-server.pid"
SERVER_EXEC_PID_FILE="${RUNTIME_DIR}/pppoe-server.exec.pid"
USERNAME="${PPPOE_TEST_USERNAME:-pppoe-user}"
PASSWORD="${PPPOE_TEST_PASSWORD:-pppoe-pass}"
LOCAL_IP="${PPPOE_TEST_LOCAL_IP:-10.0.0.1}"
REMOTE_IP_START="${PPPOE_TEST_REMOTE_IP_START:-10.0.0.100}"
MAX_SESSIONS="${PPPOE_TEST_MAX_SESSIONS:-8}"
AC_NAME="${PPPOE_TEST_AC_NAME:-landscape-test-ac}"
SERVICE_NAME="${PPPOE_TEST_SERVICE_NAME:-landscape-test-service}"

usage() {
  cat <<EOF
Usage: $(basename "$0") <up|down|status>

Environment variables:
  PPPOE_TEST_NS
  PPPOE_TEST_CLIENT_IFACE
  PPPOE_TEST_SERVER_IFACE
  PPPOE_TEST_USERNAME
  PPPOE_TEST_PASSWORD
  PPPOE_TEST_LOCAL_IP
  PPPOE_TEST_REMOTE_IP_START
EOF
}

require_root() {
  if [[ "${EUID}" -ne 0 ]]; then
    echo "This script must be run as root." >&2
    exit 1
  fi
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

runtime_prepare() {
  mkdir -p "$RUNTIME_DIR" "$ETC_NETNS_DIR/ppp"
  chmod 700 "$ETC_NETNS_DIR/ppp"

  cat > "$PPPOE_OPTIONS_FILE" <<EOF
noauth
require-pap
refuse-chap
refuse-mschap
refuse-mschap-v2
+ipv6
ipv6cp-accept-local
ipv6cp-accept-remote
lcp-echo-interval 5
lcp-echo-failure 3
mtu 1492
mru 1492
debug
logfile ${PPPD_LOG_FILE}
EOF

  cat > "$PAP_SECRETS_FILE" <<EOF
"${USERNAME}" * "${PASSWORD}" *
EOF

  : > "$CHAP_SECRETS_FILE"
  chmod 600 "$PAP_SECRETS_FILE" "$CHAP_SECRETS_FILE"
}

kill_server() {
  if [[ -f "$SERVER_EXEC_PID_FILE" ]]; then
    local exec_pid
    exec_pid="$(cat "$SERVER_EXEC_PID_FILE")"
    if [[ -n "$exec_pid" ]] && kill -0 "$exec_pid" >/dev/null 2>&1; then
      kill "$exec_pid" >/dev/null 2>&1 || true
      wait "$exec_pid" 2>/dev/null || true
    fi
  fi

  if [[ -f "$SERVER_PID_FILE" ]]; then
    local daemon_pid
    daemon_pid="$(cat "$SERVER_PID_FILE")"
    if [[ -n "$daemon_pid" ]] && kill -0 "$daemon_pid" >/dev/null 2>&1; then
      kill "$daemon_pid" >/dev/null 2>&1 || true
    fi
  fi
}

env_down() {
  kill_server

  if ip netns list | grep -q "^${NS_NAME}\b"; then
    ip netns pids "$NS_NAME" | xargs -r kill >/dev/null 2>&1 || true
    ip netns del "$NS_NAME" || true
  fi

  if ip link show "$CLIENT_IFACE" >/dev/null 2>&1; then
    ip link del "$CLIENT_IFACE" || true
  fi

  rm -rf "$ETC_NETNS_DIR" "$RUNTIME_DIR"
}

wait_for_server() {
  local tries=0
  while (( tries < 20 )); do
    if [[ -f "$SERVER_EXEC_PID_FILE" ]]; then
      local exec_pid
      exec_pid="$(cat "$SERVER_EXEC_PID_FILE")"
      if [[ -n "$exec_pid" ]] && kill -0 "$exec_pid" >/dev/null 2>&1; then
        return 0
      fi
    fi
    sleep 0.2
    tries=$((tries + 1))
  done

  echo "pppoe-server failed to start. See ${SERVER_LOG_FILE}" >&2
  return 1
}

env_up() {
  env_down >/dev/null 2>&1 || true
  runtime_prepare

  ip netns add "$NS_NAME"
  ip link add "$CLIENT_IFACE" type veth peer name "$SERVER_IFACE"
  ip link set "$SERVER_IFACE" netns "$NS_NAME"

  ip link set dev "$CLIENT_IFACE" address "$CLIENT_MAC"
  ip link set "$CLIENT_IFACE" up

  ip netns exec "$NS_NAME" ip link set lo up
  ip netns exec "$NS_NAME" ip link set dev "$SERVER_IFACE" address "$SERVER_MAC"
  ip netns exec "$NS_NAME" ip link set "$SERVER_IFACE" up

  ip netns exec "$NS_NAME" \
    pppoe-server \
    -F \
    -X "$SERVER_PID_FILE" \
    -I "$SERVER_IFACE" \
    -C "$AC_NAME" \
    -S "$SERVICE_NAME" \
    -L "$LOCAL_IP" \
    -R "$REMOTE_IP_START" \
    -N "$MAX_SESSIONS" \
    -O /etc/ppp/pppoe-server-options \
    >"$SERVER_LOG_FILE" 2>&1 &

  echo "$!" > "$SERVER_EXEC_PID_FILE"
  wait_for_server
  env_status
}

env_status() {
  local client_ifindex="unknown"
  if [[ -f "/sys/class/net/${CLIENT_IFACE}/ifindex" ]]; then
    client_ifindex="$(cat "/sys/class/net/${CLIENT_IFACE}/ifindex")"
  fi

  echo "Namespace: ${NS_NAME}"
  echo "Client iface: ${CLIENT_IFACE} (ifindex=${client_ifindex}, mac=${CLIENT_MAC})"
  echo "Server iface: ${SERVER_IFACE} (netns=${NS_NAME}, mac=${SERVER_MAC})"
  echo "Credentials: ${USERNAME}/${PASSWORD}"
  echo "Local IP: ${LOCAL_IP}"
  echo "Remote IP start: ${REMOTE_IP_START}"
  echo "Server log: ${SERVER_LOG_FILE}"
  echo "PPPD log: ${PPPD_LOG_FILE}"

  if ip netns list | grep -q "^${NS_NAME}\b"; then
    ip -brief link show "$CLIENT_IFACE"
    ip netns exec "$NS_NAME" ip -brief link show "$SERVER_IFACE"
  fi
}

main() {
  require_root
  local action="${1:-}"
  case "$action" in
    up)
      require_cmd ip
      require_cmd pppoe-server
      require_cmd pppd
      env_up
      ;;
    down)
      require_cmd ip
      env_down
      ;;
    status)
      require_cmd ip
      env_status
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main "$@"
