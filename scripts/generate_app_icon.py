import os
import math
from PIL import Image, ImageDraw, ImageFilter

WIDTH, HEIGHT = 512, 512
img = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

cx, cy = WIDTH / 2, HEIGHT / 2

# 1. Background Rounded Squircle with Subtle Radial Obsidian Gradient
radius = 232
for r in range(int(radius), 0, -1):
    ratio = r / radius
    # Dark radial gradient from #0d1322 center to #03060d outer edge
    red = int(13 * ratio + 3 * (1 - ratio))
    green = int(19 * ratio + 6 * (1 - ratio))
    blue = int(34 * ratio + 13 * (1 - ratio))
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(red, green, blue, 255))

# 2. Sleek Outer Ambient Rim Glow (Liquid Electric Cyan -> Violet)
rim_layer = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
rim_draw = ImageDraw.Draw(rim_layer)

for angle_deg in range(360):
    rad = math.radians(angle_deg)
    r_rim = 228
    px = cx + r_rim * math.cos(rad)
    py = cy + r_rim * math.sin(rad)
    
    # Smooth gradient around rim: Cyan (0 deg) -> Blue (120 deg) -> Purple (240 deg)
    t = angle_deg / 360.0
    cr = int(15 + 130 * (0.5 + 0.5 * math.sin(rad)))
    cg = int(180 + 75 * (0.5 + 0.5 * math.cos(rad)))
    cb = int(220 + 35 * (0.5 + 0.5 * math.sin(rad + 1.0)))
    
    rim_draw.ellipse([px - 5, py - 5, px + 5, py + 5], fill=(cr, cg, cb, 140))

# Blur the rim glow
rim_layer = rim_layer.filter(ImageFilter.GaussianBlur(4))
img = Image.alpha_composite(img, rim_layer)

# 3. Minimal Precision 3D Spatial Ring (Thin Orbital Track)
orbit_layer = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
orbit_draw = ImageDraw.Draw(orbit_layer)

orbit_rx = 160
orbit_ry = 65
for deg in range(0, 360, 2):
    rad = math.radians(deg)
    ox = cx + orbit_rx * math.cos(rad)
    oy = cy + 45 + orbit_ry * math.sin(rad)
    depth_alpha = int(70 + 80 * (math.sin(rad) * 0.5 + 0.5))
    orbit_draw.ellipse([ox - 1.5, oy - 1.5, ox + 1.5, oy + 1.5], fill=(0, 245, 212, depth_alpha))

# Glowing focal satellite node (representing 3D audio listener/spatial anchor)
node_rad = math.radians(40)
nx = cx + orbit_rx * math.cos(node_rad)
ny = cy + 45 + orbit_ry * math.sin(node_rad)
for gr, ga in [(16, 30), (10, 80), (6, 160), (3, 255)]:
    orbit_draw.ellipse([nx - gr, ny - gr, nx + gr, ny + gr], fill=(255, 255, 255, ga))
orbit_draw.ellipse([nx - 2, ny - 2, nx + 2, ny + 2], fill=(0, 245, 212, 255))

img = Image.alpha_composite(img, orbit_layer)

# 4. Minimal Architectural "M" Acoustic Monogram (5 sleek vertical acoustic pillars)
# High symmetry, pure lines, micro-rounded caps
bars_layer = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
bars_draw = ImageDraw.Draw(bars_layer)

pillars = [
    # (x_center_offset, height, width, color)
    (-88, 148, 14, (0, 245, 212, 255)),   # Cyan left outer
    (-44, 196, 16, (56, 189, 248, 255)),  # Sky blue left peak
    (0,   125, 14, (248, 250, 252, 255)),  # Crisp white center valley
    (44,  196, 16, (99, 102, 241, 255)),  # Indigo right peak
    (88,  148, 14, (168, 85, 247, 255)),  # Violet right outer
]

for bx, bh, bw, col in pillars:
    x0 = cx + bx - bw / 2
    x1 = cx + bx + bw / 2
    y0 = cy - bh / 2 - 12
    y1 = cy + bh / 2 - 12
    bars_draw.rounded_rectangle([x0, y0, x1, y1], radius=bw // 2, fill=col)

# Subtle ambient bloom around the monogram
bloom = bars_layer.filter(ImageFilter.GaussianBlur(12))
img = Image.alpha_composite(img, bloom)
img = Image.alpha_composite(img, bars_layer)

# 5. Delicate Glass Specular Sheen (Curved horizon reflex)
spec_layer = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
spec_draw = ImageDraw.Draw(spec_layer)
spec_draw.ellipse([cx - 150, cy - 220, cx + 150, cy - 70], fill=(255, 255, 255, 22))
spec_layer = spec_layer.filter(ImageFilter.GaussianBlur(16))
img = Image.alpha_composite(img, spec_layer)

# Ensure output directories exist
os.makedirs("dist", exist_ok=True)
os.makedirs("android/res/drawable", exist_ok=True)
for density in ["mdpi", "hdpi", "xhdpi", "xxhdpi", "xxxhdpi"]:
    os.makedirs(f"android/res/mipmap-{density}", exist_ok=True)

# Save Master 512x512 Icon
img.save("dist/mx5ic_icon_512.png", "PNG")

# Generate sizes for Linux Desktop and Android
sizes = {
    "dist/mx5ic_icon.png": (256, 256),
    "android/res/drawable/ic_launcher.png": (192, 192),
    "android/res/mipmap-mdpi/ic_launcher.png": (48, 48),
    "android/res/mipmap-hdpi/ic_launcher.png": (72, 72),
    "android/res/mipmap-xhdpi/ic_launcher.png": (96, 96),
    "android/res/mipmap-xxhdpi/ic_launcher.png": (144, 144),
    "android/res/mipmap-xxxhdpi/ic_launcher.png": (192, 192),
}

for path, size in sizes.items():
    resized = img.resize(size, Image.Resampling.LANCZOS)
    resized.save(path, "PNG")

print("[+] Minimal luxury obsidian & electric monogram icon generated successfully across all formats.")
