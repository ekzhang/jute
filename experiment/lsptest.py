import subprocess
import json
import sys
import threading
import re
import time
from typing import Any, Callable

MessageDict = dict[str, Any]
ServerCallback = Callable[[MessageDict], None]


def write_message(proc: subprocess.Popen[bytes], message_obj: MessageDict) -> None:
    """
    Encode a Python dictionary as a JSON-RPC 2.0 message, prepend
    the required 'Content-Length' header, and write it to the server.

    :param proc: The subprocess running the language server.
    :param message_obj: The message to send (in Python dict form).
    """
    message_json: str = json.dumps(message_obj, ensure_ascii=False)
    content: bytes = message_json.encode("utf-8")
    headers: str = f"Content-Length: {len(content)}\r\n\r\n"
    proc.stdin.write(headers.encode("utf-8"))
    proc.stdin.write(content)
    proc.stdin.flush()


def read_messages(
    proc: subprocess.Popen[bytes], on_message_callback: ServerCallback
) -> None:
    """
    Continuously read stdout from the server (blocking).
    Parse each LSP message's 'Content-Length' header, then read
    that many bytes, parse as JSON, and call the callback.

    :param proc: The subprocess running the language server.
    :param on_message_callback: A function that will handle each parsed message.
    """
    while True:
        line: bytes = proc.stdout.readline()
        if not line:
            print("Server closed the connection.")
            break

        line_str: str = line.decode("utf-8", errors="replace")
        match = re.match(r"Content-Length:\s+(\d+)", line_str, re.IGNORECASE)
        if match:
            content_length: int = int(match.group(1))
            # Read the blank line (usually \r\n)
            _ = proc.stdout.readline()
            payload: bytes = proc.stdout.read(content_length)

            try:
                message: MessageDict = json.loads(payload.decode("utf-8"))
            except json.JSONDecodeError:
                print("Failed to decode JSON from server.")
                continue

            on_message_callback(message)
        else:
            # Non-LSP logs or unexpected lines could appear here
            if line_str.strip():
                print("Server (non-LSP):", line_str.strip())


def main() -> None:
    """
    1. Start basedpyright-langserver in --stdio mode
    2. Send minimal LSP init with sys.executable as pythonPath
    3. Simulate textDocument/didOpen
    4. Send a hover request for the `print` function
    5. Shutdown the language server
    """
    # Start the server as a subprocess. Make sure 'basedpyright-langserver' is on your PATH.
    cmd: list[str] = ["basedpyright-langserver", "--stdio"]
    proc: subprocess.Popen[bytes] = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        bufsize=0,
    )

    def on_server_message(message: MessageDict) -> None:
        """
        Handle/print incoming LSP messages from the server.
        """
        print(f"\n[Server -> Client]\n{json.dumps(message, indent=2)}")

    # Start a thread to read messages continuously
    reader_thread: threading.Thread = threading.Thread(
        target=read_messages, args=(proc, on_server_message), daemon=True
    )
    reader_thread.start()

    # --------------------------------------------------------------------------
    # 1) Send "initialize" request
    # --------------------------------------------------------------------------
    initialize_request: MessageDict = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": None,
            "rootUri": "file:///fake/project",
            "capabilities": {
                # Specify supported client capabilities if needed
            },
            "initializationOptions": {
                "pythonPath": sys.executable,  # Use the Python running this script
            },
            "trace": "off",
        },
    }
    write_message(proc, initialize_request)

    # --------------------------------------------------------------------------
    # 2) Send "initialized" notification
    # --------------------------------------------------------------------------
    initialized_notification: MessageDict = {
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {},
    }
    write_message(proc, initialized_notification)

    # Give the server a moment to respond
    time.sleep(1)
    print()

    # --------------------------------------------------------------------------
    # 3) textDocument/didOpen
    # Simulate opening a .py file with some Python content
    # --------------------------------------------------------------------------
    did_open_request: MessageDict = {
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///fake/path/sample.py",  # Fake file path
                "languageId": "python",
                "version": 1,
                "text": "print('Hello, world!')\n",  # Python code
            }
        },
    }
    write_message(proc, did_open_request)

    # --------------------------------------------------------------------------
    # 4) Hover request: textDocument/hover over "print"
    # --------------------------------------------------------------------------
    hover_request: MessageDict = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/hover",
        "params": {
            "textDocument": {
                "uri": "file:///fake/path/sample.py"  # Match the opened file
            },
            "position": {
                # Hover over the first character of "print"
                "line": 0,
                "character": 0,
            },
        },
    }
    write_message(proc, hover_request)

    # Allow some time for the hover response
    time.sleep(1)

    # --------------------------------------------------------------------------
    # 5) Shutdown request & exit notification
    # --------------------------------------------------------------------------
    shutdown_request: MessageDict = {"jsonrpc": "2.0", "id": 3, "method": "shutdown"}
    write_message(proc, shutdown_request)

    exit_notification: MessageDict = {"jsonrpc": "2.0", "method": "exit"}
    write_message(proc, exit_notification)

    # Wait for server process to exit
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        print("Server did not shut down cleanly within 5 seconds.")


if __name__ == "__main__":
    main()
