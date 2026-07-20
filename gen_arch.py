from PIL import Image, ImageDraw, ImageFont
import math

W, H = 1120, 720
BG = (15, 18, 28, 255)
img = Image.new("RGBA", (W, H), BG)
d = ImageDraw.Draw(img)


def font(size, bold=True):
    for c in ["C:/Windows/Fonts/arialbd.ttf", "C:/Windows/Fonts/arial.ttf",
              "C:/Windows/Fonts/DejaVuSans-Bold.ttf"]:
        try:
            return ImageFont.truetype(c, size)
        except Exception:
            continue
    return ImageFont.load_default()


def text(s, x, y, f, fill, anchor="mm"):
    w = d.textlength(s, font=f)
    dx = {"mm": -w / 2, "lm": 0, "rm": -w}[anchor]
    d.text((x + dx, y - f.size / 2), s, font=f, fill=fill)


def box(x, y, w, h, fill, outline, label, lcolor=(235, 242, 250), lsize=24):
    d.rounded_rectangle([x, y, x + w, y + h], radius=14, fill=fill,
                        outline=outline, width=3)
    text(label, x + w / 2, y + h / 2, font(lsize, True), lcolor)


def arrow(x1, y1, x2, y2, color=(150, 170, 195), width=3, label=None, lsize=18):
    d.line([x1, y1, x2, y2], fill=color, width=width)
    ang = math.atan2(y2 - y1, x2 - x1)
    for da in (math.radians(155), math.radians(205)):
        ax = x2 + 15 * math.cos(ang + da)
        ay = y2 + 15 * math.sin(ang + da)
        d.line([x2, y2, ax, ay], fill=color, width=width)
    if label:
        f = font(lsize)
        w = d.textlength(label, font=f)
        mx, my = (x1 + x2) / 2, (y1 + y2) / 2
        d.rectangle([mx - w / 2 - 5, my - lsize / 2 - 3, mx + w / 2 + 5, my + lsize / 2 + 3],
                    fill=(15, 18, 28, 255))
        text(label, mx, my, f, (170, 190, 210))


BOX = (30, 41, 59, 255)
OUT = (88, 166, 255, 255)
MATCH = (99, 102, 241, 255)
PROTO = [
    ("gRPC listener", (124, 179, 255)),
    ("SSH listener", (126, 231, 135)),
    ("TLS listener", (255, 205, 110)),
    ("HTTP listener", (255, 138, 128)),
]

# Title
text("cmux-rs data flow", 40, 36, font(34, True), (220, 230, 245), anchor="lm")

# Column x positions
CLIENT_X, LISTEN_X, MUX_X = 50, 300, 560
BW_CLIENT, BW_LISTEN, BW_MUX = 170, 200, 230

# Clients
cy = 360
box(CLIENT_X, cy - 45, BW_CLIENT, 90, BOX, (120, 140, 170), "Clients")
text("TCP", CLIENT_X + BW_CLIENT / 2, cy + 75, font(20), (150, 170, 195))

# TcpListener
box(LISTEN_X, cy - 45, BW_LISTEN, 90, BOX, OUT, "TcpListener")

# Cmux core
mux_y = 300
box(MUX_X, mux_y, BW_MUX, 220, MATCH, (140, 150, 255), "")
text("Cmux.serve()", MUX_X + BW_MUX / 2, mux_y + 30, font(24, True), (225, 230, 255))
for i, step in enumerate(["accept()", "peek + timeout", "match in order", "BufferedStream"]):
    text(step, MUX_X + BW_MUX / 2, mux_y + 78 + i * 36, font(20), (200, 205, 235))

# Shutdown watch (top)
box(MUX_X, 90, BW_MUX, 70, (40, 50, 70, 255), (255, 138, 128), "Shutdown watch")
arrow(MUX_X + BW_MUX / 2, mux_y, MUX_X + BW_MUX / 2, 160,
      color=(255, 138, 128), label="signal", lsize=18)

# Proto listeners
PROT_X = 870
BW_PROT = 220
gap = 130
for i, (label, color) in enumerate(PROTO):
    y = 120 + i * gap
    box(PROT_X, y, BW_PROT, 80, BOX, color, label)
    # arrow from mux right edge to listener left edge, centered on listener
    arrow(MUX_X + BW_MUX, mux_y + 110, PROT_X, y + 40, color=color, width=3)

# connecting arrows
arrow(CLIENT_X + BW_CLIENT, cy, LISTEN_X, cy, label="conn", lsize=18)
arrow(LISTEN_X + BW_LISTEN, cy, MUX_X, mux_y + 110, label="stream", lsize=18)

# Peek detail note
nf = font(18)
note = [
    "First N bytes matched against each matcher in order;",
    "first match wins, unmatched connections are dropped,",
    "and the peeked bytes are replayed via BufferedStream.",
]
for i, line in enumerate(note):
    text(line, 50, 600 + i * 24, nf, (140, 160, 185), anchor="lm")

img.save("assets/architecture.png")
print("architecture written")
