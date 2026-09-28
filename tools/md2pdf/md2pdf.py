#!/usr/bin/env python3
"""Convert a Markdown file with Mermaid diagrams into a PDF.

Mermaid code blocks are rendered to images through the mermaid.ink service and
embedded in the document. The Markdown is then turned into HTML and the HTML
into a PDF.
"""

from __future__ import annotations

import argparse
import base64
import re
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

import markdown
from xhtml2pdf import pisa

MERMAID_BLOCK = re.compile(r"```mermaid\s*\n(.*?)```", re.DOTALL)
IMAGE_MARKDOWN = re.compile(r"!\[([^\]]*)\]\(([^)]+)\)")
IMAGE_HTML = re.compile(r'(<img\b[^>]*?\bsrc=")([^"]+)(")')
MERMAID_ENDPOINT = "https://mermaid.ink/img/"
IMAGE_WIDTH = 460
REQUEST_TIMEOUT_S = 90
MAX_ATTEMPTS = 3

CSS = """
@page { size: A4; margin: 2cm; }
body { font-family: Helvetica, sans-serif; font-size: 11pt; line-height: 1.45; }
h1 { font-size: 20pt; }
h2 { font-size: 15pt; margin-top: 16pt; }
h3 { font-size: 12pt; }
code { font-family: Courier, monospace; font-size: 9pt; }
pre { background-color: #f2f2f2; padding: 6pt; }
table { border-collapse: collapse; }
th, td { border: 1px solid #999999; padding: 4pt; }
img { border: 1px solid #cccccc; }
"""


def render_mermaid(code: str, destination: Path) -> None:
    """Render a Mermaid diagram to a PNG file via mermaid.ink."""
    encoded = base64.urlsafe_b64encode(code.encode("utf-8")).decode("ascii")
    query = urllib.parse.urlencode({"type": "png", "bgColor": "FFFFFF"})
    url = f"{MERMAID_ENDPOINT}{encoded}?{query}"
    request = urllib.request.Request(url, headers={"User-Agent": "md2pdf/1.0"})
    last_error = None
    for attempt in range(1, MAX_ATTEMPTS + 1):
        try:
            with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_S) as response:
                data = response.read()
            if not data.startswith(b"\x89PNG\r\n\x1a\n"):
                raise SystemExit(f"mermaid.ink did not return a PNG image for:\n{code}")
            destination.write_bytes(data)
            return
        except (urllib.error.URLError, OSError) as error:
            last_error = error
            time.sleep(2 * attempt)
    raise SystemExit(
        f"failed to render Mermaid diagram after {MAX_ATTEMPTS} attempts: {last_error}"
    )


def substitute_mermaid(text: str, workdir: Path) -> str:
    """Replace every Mermaid code block with an <img> tag."""
    counter = 0

    def replace(match: re.Match) -> str:
        nonlocal counter
        counter += 1
        image = workdir / f"mermaid-{counter}.png"
        render_mermaid(match.group(1).strip(), image)
        return f'<img src="{image}" width="{IMAGE_WIDTH}" />'

    return MERMAID_BLOCK.sub(replace, text)


def resolve_images(text: str, base: Path) -> str:
    """Rewrite relative image paths to absolute paths next to the source file."""

    def absolute(path: str) -> str:
        if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*://", path) or path.startswith("/"):
            return path
        return str((base / path).resolve())

    text = IMAGE_MARKDOWN.sub(
        lambda m: f"![{m.group(1)}]({absolute(m.group(2))})", text
    )
    return IMAGE_HTML.sub(
        lambda m: f"{m.group(1)}{absolute(m.group(2))}{m.group(3)}", text
    )


def convert(source: Path, destination: Path) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workdir = Path(tmp)
        text = substitute_mermaid(source.read_text(encoding="utf-8"), workdir)
        text = resolve_images(text, source.parent.resolve())
        body = markdown.markdown(
            text,
            extensions=["fenced_code", "tables", "toc", "sane_lists"],
        )
        html = f"<html><head><style>{CSS}</style></head><body>{body}</body></html>"
        with destination.open("wb") as pdf:
            result = pisa.CreatePDF(html, dest=pdf)
        if result.err:
            raise SystemExit("failed to render PDF")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Convert a Markdown file with Mermaid diagrams into a PDF."
    )
    parser.add_argument("source", type=Path, help="input Markdown file")
    parser.add_argument("destination", type=Path, help="output PDF file")
    args = parser.parse_args()
    args.destination.parent.mkdir(parents=True, exist_ok=True)
    convert(args.source, args.destination)
    print(f"wrote {args.destination}")


if __name__ == "__main__":
    main()
