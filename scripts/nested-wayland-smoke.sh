#!/usr/bin/env bash
set -euo pipefail
binary=${1:?native demo binary is required}
runtime="$PWD/target/wayland-runtime"
rm -rf "$runtime"
mkdir -m 700 -p "$runtime"
export XDG_RUNTIME_DIR="$runtime"
weston --backend=headless-backend.so --socket=gpui-ci --idle-time=0 >target/weston.log 2>&1 &
weston_pid=$!
app_pid=
cleanup() {
  [[ -z "${app_pid:-}" ]] || kill "$app_pid" 2>/dev/null || true
  kill "$weston_pid" 2>/dev/null || true
}
trap cleanup EXIT
for _ in {1..100}; do [[ -S "$runtime/gpui-ci" ]] && break; sleep .1; done
[[ -S "$runtime/gpui-ci" ]] || { cat target/weston.log; exit 1; }
WAYLAND_DISPLAY=gpui-ci WINIT_UNIX_BACKEND=wayland "$binary" >target/demo.log 2>&1 &
app_pid=$!
sleep 5
kill -0 "$app_pid" || { cat target/demo.log; exit 1; }
# A GUI event loop remaining healthy for five seconds proves native startup;
# CI terminates it deliberately rather than pretending a window exits itself.
