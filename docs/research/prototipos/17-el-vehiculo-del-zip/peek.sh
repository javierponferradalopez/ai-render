#!/bin/bash
# peek.sh <patron> [ventana_bytes] [n_ocurrencia]
B=/opt/homebrew/Caskroom/claude-code/2.1.228/claude
pat="$1"; win="${2:-6000}"; nth="${3:-1}"
off=$(LC_ALL=C grep -a -b -o -- "$pat" "$B" | sed -n "${nth}p" | cut -d: -f1)
[ -z "$off" ] && { echo "NO MATCH: $pat"; exit 1; }
start=$(( off - win/2 )); [ $start -lt 0 ] && start=0
dd if="$B" bs=1 skip=$start count=$win 2>/dev/null \
  | LC_ALL=C strings -n 4 \
  | LC_ALL=C grep -v '^[[:space:]]*$'
