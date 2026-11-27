#!/usr/bin/env python3
"""LocalSend (protocol v2), the sending half, with the standard library only.

  localsend.py discover [seconds]   one JSON line per device found
  localsend.py send <ip> <port> <http|https> <file>...
                                    prints: waiting / progress <done> <total> / done / error <why>
"""
import hashlib
import json
import os
import secrets
import socket
import ssl
import struct
import sys
import time
import http.client

GROUP, PORT = "224.0.0.167", 53317
ME = {
    "alias": f"octopus · {socket.gethostname()}",
    "version": "2.1",
    "deviceModel": "octopus",
    "deviceType": "desktop",
    "fingerprint": hashlib.sha256(socket.gethostname().encode()).hexdigest(),
    "port": PORT,
    "protocol": "https",
    "download": False,
}


def discover(seconds: float) -> None:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    if hasattr(socket, "SO_REUSEPORT"):
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
    sock.bind(("", PORT))
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, struct.pack("4sl", socket.inet_aton(GROUP), socket.INADDR_ANY))
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)
    sock.settimeout(0.5)
    announce = json.dumps({**ME, "announce": True}).encode()
    seen = set()
    end = time.time() + seconds
    next_announce = 0.0
    while time.time() < end:
        if time.time() >= next_announce:
            sock.sendto(announce, (GROUP, PORT))
            next_announce = time.time() + 1.5
        try:
            data, (ip, _) = sock.recvfrom(65535)
        except socket.timeout:
            continue
        try:
            info = json.loads(data)
        except ValueError:
            continue
        if info.get("fingerprint") in (ME["fingerprint"], None) or info["fingerprint"] in seen:
            continue
        seen.add(info["fingerprint"])
        print(json.dumps({
            "ip": ip,
            "alias": info.get("alias", ip),
            "model": info.get("deviceModel") or "",
            "type": info.get("deviceType") or "desktop",
            "port": info.get("port", PORT),
            "protocol": info.get("protocol", "https"),
        }), flush=True)


def connect(ip: str, port: int, protocol: str, timeout: float):
    if protocol == "https":
        return http.client.HTTPSConnection(ip, port, timeout=timeout, context=ssl._create_unverified_context())
    return http.client.HTTPConnection(ip, port, timeout=timeout)


def send(ip: str, port: int, protocol: str, paths: list[str]) -> None:
    files = {}
    for path in paths:
        fid = secrets.token_hex(8)
        files[fid] = {"id": fid, "fileName": os.path.basename(path), "size": os.path.getsize(path), "fileType": "application/octet-stream"}
    total = sum(f["size"] for f in files.values())
    body = json.dumps({"info": ME, "files": files})
    print("waiting", flush=True)
    # the other side may take a while to accept
    conn = connect(ip, port, protocol, 120)
    conn.request("POST", "/api/localsend/v2/prepare-upload", body, {"Content-Type": "application/json"})
    reply = conn.getresponse()
    data = reply.read()
    if reply.status == 204:
        print("done", flush=True)
        return
    if reply.status != 200:
        print(f"error {'recusado' if reply.status == 403 else reply.status}", flush=True)
        return
    session = json.loads(data)
    tokens = session["files"]
    done = 0
    by_name = dict(zip(files.keys(), paths))
    for fid, token in tokens.items():
        path = by_name[fid]
        size = files[fid]["size"]
        up = connect(ip, port, protocol, 60)
        up.putrequest("POST", f"/api/localsend/v2/upload?sessionId={session['sessionId']}&fileId={fid}&token={token}")
        up.putheader("Content-Length", str(size))
        up.putheader("Content-Type", "application/octet-stream")
        up.endheaders()
        with open(path, "rb") as f:
            while chunk := f.read(256 * 1024):
                up.send(chunk)
                done += len(chunk)
                print(f"progress {done} {total}", flush=True)
        r = up.getresponse()
        r.read()
        if r.status != 200:
            print(f"error {r.status}", flush=True)
            return
    print("done", flush=True)


if __name__ == "__main__":
    try:
        if sys.argv[1] == "discover":
            discover(float(sys.argv[2]) if len(sys.argv) > 2 else 4)
        elif sys.argv[1] == "send":
            send(sys.argv[2], int(sys.argv[3]), sys.argv[4], sys.argv[5:])
    except Exception as err:  # noqa: BLE001 - reported to the window
        print(f"error {err}", flush=True)
