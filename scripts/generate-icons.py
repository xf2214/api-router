import os
import struct
from io import BytesIO
from PIL import Image, ImageDraw, ImageFont

ICONS_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(ICONS_DIR, exist_ok=True)

# Generate a simple source image: dark blue rounded square with "AR" text
SIZE = 1024
img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Draw rounded rectangle background
margin = 80
radius = 160
corner_box = [margin, margin, SIZE - margin, SIZE - margin]
draw.rounded_rectangle(corner_box, radius=radius, fill=(30, 111, 217, 255))

# Draw "AR" text
try:
    font = ImageFont.truetype("arial.ttf", 420)
except Exception:
    font = ImageFont.load_default()

text = "AR"
bbox = draw.textbbox((0, 0), text, font=font)
text_w = bbox[2] - bbox[0]
text_h = bbox[3] - bbox[1]
position = ((SIZE - text_w) // 2, (SIZE - text_h) // 2 - 40)
draw.text(position, text, fill=(255, 255, 255, 255), font=font)

# Save PNGs
png_sizes = [32, 128]
for s in png_sizes:
    resized = img.resize((s, s), Image.LANCZOS)
    resized.save(os.path.join(ICONS_DIR, f"{s}x{s}.png"))

# 128x128@2x
resized = img.resize((256, 256), Image.LANCZOS)
resized.save(os.path.join(ICONS_DIR, "128x128@2x.png"))

# Save multi-size ICO
ico_sizes = [16, 32, 48, 128, 256]
ico_imgs = [img.resize((s, s), Image.LANCZOS) for s in ico_sizes]
ico_imgs[0].save(
    os.path.join(ICONS_DIR, "icon.ico"),
    format="ICO",
    sizes=[(s, s) for s in ico_sizes],
)

# Save ICNS for macOS
icns_sizes = [
    (16, b"icp4"),
    (32, b"icp5"),
    (64, b"icp6"),
    (128, b"ic07"),
    (256, b"ic08"),
    (512, b"ic09"),
    (1024, b"ic10"),
]

records = []
for s, icon_type in icns_sizes:
    png_data = BytesIO()
    img.resize((s, s), Image.LANCZOS).save(png_data, format="PNG")
    data = png_data.getvalue()
    header = struct.pack(">4s I", icon_type, len(data) + 8)
    records.append(header + data)

body = b"".join(records)
file_size = 8 + len(body)
icns_data = struct.pack(">4s I", b"icns", file_size) + body

with open(os.path.join(ICONS_DIR, "icon.icns"), "wb") as f:
    f.write(icns_data)

print("Icons generated successfully.")
