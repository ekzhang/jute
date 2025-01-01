import os
import datetime
import hashlib
import hmac
import json
import re
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Any

import zmq

kernel_dir = os.getenv("KERNEL_DIR")
if not kernel_dir:
    kernel_dir = Path("/usr/local/share/jupyter/kernels/python3")
else:
    kernel_dir = Path(kernel_dir)

kernel_json = json.loads((kernel_dir / "kernel.json").read_text())

print(f'Starting kernel "{kernel_json["display_name"]}" ...')

argv = list(kernel_json["argv"])
if re.match(r"python3?(?:\.exe)?$", argv[0], re.IGNORECASE):
    argv[0] = sys.executable

digest_key = str(uuid.uuid4())
Path("/tmp/kernel-1234.json").write_text(
    json.dumps(
        {
            "control_port": 50160,
            "shell_port": 57503,
            "transport": "tcp",
            "signature_scheme": "hmac-sha256",
            "stdin_port": 52597,
            "hb_port": 42540,
            "ip": "127.0.0.1",
            "iopub_port": 40885,
            "key": digest_key,
        }
    )
)

for i, arg in enumerate(argv):
    if "{connection_file}" in arg:
        argv[i] = arg.format(connection_file="/tmp/kernel-1234.json")


print(argv)
p = subprocess.Popen(argv)

time.sleep(1)

print("Connecting to kernel ...")

session_id = str(uuid.uuid4())

ctx = zmq.Context.instance()
shell_channel = ctx.socket(zmq.DEALER)
shell_channel.setsockopt(zmq.LINGER, 1000)
shell_channel.connect("tcp://127.0.0.1:57503")

iopub_channel = ctx.socket(zmq.SUB)
iopub_channel.setsockopt(zmq.LINGER, 1000)
iopub_channel.setsockopt(zmq.SUBSCRIBE, b"")
iopub_channel.connect("tcp://127.0.0.1:40885")


def msg_ready():
    return iopub_channel.poll(10, zmq.POLLIN)


def encode_msg(message_type: str, content: dict[str, Any]):
    msg_id = str(uuid.uuid4())
    header = {
        "msg_id": msg_id,
        "session": session_id,
        "username": "",
        "date": datetime.datetime.now().isoformat(),
        "msg_type": message_type,
        "version": "5.4",
    }
    msg_list: list[bytes] = [
        json.dumps(header).encode(),
        json.dumps({}).encode(),  # parent_header
        json.dumps({}).encode(),  # metadata
        json.dumps(content).encode(),
    ]
    mac = hmac.new(digest_key.encode(), digestmod=hashlib.sha256)
    for msg in msg_list:
        mac.update(msg)
    msg_list = [b"<IDS|MSG>", mac.hexdigest().encode()] + msg_list
    return msg_list


content = {
    "code": "print(99 * 10 + 1)",
    "silent": False,
    "store_history": True,
    "user_expressions": {},
    "allow_stdin": False,
    "stop_on_error": True,
}

shell_channel.send_multipart(encode_msg("execute_request", content))


state = "busy"
while state != "idle":
    if not msg_ready():
        continue
    msg_list = iopub_channel.recv_multipart()
    delim_idx = msg_list.index(b"<IDS|MSG>")
    header, parent_header, metadata, content = [
        json.loads(buf) for buf in msg_list[delim_idx + 2 :]
    ]
    match header["msg_type"]:
        case "status":
            state = content["execution_state"]
            print(f"Kernel state: {state}")
        case "stream":
            print(content["text"])
        case "execute_input":
            print("INFO: Kernel is executing", content["code"])
        case "execute_result":
            print(content["data"]["text/plain"])
        case "error":
            print("ERROR:", content["traceback"])
        case _:
            print(f"Unknown message type: {header['msg_type']}")


print("Terminating kernel ...")
p.terminate()
p.wait()
