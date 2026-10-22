//! Procedurally drawn application icon (no image assets, no dependencies).
//!
//! The mark is an original design: a violet rounded square carrying a
//! white "memory trace" — a ring with a highlighted arc and a nucleus dot,
//! suggesting an engram lit up by recall.

/// Draw the 256×256 RGBA icon.
#[must_use]
pub fn icon_rgba(size: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; size * size * 4];
    let n = size as f32;
    let cx = n * 0.5;
    let cy = n * 0.47;
    let radius = n * 0.46; // rounded-square corner radius reference

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Background: rounded square with a vertical gradient.
            let half = n * 0.5 - 1.0;
            let qx = (px - cx).abs() - (half - radius);
            let qy = (py - cy).abs() - (half - radius);
            let outside = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
            let alpha = (-outside * (n / 24.0)).clamp(0.0, 1.0);
            if alpha <= 0.0 {
                continue;
            }

            let t = py / n;
            let (r, g, b) = lerp((138, 108, 255), (82, 56, 190), t);
            let (mut fr, mut fg, mut fb) = (r, g, b);
            let mut cover = 0.0f32;

            // Glyph distances from the mark's centre.
            let dx = px - cx;
            let dy = py - cy;
            let d = dx.hypot(dy);

            // Nucleus dot.
            if d < n * 0.055 {
                cover = 1.0;
            }
            // Ring.
            let ring = (d - n * 0.19).abs();
            if ring < n * 0.022 {
                cover = cover.max(1.0 - ring / (n * 0.022));
            }
            // Highlighted arc segment (upper right), thicker and brighter.
            let angle = dy.atan2(dx); // -π..π, 0 = +x
            if d > n * 0.24 && d < n * 0.30 && angle < -0.35 && angle > -1.25 {
                cover = cover.max(0.85);
            }

            if cover > 0.0 {
                fr = lerp_channel(fr, 255.0, cover);
                fg = lerp_channel(fg, 255.0, cover);
                fb = lerp_channel(fb, 255.0, cover);
            }

            let i = (y * size + x) * 4;
            rgba[i] = fr as u8;
            rgba[i + 1] = fg as u8;
            rgba[i + 2] = fb as u8;
            rgba[i + 3] = (alpha * 255.0) as u8;
        }
    }
    rgba
}

fn lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (f32, f32, f32) {
    let f = |x: u8, y: u8| f32::from(x) + (f32::from(y) - f32::from(x)) * t;
    (f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

fn lerp_channel(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

/// Build the egui icon data for the window.
#[must_use]
pub fn icon_data() -> egui::viewport::IconData {
    const S: usize = 256;
    egui::viewport::IconData {
        width: S as u32,
        height: S as u32,
        rgba: icon_rgba(S),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_has_opaque_center_and_transparent_corner() {
        let s = 64;
        let rgba = icon_rgba(s);
        let idx = |x: usize, y: usize| (y * s + x) * 4;
        // Center pixel: opaque background or glyph.
        assert_eq!(rgba[idx(32, 30) + 3], 255);
        // Far corner: fully transparent.
        assert_eq!(rgba[idx(0, 0) + 3], 0);
    }
}
