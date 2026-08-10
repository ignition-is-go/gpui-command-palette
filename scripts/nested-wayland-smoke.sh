#!/usr/bin/env bash
set -euo pipefail
binary=${1:?native demo binary is required}
runtime="$PWD/target/wayland-runtime"
rm -rf "$runtime"
mkdir -m 700 -p "$runtime"
export XDG_RUNTIME_DIR="$runtime"
Xvfb :99 -screen 0 1024x768x24 -nolisten tcp >target/xvfb.log 2>&1 &
xvfb_pid=$!
export DISPLAY=:99
weston --backend=x11-backend.so --socket=gpui-ci --idle-time=0 --width=900 --height=600 >target/weston.log 2>&1 &
weston_pid=$!
app_pid=
cleanup() {
  [[ -z "${app_pid:-}" ]] || kill "$app_pid" 2>/dev/null || true
  kill "$weston_pid" 2>/dev/null || true
  kill "$xvfb_pid" 2>/dev/null || true
}
trap cleanup EXIT
for _ in {1..100}; do [[ -S "$runtime/gpui-ci" ]] && break; sleep .1; done
[[ -S "$runtime/gpui-ci" ]] || { cat target/xvfb.log target/weston.log; exit 1; }
WAYLAND_DISPLAY=gpui-ci "$binary" >target/demo.log 2>&1 &
app_pid=$!
sleep 5
kill -0 "$app_pid" || { cat target/demo.log; exit 1; }
# A GUI event loop remaining healthy for five seconds proves native startup;
# CI terminates it deliberately rather than pretending a window exits itself.
