"""Del stream de una corrida, lo que se lee y se guarda.

Un turno de `--output-format stream-json` son 150 KB de ficheros leídos que no
dicen nada del experimento. Lo que dice algo es: si la pizarra apareció en la
lista de herramientas, qué llamadas hubo, qué contestó cada una, y qué respondió
el agente. Eso es lo que queda en `registros/`.

Uso: destila.py <directorio-de-turnos>      -> escribe registro.json e imprime el resumen
     destila.py --sesion <turno.jsonl>      -> imprime el session_id, para el --resume
"""

import json
import os
import sys

LA_PIZARRA = "mcp__plugin_flipchart_flipchart__"


def eventos(ruta):
    for line in open(ruta):
        try:
            yield json.loads(line)
        except Exception:
            continue


def session_id(ruta):
    sid = ""
    for e in eventos(ruta):
        sid = e.get("session_id") or sid
    return sid


def resultado_de(bloque):
    """El texto de un tool_result, que llega como str o como lista de bloques."""
    contenido = bloque.get("content")
    if isinstance(contenido, str):
        return contenido
    if isinstance(contenido, list):
        return " ".join(c.get("text", "") for c in contenido if isinstance(c, dict))
    return json.dumps(contenido, ensure_ascii=False)


def destila(ruta):
    turno = {"herramientas": [], "pizarra": [], "respuesta": None}
    for e in eventos(ruta):
        tipo = e.get("type")
        if e.get("subtype") == "init":
            turno["servidor"] = [m for m in e.get("mcp_servers", []) if "flip" in m["name"]]
            turno["ofrecidas"] = [t for t in e.get("tools", []) if "flip" in t]
        if tipo == "assistant":
            for c in e["message"].get("content", []):
                if c.get("type") != "tool_use":
                    continue
                turno["herramientas"].append(c["name"])
                if c["name"].startswith(LA_PIZARRA):
                    turno["pizarra"].append(
                        {"llamada": c["name"], "id": c["id"], "argumentos": c["input"]}
                    )
        if tipo == "user":
            for c in e.get("message", {}).get("content") or []:
                if not isinstance(c, dict) or c.get("type") != "tool_result":
                    continue
                for llamada in turno["pizarra"]:
                    if llamada["id"] == c.get("tool_use_id"):
                        llamada["contesto"] = resultado_de(c)
        if tipo == "result":
            turno["respuesta"] = e.get("result")
            turno["coste_usd"] = e.get("total_cost_usd")
            turno["modelo"] = list((e.get("modelUsage") or {}).keys())
    for llamada in turno["pizarra"]:
        llamada.pop("id", None)
    return turno


def main(argv):
    if argv[0] == "--sesion":
        print(session_id(argv[1]))
        return
    salida = argv[0]
    turnos = []
    for f in sorted(os.listdir(salida)):
        if f.startswith("turno-") and f.endswith(".jsonl"):
            turnos.append({"turno": f, **destila(os.path.join(salida, f))})
    with open(os.path.join(salida, "registro.json"), "w") as destino:
        json.dump(turnos, destino, ensure_ascii=False, indent=2)
    for t in turnos:
        dibujos = [
            f"{c['llamada'].removeprefix(LA_PIZARRA)}({c['argumentos'].get('view_id')})"
            f" -> {str(c.get('contesto'))[:60]}"
            for c in t["pizarra"]
        ]
        print(f"{t['turno']}: {len(t['pizarra'])} llamadas a la pizarra")
        for d in dibujos:
            print(f"    {d}")


if __name__ == "__main__":
    main(sys.argv[1:])
