pub mod color {
    use crate::models::{RGB, XYZ};
    use core::f32;
    use std::f64::consts::PI;

    fn illuminant_d(temp: f32, x: &mut f32, y: &mut f32) {
        if temp >= 2500.0 && temp <= 7000.0 {
            *x = 0.244063 + 0.09911e3 / temp + 2.9678e6 / temp.powi(2) - 4.6070e9 / temp.powi(3);
        } else if temp > 7000.0 && temp <= 25000.0 {
            *x = 0.237040 + 0.24748e3 / temp + 1.9018e6 / temp.powi(2) - 2.0064e9 / temp.powi(3);
        } else {
            return;
        }

        *y = (-3.0 * x.powi(2)) + (2.870 * (*x)) - 0.275;
    }

    fn lanckian_locus(temp: f32, x: &mut f32, y: &mut f32) {
        // https://en.wikipedia.org/wiki/Planckian_locus#Approximation
        if temp >= 1667.0 && temp <= 4000.0 {
            *x = -0.2661239e9 / temp.powi(3) - 0.2343589e6 / temp.powi(2)
                + 0.8776956e3 / temp
                + 0.179910;
            if temp <= 2222.0 {
                *y = -1.1064814 * (*x).powi(3) - 1.34811020 * (*x).powi(2) + 2.18555832 * *x
                    - 0.20219683;
            } else {
                *y = -0.9549476 * (*x).powi(3) - 1.37418593 * (*x).powi(2) + 2.09137015 * *x
                    - 0.16748867;
            }
        } else if temp > 4000.0 && temp < 25000.0 {
            *x = -3.0258469e9 / temp.powi(3)
                + 2.1070379e6 / temp.powi(2)
                + 0.2226347e3 / temp
                + 0.240390;
            *y =
                3.0817580 * (*x).powi(3) - 5.87338670 * (*x).powi(2) + 3.75112997 * *x - 0.37001483;
        } else {
            return;
        }
    }

    fn srgb_gamma(value: f32, gamme: f32) -> f32 {
        // https://en.wikipedia.org/wiki/SRGB
        if value <= 0.0031308 {
            return 12.92 * value;
        } else {
            return (1.055 * value).powf(1.0 / gamme) - 0.055;
        }
    }

    fn clamp(value: f32) -> f32 {
        if value > 1.0 {
            return 1.0;
        } else if value < 0.0 {
            return 0.0;
        } else {
            return value;
        }
    }

    fn xyz_to_srgb(xyz: &XYZ) -> RGB {
        // http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
        let rgp = RGB {
            r: srgb_gamma(
                clamp(3.2404542 * xyz.x - 1.5371385 * xyz.y - 0.4985314 * xyz.z),
                2.2,
            ),
            g: srgb_gamma(
                clamp(-0.9692660 * xyz.x + 1.8760108 * xyz.y + 0.0415560 * xyz.z),
                2.2,
            ),
            b: srgb_gamma(
                clamp(0.0556434 * xyz.x - 0.2040259 * xyz.y + 1.0572252 * xyz.z),
                2.2,
            ),
        };
        return rgp;
    }

    fn fmax(x: f32, y: f32) -> f32 {
        if x > y {
            x
        } else {
            y
        }
    }

    fn srgb_normalize(rgb: &mut RGB) {
        let maxw: f32 = fmax(rgb.r, fmax(rgb.g, rgb.b));

        rgb.r /= maxw;
        rgb.g /= maxw;
        rgb.b /= maxw;
    }

    pub fn calc_whitepoint(temp: f32) -> RGB {
        if temp == 6500.0 {
            return RGB {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            };
        }

        let mut wp = XYZ {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        if temp >= 25000.0 {
            illuminant_d(25000.0, &mut wp.x, &mut wp.y);
        } else if temp >= 4000.0 {
            illuminant_d(temp, &mut wp.x, &mut wp.y);
        } else if temp >= 2500.0 {
            let (mut x1, mut y1, mut x2, mut y2): (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.0);
            illuminant_d(temp, &mut x1, &mut y1);
            lanckian_locus(temp, &mut x2, &mut y2);
            let factor: f32 = (4000. - temp) / 1500.;
            let sinefactor: f32 = ((PI as f32) * factor).cos() + 1.0 / 2.0;
            wp.x = x1 * sinefactor + x2 * (1.0 - sinefactor);
            wp.y = y1 * sinefactor + y2 * (1.0 - sinefactor);
        } else {
            let args = if temp >= 1667.0 { temp } else { 1667.0 };
            lanckian_locus(args, &mut wp.x, &mut wp.y);
        }

        wp.z = 1.0 - wp.x - wp.y;

        let mut wp_rgb: RGB = xyz_to_srgb(&wp);
        srgb_normalize(&mut wp_rgb);

        return wp_rgb;
    }
}
