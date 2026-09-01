#!/usr/bin/env python3
import json, sys
for line in open(sys.argv[1]):
    line = line.strip()
    if not line: continue
    try: o = json.loads(line)
    except Exception: continue
    if o.get("type") == "system" and o.get("subtype") == "init":
        print("mcp_servers:", json.dumps(o.get("mcp_servers")))
        sc = o.get("slash_commands") or []
        print("slash toy:", [c for c in sc if "toy" in c or "check" in c])
        tl = o.get("tools") or []
        print("tools toy:", [t for t in tl if "toy" in t.lower()])
        print("n_tools:", len(tl), "n_slash:", len(sc))
    elif o.get("type") == "result":
        print("result usage:", json.dumps(o.get("usage", {}).get("cache_creation_input_tokens")), "cost", o.get("total_cost_usd"))
