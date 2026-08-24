use gpui::{Hsla, hsla};

pub const INK_FILL_SCALE: f32 = 1.0;
pub const INK_HAIRLINE_SCALE: f32 = 1.35;
pub const SCRIM_ALPHA_DARK: f32 = 0.60;
pub const WASH_HOVER: f32 = 0.08;
pub const WASH_SELECTED: f32 = 0.11;
pub const PRESS_ACTIVE: f32 = 0.10;
pub const INK_ZEBRA: f32 = 0.03;

pub fn grey(value: u8) -> Hsla {
    hsla(0.0, 0.0, value as f32 / 255.0, 1.0)
}

/// Achromatic OKLCH lightness → Hsla (chroma 0).
pub fn neutral(lightness: f32) -> Hsla {
    let [v, _, _] = oklch_to_srgb(lightness, 0.0, 0.0);
    hsla(0.0, 0.0, v, 1.0)
}

pub fn oklch(l: f32, c: f32, h_deg: f32) -> Hsla {
    let [r, g, b] = oklch_to_srgb(l, c, h_deg);
    let (h, s, lightness) = rgb_to_hsl(r, g, b);
    hsla(h, s, lightness, 1.0)
}

pub fn oklch_to_srgb(l: f32, c: f32, h_deg: f32) -> [f32; 3] {
    let h = h_deg.to_radians();
    let a = c * h.cos();
    let b = c * h.sin();
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_93 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;
    [gamma_encode(r), gamma_encode(g), gamma_encode(b)]
}

fn gamma_encode(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    if s <= f32::EPSILON {
        return [l, l, l];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hue = |mut t: f32| {
        t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [hue(h + 1.0 / 3.0), hue(h), hue(h - 1.0 / 3.0)]
}

pub fn relative_luminance(color: Hsla) -> f32 {
    let lin = |c: f32| {
        if c <= 0.040_45 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let [r, g, b] = hsl_to_rgb(color.h, color.s, color.l);
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

pub fn contrast_ratio(a: Hsla, b: Hsla) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

pub fn flatten(fg: Hsla, bg: Hsla) -> Hsla {
    let a = fg.a.clamp(0.0, 1.0);
    let [fr, fg_, fb] = hsl_to_rgb(fg.h, fg.s, fg.l);
    let [br, bg_, bb] = hsl_to_rgb(bg.h, bg.s, bg.l);
    let (h, s, l) = {
        let r = fr * a + br * (1.0 - a);
        let g = fg_ * a + bg_ * (1.0 - a);
        let b = fb * a + bb * (1.0 - a);
        rgb_to_hsl(r, g, b)
    };
    hsla(h, s, l, 1.0)
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;
    if delta < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let h = if (max - r).abs() < f32::EPSILON {
        ((g - b) / delta).rem_euclid(6.0)
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    } / 6.0;
    (h, s, l)
}

pub fn ink_for(dark: bool, alpha: f32) -> Hsla {
    if dark {
        hsla(0.0, 0.0, 1.0, alpha)
    } else {
        hsla(0.0, 0.0, 0.0, alpha * INK_FILL_SCALE)
    }
}

pub fn wash_for(dark: bool, alpha: f32) -> Hsla {
    if dark {
        hsla(0.0, 0.0, 0.92, alpha)
    } else {
        hsla(0.0, 0.0, 0.10, alpha * INK_FILL_SCALE)
    }
}

/// A transient pressed-state layer.
///
/// Unlike [`wash_for`], press always recedes from the local plane. This keeps
/// active controls darker than hover in both modes without borrowing a themed
/// component plate such as `input_bg`.
pub fn press_for(alpha: f32) -> Hsla {
    hsla(0.0, 0.0, 0.0, alpha)
}

pub fn hairline_for(dark: bool, alpha: f32) -> Hsla {
    if dark {
        hsla(0.0, 0.0, 1.0, alpha)
    } else {
        hsla(0.0, 0.0, 0.0, (alpha * INK_HAIRLINE_SCALE).min(0.5))
    }
}

pub fn scrim_for(dark: bool, alpha_dark: f32) -> Hsla {
    if dark {
        // Notion's installed dark popup backdrop is rgba(15, 15, 15, 0.6).
        // Keeping the 60% coverage while lifting the pigment off pure black
        // preserves context behind a modal instead of collapsing it to near-black.
        hsla(0.0, 0.0, 15.0 / 255.0, alpha_dark)
    } else {
        hsla(0.0, 0.0, 0.0, 0.32 * (alpha_dark / SCRIM_ALPHA_DARK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_ratio_white_black_is_21() {
        let white = grey(0xff);
        let black = grey(0x00);
        assert!((contrast_ratio(white, black) - 21.0).abs() < 0.01);
    }

    #[test]
    fn hairline_strengthens_in_light() {
        let d = hairline_for(true, 0.10);
        let l = hairline_for(false, 0.10);
        assert_eq!(d.l, 1.0);
        assert_eq!(l.l, 0.0);
        assert!(l.a > d.a);
        assert!(hairline_for(false, 0.60).a <= 0.5);
    }

    #[test]
    fn ink_keeps_alpha_in_both_modes() {
        assert_eq!(ink_for(true, 0.03).a, 0.03);
        assert_eq!(ink_for(false, 0.03).a, 0.03);
        assert_eq!(ink_for(true, 0.03).l, 1.0);
        assert_eq!(ink_for(false, 0.03).l, 0.0);
    }

    #[test]
    fn press_is_a_mode_independent_recessed_layer() {
        let press = press_for(PRESS_ACTIVE);
        assert_eq!(press.l, 0.0);
        assert_eq!(press.a, PRESS_ACTIVE);
    }

    #[test]
    fn faintest_ink_moves_bg() {
        for dark in [true, false] {
            let bg = if dark { grey(6) } else { grey(0xff) };
            let plate = flatten(ink_for(dark, 0.03), bg);
            assert!(
                (plate.l - bg.l).abs() >= 0.02,
                "dark={dark} delta {}",
                (plate.l - bg.l).abs()
            );
        }
    }

    #[test]
    fn scrim_uses_dark_popup_pigment_and_is_weaker_in_light() {
        let d = scrim_for(true, SCRIM_ALPHA_DARK);
        let l = scrim_for(false, SCRIM_ALPHA_DARK);
        assert_eq!(d.l, 15.0 / 255.0);
        assert_eq!(l.l, 0.0);
        assert!(l.a < d.a);
    }
}
