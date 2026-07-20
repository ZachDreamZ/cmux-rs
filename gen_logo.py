from PIL import Image, ImageDraw, ImageFont
import math

W, H = 1024, 1024
BG = (15, 18, 28, 255)
img = Image.new("RGBA", (W, H), BG)
d = ImageDraw.Draw(img)

PORT_FILL = (88, 166, 255)
PORT_RING = (173, 216, 230)
PROTO = [
    ("gRPC", (124, 179, 255)),
    ("SSH", (126, 231, 135)),
    ("TLS", (255, 205, 110)),
    ("HTTP", (255, 138, 128)),
]


def font(size, bold=True):
    for c in ["C:/Windows/Fonts/arialbd.ttf", "C:/Windows/Fonts/arial.ttf",
              "C:/Windows/Fonts/DejaVuSans-Bold.ttf"]:
        try:
            return ImageFont.truetype(c, size)
        except Exception:
            continue
    return ImageFont.load_default()


def text(d, s, x, y, f, fill):
    w = d.textlength(s, font=f)
    d.text((x - w / 2, y), s, font=f, fill=fill)


cx, cy = W // 2, H // 2 + 20
port_r = 118

# soft glow behind the port node
for r, a in [(port_r + 70, 28), (port_r + 38, 55)]:
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(88, 166, 255, a))

# central port node
d.ellipse([cx - port_r, cy - port_r, cx + port_r, cy + port_r],
          fill=PORT_FILL, outline=PORT_RING, width=8)
text(d, "cmux", cx, cy - 40, font(62), (10, 14, 22, 255))

# fan of protocol streams
n = len(PROTO)
for i, (label, color) in enumerate(PROTO):
    ang = -90 + (i - (n - 1) / 2) * 44
    rad = math.radians(ang)
    x1 = cx + math.cos(rad) * (port_r - 4)
    y1 = cy + math.sin(rad) * (port_r - 4)
    length = 250
    nx = cx + math.cos(rad) * (port_r + length)
    ny = cy + math.sin(rad) * (port_r + length)
    node_r = 66
    # connecting line
    d.line([x1, y1, nx, ny], fill=color, width=11)
    # protocol node
    d.ellipse([nx - node_r, ny - node_r, nx + node_r, ny + node_r],
              fill=color, outline=(255, 255, 255, 130), width=3)
    text(d, label, nx, ny - 21, font(32), (15, 18, 28, 255))
    # small dot where the line meets the port for polish
    d.ellipse([x1 - 5, y1 - 5, x1 + 5, y1 + 5], fill=color)

# title + subtitle
text(d, "cmux-rs", cx, 70, font(58, True), (220, 230, 245, 255))
text(d, "Connection Multiplexer", cx, 142, font(30), (140, 160, 185, 255))

img.save("assets/logo.png")
print("logo written")
