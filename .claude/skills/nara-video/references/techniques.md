# Visual Techniques

Read this when designing scenes.
The primary source, `references/design.md` of the `fframes-video` skill, covers general motion, typography, and scene structure; this file adds what production taught on top of it.

Everything is a pure function of `frame.seconds()`. No state is kept between frames, so any frame can be rendered alone.

## UI Walkthrough from a Screenshot

### Capture

- Take the screenshot in a desktop browser. A headless browser (Obscura) ignored CSS zoom and laid a form out differently from Chrome. Playwright with `deviceScaleFactor: 2` is an untested alternative that needs a Chromium download.
- A page that reveals sections after input needs one screenshot per state. A single screenshot shows every section from the start, which the live page never does at that step. Capturing two states and cross-fading at the step is untested.

### Camera

The screenshot sits in a clipped viewport inside a browser-style card.
A camera `(cx, cy, s)` names the page point at the viewport center and the zoom.

```rust
let transform = format!(
    "translate({:.3} {:.3}) scale({:.4})",
    VIEW_X + VIEW_W / 2. - s * cx,
    VIEW_Y + VIEW_H / 2. - s * cy,
    s
);
```

- Clamp the camera so the viewport never leaves the image: `cx` in `[VIEW_W / (2 s), IMAGE_W - VIEW_W / (2 s)]`, and the same for `cy` with heights.
- Each beat has its own camera, and the move from the previous one runs for 0.8 s from the beat boundary with smoothstep easing.
- Keep zoom at or below 2x for an 840 px wide screenshot; beyond that the text blurs.
- Pick each zoom so no text is cut at the viewport edge, and check it in a `frame` PNG.

### Overlays in Page Coordinates

Overlays live inside the camera group and use the screenshot's pixel coordinates, so they move and zoom with the page.

- Measure field rectangles on the screenshot, then verify them by drawing boxes and looking at the result: `ffmpeg -i shot.png -vf "drawbox=x=21:y=274:w=379:h=23:color=red:t=1" boxes.png`.
- Typing: `chars = (t - start) * per_second`, cut on character boundaries, with a caret placed by `frame.text_width` that stays solid while typing and blinks afterwards.
- Cover rectangles hide what the screenshot already shows, such as a preselected dropdown value or a placeholder. Sample the background color with `ffmpeg -i shot.png -vf "crop=1:1:X:Y,format=rgb24" -f rawvideo - | od -An -tx1`.
- Draw a dropdown list in vector and derive its highlighted row from the pointer's y position, so the highlight follows the cursor without separate keyframes.
- A focus ring uses `stroke-width = 4 / s` so it keeps the same on-screen width at any zoom.
- A button press is a short tinted flash over the button. The real action never runs on a live system, because submitting a form there creates real records and sends real email.

### Cursor in Screen Coordinates

The pointer path is keyed in page coordinates, then mapped to screen coordinates with the current camera, so the cursor keeps a constant size while the page zooms.

```rust
fn to_screen(cam: Camera, p: Point) -> Point {
    (
        VIEW_X + VIEW_W / 2. + cam.2 * (p.0 - cam.0),
        VIEW_Y + VIEW_H / 2. + cam.2 * (p.1 - cam.1),
    )
}
```

Each pointer key moves from the previous target over 0.45 s, and each click draws an expanding ring for 0.5 s.

## Portrait 1080x1920 Layout

### Safe Zone

Text and key elements stay where neither TikTok nor Reels draws its own UI.
The nara bound is the intersection of both platforms' ad safe zones; organic posts carry the same caption, account name, and button rail, so the ad zone is the stricter bound.

| Edge        | TikTok                    | Reels                         | nara bound          |
| ----------- | ------------------------- | ----------------------------- | ------------------- |
| Top         | 240 px                    | 269 px (14 %)                 | y ≥ 270             |
| Bottom      | 660 px                    | 672 px (35 %)                 | y ≤ 1248            |
| Left        | 120 px                    | 65 px (6 %)                   | x ≥ 120             |
| Right       | 120 px                    | 65 px (6 %)                   | x ≤ 960             |
| Button rail | x 780-960 from y 840 down | right 227 px from y 1152 down | x ≤ 780 below y 840 |

- TikTok values come from the official In-Feed Standard (LTR) template on the [TikTok Ads video specifications page](https://ads.tiktok.com/help/article/video-ads-specifications), measured at its 720x1280 scale and multiplied by 1.5.
- Reels values are secondary. Meta's own Reels safe zone checker (PPT, PSD, Keynote) on the [Reels ads page](https://www.facebook.com/business/ads/facebook-instagram-reels-ads) does not download outside a browser, and the 14/35/6 % figures come from third-party guides that agree with each other. Confirm them against the checker when it is at hand.

Check a frame by shading everything outside the bound, then look at the result:

```bash
ffmpeg -i frames/<t>.png -vf "drawbox=x=0:y=0:w=1080:h=270:color=red@0.35:t=fill,drawbox=x=0:y=1248:w=1080:h=672:color=red@0.35:t=fill,drawbox=x=0:y=270:w=120:h=978:color=red@0.35:t=fill,drawbox=x=960:y=270:w=120:h=978:color=red@0.35:t=fill,drawbox=x=780:y=840:w=180:h=408:color=red@0.35:t=fill" safe.png
```

### Grid

- The first video placed its eyebrow at y 284, step title at y 356, progress bars at y 392, browser card from y 464 to 1240, and subtitles from y 1340, with text starting at x 70. A shaded frame shows the subtitles, the left edge of the text, and the top of the eyebrow outside the bound, so reusing that grid needs the subtitles above y 1248 and the text right of x 120.
- Text is never below 28 px, except the screenshot's own UI text.
- The first frame already shows the title, the main visual, and the first subtitle, with no fade from black.

## Text

`text_break_lines` rendered wrapped text at regular weight although 600 and 700 were requested.
Write line breaks by hand as `\n` and use plain `<text font-weight="700">` for bold text; the root cause was not investigated.

## Brand Colors

Take the palette from the product's own CSS theme, so the video matches what the viewer sees on the site.
DaisyUI themes define colors in OKLCH; this converts a value to hex:

```python
import math

def oklch_to_hex(L, C, H):
    a, b = C * math.cos(math.radians(H)), C * math.sin(math.radians(H))
    l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s = (L - 0.0894841775 * a - 1.2914855480 * b) ** 3
    rgb = (
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    )
    gamma = lambda x: 12.92 * x if x <= 0.0031308 else 1.055 * x ** (1 / 2.4) - 0.055
    return "#" + "".join("%02x" % round(max(0, min(1, gamma(c))) * 255) for c in rgb)

print(oklch_to_hex(0.508, 0.118, 165.612))
```

| DaisyUI token  | Use in the video                            |
| -------------- | ------------------------------------------- |
| `primary`      | Eyebrow, progress bars, focus ring, buttons |
| `accent`       | Ring glow, underline emphasis               |
| `neutral`      | Titles                                      |
| `base-content` | Subtitles                                   |
| `base-200`     | Background, with the site's own pattern     |
| `base-300`     | Soft fills                                  |
