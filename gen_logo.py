from PIL import Image, ImageDraw, ImageFont
import math

W, H = 1024, 1024
img = Image.new("RGBA", (W, H), (15, 18, 28, 255))
d = ImageDraw.Draw(img)

BG = (15, 18, 28)
PORT_FILL = (88, 166, 255)
PORT_RING = (173, 216, 230)
PROTO = [
    ("gRPC", (124, 179, 255)),
    ("SSH", (126, 231, 135)),
    ("TLS", (255, 205, 110)),
    ("HTTP", (255, 138, 128)),
]


def font(size, bold=True):
    candidates = [
        "C:/Windows/Fonts/seguiemj.ttf",
        "C:/Windows/Fonts/arialbd.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/DejaVuSans-Bold.ttf",
    ]
    for c in candidates:
        try:
            return ImageFont.truetype(c, size)
        except Exception:
            continue
    return ImageFont.load_default()


cx, cy = W // 2, H // 2
port_r = 120

# glow behind port
for r, a in [(port_r + 60, 40), (port_r + 30, 70)]:
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(88, 166, 255, a))

# central port node
d.ellipse([cx - port_r, cy - port_r, cx + port_r, cy + port_r],
          fill=PORT_FILL, outline=PORT_RING, width=8)

# port label
pf = font(64)
pw = d.textlength("cmux", font=pf)
d.text((cx - pw / 2, cy - 38), "cmux", font=pf, fill=(10, 14, 22, 255))

# branches
n = len(PROTO)
for i, (label, color) in enumerate(PROTO):
    ang = -90 + (i - (n - 1) / 2) * 46
    rad = math.radians(ang)
    # line from port edge outward
    x1 = cx + math.cos(rad) * (port_r - 6)
    y1 = cy + math.sin(rad) * (port_r - 6)
    length = 250
    x2 = cx + math.cos(rad) * (port_r + length)
    y2 = cy + math.sin(rad) * (port_r + length)
    # protocol node
    node_r = 64
    nx, ny = x2, y2
    d.line([x1, y1, nx, ny], fill=color, width=10)
    d.ellipse([nx - node_r, ny - node_r, nx + node_r, ny + node_r],
              fill=color, outline=(255, 255, 255, 120), width=3)
    lf = font(34)
    tw = d.textlength(label, font=lf)
    d.text((nx - tw / 2, ny - 22), label, font=lf, fill=(15, 18, 28, 255))

# title
tf = font(54, bold=True)
title = "cmux-rs"
tw = d.textlength(title, font=tf)
d.text((cx - tw / 2, 60), title, font=tf, fill=(220, 230, 245, 255))
sf = font(30)
sub = "Connection Multiplexer"
sw = d.textlength(sub, font=sf)
d.text((cx - sw / 2, 128), sub, font=sf, fill=(140, 160, 185, 255))

img.save("assets/logo.png")
print("logo written")
