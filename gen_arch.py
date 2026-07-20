from PIL import Image, ImageDraw, ImageFont

W, H = 1100, 760
img = Image.new("RGBA", (W, H), (15, 18, 28, 255))
d = ImageDraw.Draw(img)

def font(size, bold=True):
    for c in ["C:/Windows/Fonts/arialbd.ttf", "C:/Windows/Fonts/arial.ttf",
              "C:/Windows/Fonts/DejaVuSans-Bold.ttf"]:
        try:
            return ImageFont.truetype(c, size)
        except Exception:
            continue
    return ImageFont.load_default()

def box(x, y, w, h, fill, outline, label, lcolor=(235, 242, 250), lsize=26, lbold=True):
    d.rounded_rectangle([x, y, x + w, y + h], radius=14, fill=fill, outline=outline, width=3)
    f = font(lsize, lbold)
    tw = d.textlength(label, font=f)
    d.text((x + (w - tw) / 2, y + (h - lsize) / 2 - 2), label, font=f, fill=lcolor)

def arrow(x1, y1, x2, y2, color=(150, 170, 195), width=3, label=None, lsize=20):
    d.line([x1, y1, x2, y2], fill=color, width=width)
    # arrowhead
    import math
    ang = math.atan2(y2 - y1, x2 - x1)
    for da in (math.radians(150), math.radians(210)):
        ax = x2 + 14 * math.cos(ang + da)
        ay = y2 + 14 * math.sin(ang + da)
        d.line([x2, y2, ax, ay], fill=color, width=width)
    if label:
        f = font(lsize)
        tw = d.textlength(label, font=f)
        mx, my = (x1 + x2) / 2, (y1 + y2) / 2
        d.rectangle([mx - tw / 2 - 4, my - lsize / 2 - 2, mx + tw / 2 + 4, my + lsize / 2 + 2],
                    fill=(15, 18, 28, 255))
        d.text((mx - tw / 2, my - lsize / 2), label, font=f, fill=(170, 190, 210))

BOX = (30, 41, 59, 255)
OUT = (88, 166, 255, 255)
PORT = (88, 166, 255, 255)
MATCH = (99, 102, 241, 255)
PROTO = [
    ("gRPC listener", (124, 179, 255)),
    ("SSH listener", (126, 231, 135)),
    ("TLS listener", (255, 205, 110)),
    ("HTTP listener", (255, 138, 128)),
]

# Title
tf = font(34, True)
d.text((40, 24), "cmux-rs data flow", font=tf, fill=(220, 230, 245, 255))

# External clients (left)
box(40, 300, 170, 90, BOX, (120, 140, 170), "Clients", lsize=26)
d.text((70, 410), "TCP", font=font(20), fill=(150, 170, 195))

# TcpListener
box(280, 300, 190, 90, BOX, OUT, "TcpListener", lsize=24)

# Cmux serve core
box(530, 250, 220, 190, MATCH, (140, 150, 255), "", lsize=24)
d.text((560, 280), "Cmux.serve()", font=font(24, True), fill=(225, 230, 255))
d.text((548, 322), "accept()", font=font(20), fill=(190, 195, 230))
d.text((548, 352), "peek + timeout", font=font(20), fill=(190, 195, 230))
d.text((548, 382), "match in order", font=font(20), fill=(190, 195, 230))
d.text((548, 412), "BufferedStream", font=font(20), fill=(190, 195, 230))

# Shutdown watch (top)
box(530, 60, 220, 70, (40, 50, 70, 255), (255, 138, 128), "Shutdown watch", lsize=22)
arrow(640, 250, 640, 130, color=(255, 138, 128), label="signal", lsize=18)

# proto listeners (right column)
ly = 110
for i, (label, color) in enumerate(PROTO):
    y = ly + i * 130
    box(830, y, 230, 80, BOX, color, label, lsize=22)
    # arrow from core to listener
    arrow(750, 345, 830, y + 40, color=color, width=3)

# connecting arrows
arrow(210, 345, 280, 345, label="conn", lsize=18)
arrow(470, 345, 530, 345, label="stream", lsize=18)

# Peek detail note
nf = font(18)
d.text((560, 470), "first N bytes matched against each matcher;", font=nf, fill=(140, 160, 185))
d.text((560, 492), "first match wins; unmatched dropped; peeked", font=nf, fill=(140, 160, 185))
d.text((560, 514), "bytes replayed via BufferedStream.", font=nf, fill=(140, 160, 185))

img.save("assets/architecture.png")
print("architecture written")
