#!/usr/bin/env bash
# Claude Code Stop hook — flashes Clawd mascot on Ditoo Pro, plays alert sound.
#
# Usage (Claude Code wires this up automatically via settings.json):
#   Fires when Claude finishes a turn. No arguments needed.
#
# Override sound:
#   Pass a path as $1, e.g. command: "hooks/notify.sh /path/to/sound.mp3"
#
# Override device:
#   Set DIVOOM_DEVICE env var, e.g. DIVOOM_DEVICE=aa-bb-cc-dd-ee-ff

# Drain stdin — Claude Code sends a JSON payload we don't need
cat > /dev/null &

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_DIR/target/release/divoom-ditoo-pro-controller"
MASCOT="$PROJECT_DIR/images/claude.divoom16"
DEFAULT_SOUND="$PROJECT_DIR/sounds/familymart.mp3"
SOUND="${1:-$DEFAULT_SOUND}"
DEVICE="${DIVOOM_DEVICE:-}"
[[ -n "$DEVICE" ]] || exit 0  # no device configured — skip silently
LOG="$PROJECT_DIR/hooks/notify.log"

# Exit silently if the binary or mascot image is not in place
[[ -x "$BINARY" && -f "$MASCOT" ]] || exit 0

# Fork the heavy work to background so Claude Code is not blocked.
# Bluetooth operations + audio playback take ~10s total.
(
  # Save current display state
  STATE=$("$BINARY" send "$DEVICE" get-settings 2>>"$LOG") || exit 0
  CHANNEL=$(echo "$STATE" | grep -o 'channel=[0-9]*' | cut -d= -f2)
  BRIGHTNESS=$(echo "$STATE" | grep -o 'brightness=[0-9]*' | cut -d= -f2)

  # Flash Clawd mascot
  "$BINARY" send "$DEVICE" animation "$MASCOT" 2>>"$LOG"

  # Play alert sound
  if [[ -f "$SOUND" ]]; then
    afplay "$SOUND"
  else
    printf '%s [notify] Sound not found: %s — put familymart.mp3 in sounds/\n' \
      "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$SOUND" >>"$LOG"
    afplay /System/Library/Sounds/Ping.aiff
  fi

  # Restore display to previous state
  "$BINARY" send "$DEVICE" restore-state "$CHANNEL" "$BRIGHTNESS" 2>>"$LOG"
) >> "$LOG" 2>&1 &

exit 0
