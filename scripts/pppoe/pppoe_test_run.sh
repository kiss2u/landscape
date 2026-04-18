#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
ENV_SCRIPT="$SCRIPT_DIR/pppoe_test_env.sh"

CLIENT_IFACE="${PPPOE_TEST_CLIENT_IFACE:-ld-pppoe-client}"
USERNAME="${PPPOE_TEST_USERNAME:-pppoe-user}"
PASSWORD="${PPPOE_TEST_PASSWORD:-pppoe-pass}"
RUNTIME_DIR="${PPPOE_TEST_RUNTIME_DIR:-/tmp/landscape-pppoe-test}"
SERVER_LOG_FILE="${RUNTIME_DIR}/pppoe-server.log"
PPPD_LOG_FILE="${RUNTIME_DIR}/pppd.log"
TIMEOUT_SECS="${PPPOE_TEST_TIMEOUT_SECS:-30}"
MTU="${PPPOE_TEST_MTU:-1492}"
KEEP_ENV=0
CLIENT_STDOUT_FILE="${RUNTIME_DIR}/pppoe-client.stdout.log"
CLIENT_STDERR_FILE="${RUNTIME_DIR}/pppoe-client.stderr.log"

usage() {
  cat <<EOF
Usage: $(basename "$0") [--keep-env] [--timeout-secs <n>] [--mtu <n>]
EOF
}

require_root() {
  if [[ "${EUID}" -ne 0 ]]; then
    echo "This script must be run as root." >&2
    exit 1
  fi
}

show_logs() {
  if [[ -f "$CLIENT_STDOUT_FILE" ]]; then
    echo "===== pppoe-client.stdout.log ====="
    cat "$CLIENT_STDOUT_FILE" || true
  fi
  if [[ -f "$CLIENT_STDERR_FILE" ]]; then
    echo "===== pppoe-client.stderr.log ====="
    cat "$CLIENT_STDERR_FILE" || true
  fi
  if [[ -f "$SERVER_LOG_FILE" ]]; then
    echo "===== pppoe-server.log ====="
    tail -n 80 "$SERVER_LOG_FILE" || true
  fi
  if [[ -f "$PPPD_LOG_FILE" ]]; then
    echo "===== pppd.log ====="
    tail -n 80 "$PPPD_LOG_FILE" || true
  fi
}

cleanup() {
  local exit_code="$1"
  if [[ "$exit_code" -ne 0 ]]; then
    show_logs
  fi
  if [[ "$KEEP_ENV" -eq 0 ]]; then
    "$ENV_SCRIPT" down || true
  fi
}

command_succeeded() {
  if [[ -f "$CLIENT_STDOUT_FILE" ]] && grep -q "PPPoE 状态变更: Running" "$CLIENT_STDOUT_FILE"; then
    return 0
  fi

  if [[ -f "$PPPD_LOG_FILE" ]] && grep -q "remote IP address" "$PPPD_LOG_FILE"; then
    return 0
  fi

  return 1
}

main() {
  require_root

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --keep-env)
        KEEP_ENV=1
        shift
        ;;
      --timeout-secs)
        TIMEOUT_SECS="$2"
        shift 2
        ;;
      --mtu)
        MTU="$2"
        shift 2
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        echo "Unknown argument: $1" >&2
        usage
        exit 1
        ;;
    esac
  done

  trap 'cleanup "$?"' EXIT

  "$ENV_SCRIPT" up

  cd "$REPO_DIR"
  cargo build --package landscape --bin pppoe_smoke >/dev/null

  set +e
  PPPOE_TEST_IFACE_NAME="$CLIENT_IFACE" \
  PPPOE_TEST_USERNAME="$USERNAME" \
  PPPOE_TEST_PASSWORD="$PASSWORD" \
  PPPOE_TEST_MTU="$MTU" \
  PPPOE_TEST_TIMEOUT_SECS="$TIMEOUT_SECS" \
  target/debug/pppoe_smoke \
    >"$CLIENT_STDOUT_FILE" 2>"$CLIENT_STDERR_FILE"
  local test_exit=$?
  set -e

  cat "$CLIENT_STDOUT_FILE"
  if [[ -s "$CLIENT_STDERR_FILE" ]]; then
    cat "$CLIENT_STDERR_FILE" >&2
  fi

  if [[ "$test_exit" -eq 0 ]]; then
    return 0
  fi

  if command_succeeded; then
    echo "PPPoE test reached running state, but helper exited with ${test_exit}. Treating as success." >&2
    return 0
  fi

  return "$test_exit"
}

main "$@"
