#!/usr/bin/env python3
"""Extrae del stream el tamano de entrada del primer turno y los inventarios."""
import json, sys
for line in open(sys.argv[1]):
    try: o = json.loads(line)
    except Exception: continue
    if o.get("type") == "system" and o.get("subtype") == "init":
        sc = o.get("slash_commands") or []; tl = o.get("tools") or []
        n_toy_s = len([c for c in sc if "toy" in c or "check" in c])
        n_toy_t = len([t for t in tl if "toy" in t.lower()])
        print("n_slash=%d n_tools=%d toy_slash=%d toy_tools=%d" % (len(sc), len(tl), n_toy_s, n_toy_t), end=" ")
    if o.get("type") == "result":
        u = o.get("usage", {})
        tot = u.get("input_tokens",0)+u.get("cache_creation_input_tokens",0)+u.get("cache_read_input_tokens",0)
        mu = o.get("modelUsage", {})
        mi = sum(v.get("inputTokens",0)+v.get("cacheCreationInputTokens",0)+v.get("cacheReadInputTokens",0) for v in mu.values())
        print("input_total=%d modelUsage_total=%d out=%d" % (tot, mi, u.get("output_tokens",0)))
