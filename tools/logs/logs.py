#!/usr/bin/env python3
"""Stream the Pico's USB serial logs and stop cleanly on Ctrl-C.

While streaming, the terminal is put in a mode where Ctrl-C is read as a
keystroke instead of SIGINT. That keeps the parent `make` from printing an
"Interrupt" error when the user stops the logs.
"""

import glob
import os
import select
import signal
import sys
import termios
import time

import serial

# How long to wait for the port to appear (a freshly flashed board may need a
# moment, or a replug, before it enumerates).
WAIT_SECONDS = 20.0

PORT_PATTERNS = ("/dev/cu.usbmodem*", "/dev/ttyACM*")


def find_port(timeout=WAIT_SECONDS):
    """Return the first USB serial port that shows up, or None on timeout."""
    deadline = time.monotonic() + timeout
    while True:
        for pattern in PORT_PATTERNS:
            matches = sorted(glob.glob(pattern))
            if matches:
                return matches[0]
        if time.monotonic() >= deadline:
            return None
        time.sleep(0.2)


def main():
    port = find_port()
    if port is None:
        print(
            "No USB serial port found: flash with 'make upload-firmware' and "
            "connect the Pico over USB.",
            file=sys.stderr,
        )
        return 1

    print(f"Reading {port} (press Ctrl-C to stop)", flush=True)

    stdin_fd = sys.stdin.fileno()
    saved = None
    if sys.stdin.isatty():
        saved = termios.tcgetattr(stdin_fd)
        raw = termios.tcgetattr(stdin_fd)
        raw[3] &= ~(termios.ISIG | termios.ICANON | termios.ECHO)
        raw[6][termios.VMIN] = 0
        raw[6][termios.VTIME] = 0
        termios.tcsetattr(stdin_fd, termios.TCSADRAIN, raw)

    def restore(*_):
        if saved is not None:
            termios.tcsetattr(stdin_fd, termios.TCSADRAIN, saved)

    def stop(*_):
        restore()
        sys.exit(0)

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)

    try:
        with serial.Serial(port, 115200, timeout=0) as ser:
            fd = ser.fileno()
            while True:
                ready, _, _ = select.select([fd, stdin_fd], [], [], 0.2)
                if fd in ready:
                    data = os.read(fd, 4096)
                    if not data:
                        break
                    sys.stdout.write(data.decode("utf-8", "replace"))
                    sys.stdout.flush()
                if stdin_fd in ready:
                    if os.read(stdin_fd, 1) in (b"\x03", b"\x04"):
                        break
    except KeyboardInterrupt:
        pass
    finally:
        restore()
        print("\n(stopped)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
