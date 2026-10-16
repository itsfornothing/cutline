#!/usr/bin/env python3
"""
Generates Scene 00: Hook take (8.0s, 60fps, 1080p)
Addresses the developer pain points:
"You're paying for renders that fail, previews that lie, and three jobs for three aspect ratios."
"""

import os
import subprocess
from PIL import Image, ImageDraw, ImageFont

TAKES_DIR = os.path.abspath("marketing/takes")
FONTS_DIR = os.path.abspath("vendor/filmcraft/assets/fonts")
os.makedirs(TAKES_DIR, exist_ok=True)

FONT_INTER_REG = os.path.join(FONTS_DIR, "Inter-Regular.ttf")
FONT_INTER_BOLD = os.path.join(FONTS_DIR, "Inter-Bold.ttf")
FONT_MONO = os.path.join(FONTS_DIR, "JetBrainsMono-Regular.ttf")

def get_font(path, size):
    try:
        return ImageFont.truetype(path, size)
    except Exception:
        return ImageFont.load_default()

def gen_hook_frame(f, total):
    t = f / total
    img = Image.new("RGB", (1920, 1080), "#09090B")
    draw = ImageDraw.Draw(img)

    # Subtle grid lines
    for x in range(0, 1920, 80):
        draw.line([(x, 0), (x, 1080)], fill="#141417", width=1)
    for y in range(0, 1080, 80):
        draw.line([(0, y), (1920, y)], fill="#141417", width=1)

    # Header indicator
    draw.text((80, 70), "HEADLESS VIDEO RENDERING IN PRODUCTION", font=get_font(FONT_MONO, 14), fill="#71717A")
    draw.line([80, 100, 1840, 100], fill="#27272A", width=1)

    # Pain Point 1: Renders that fail (0.0s - 2.6s)
    act1 = t >= 0.05
    p1_bg = "#1C1917" if act1 else "#141416"
    p1_border = "#EF4444" if act1 else "#27272A"
    draw.rounded_rectangle([120, 180, 620, 780], radius=12, fill=p1_bg, outline=p1_border, width=2 if act1 else 1)
    draw.text((160, 220), "01 / SILENT FAILURES", font=get_font(FONT_MONO, 13), fill="#EF4444" if act1 else "#52525B")
    draw.text((160, 260), "Renders that fail.", font=get_font(FONT_INTER_BOLD, 28), fill="#FFFFFF" if act1 else "#52525B")
    
    if act1:
        draw.rounded_rectangle([160, 340, 580, 480], radius=8, fill="#0A0A0A", outline="#7F1D1D", width=1)
        draw.text((180, 360), "$ cutline-legacy worker:run", font=get_font(FONT_MONO, 12), fill="#A1A1AA")
        draw.text((180, 395), "FATAL: Worker OOM killed (SIGKILL)", font=get_font(FONT_MONO, 12), fill="#F87171")
        draw.text((180, 430), "Elapsed: 4m 12s | Billed: $1.85", font=get_font(FONT_MONO, 12), fill="#FCA5A5")

        draw.text((160, 530), "• Non-deterministic memory spikes", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")
        draw.text((160, 575), "• Billed for uncompleted runs", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")
        draw.text((160, 620), "• Silent error swallowing in CI", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")

    # Pain Point 2: Previews that lie (2.6s - 5.2s)
    act2 = t >= 0.32
    p2_bg = "#1C1917" if act2 else "#141416"
    p2_border = "#F59E0B" if act2 else "#27272A"
    draw.rounded_rectangle([710, 180, 1210, 780], radius=12, fill=p2_bg, outline=p2_border, width=2 if act2 else 1)
    draw.text((750, 220), "02 / PREVIEW PARITY", font=get_font(FONT_MONO, 13), fill="#F59E0B" if act2 else "#52525B")
    draw.text((750, 260), "Previews that lie.", font=get_font(FONT_INTER_BOLD, 28), fill="#FFFFFF" if act2 else "#52525B")

    if act2:
        draw.rounded_rectangle([750, 340, 1170, 480], radius=8, fill="#0A0A0A", outline="#78350F", width=1)
        draw.text((770, 360), "DOM Preview vs Native Render", font=get_font(FONT_MONO, 12), fill="#A1A1AA")
        draw.text((770, 395), "WARN: Layout shift detected", font=get_font(FONT_MONO, 12), fill="#FBBF24")
        draw.text((770, 430), "Canvas: 60fps | Encoder: 14fps", font=get_font(FONT_MONO, 12), fill="#FDE68A")

        draw.text((750, 530), "• Browser font metrics mismatch", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")
        draw.text((750, 575), "• WebGL shader differences", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")
        draw.text((750, 620), "• No frame hash regression gate", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")

    # Pain Point 3: Three jobs for three aspect ratios (5.2s - 8.0s)
    act3 = t >= 0.62
    p3_bg = "#1C1917" if act3 else "#141416"
    p3_border = "#38BDF8" if act3 else "#27272A"
    draw.rounded_rectangle([1300, 180, 1800, 780], radius=12, fill=p3_bg, outline=p3_border, width=2 if act3 else 1)
    draw.text((1340, 220), "03 / MULTI-FORMAT TAX", font=get_font(FONT_MONO, 13), fill="#38BDF8" if act3 else "#52525B")
    draw.text((1340, 260), "Three jobs for three ratios.", font=get_font(FONT_INTER_BOLD, 26), fill="#FFFFFF" if act3 else "#52525B")

    if act3:
        draw.rounded_rectangle([1340, 340, 1760, 480], radius=8, fill="#0A0A0A", outline="#075985", width=1)
        draw.text((1360, 360), "Job #1 (9:16 Story)  -> $0.24", font=get_font(FONT_MONO, 12), fill="#E0F2FE")
        draw.text((1360, 395), "Job #2 (1:1 Square)  -> $0.24", font=get_font(FONT_MONO, 12), fill="#E0F2FE")
        draw.text((1360, 430), "Job #3 (16:9 Landscape)-> $0.24", font=get_font(FONT_MONO, 12), fill="#E0F2FE")

        draw.text((1340, 530), "• Re-rendering frames from scratch", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")
        draw.text((1340, 575), "• Triple billing on cloud compute", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")
        draw.text((1340, 620), "• Out-of-sync multi-platform assets", font=get_font(FONT_INTER_REG, 15), fill="#D4D4D8")

    # Bottom banner
    draw.rectangle([0, 960, 1920, 1080], fill="#111113", outline="#27272A", width=1)
    draw.text((120, 1005), "There is a better architecture.", font=get_font(FONT_INTER_BOLD, 22), fill="#F59E0B")
    draw.text((540, 1008), "Deterministic Headless Video Rendering Engine", font=get_font(FONT_INTER_REG, 17), fill="#A1A1AA")
    return img

def main():
    out_file = os.path.join(TAKES_DIR, "s00_hook.mp4")
    num_frames = 8 * 60  # 8.0s at 60 fps = 480 frames
    print("Generating s00_hook.mp4...")
    cmd = [
        "ffmpeg", "-y",
        "-f", "rawvideo",
        "-vcodec", "rawvideo",
        "-s", "1920x1080",
        "-pix_fmt", "rgb24",
        "-r", "60",
        "-i", "-",
        "-c:v", "libx264",
        "-preset", "veryfast",
        "-crf", "18",
        "-pix_fmt", "yuv420p",
        out_file
    ]
    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for i in range(num_frames):
        img = gen_hook_frame(i, num_frames)
        proc.stdin.write(img.tobytes())
    proc.stdin.close()
    proc.wait()
    print("✓ Rendered s00_hook.mp4 successfully.")

if __name__ == "__main__":
    main()
