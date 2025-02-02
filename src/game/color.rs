#[derive(Clone, Debug)]
pub enum Color {
    RGBA(u8, u8, u8, u8),
    HSVA(u8, u8, u8, u8),
}
impl Color {
    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::RGBA(r, g, b, a)
    }

    pub const fn new_hsva(h: u8, s: u8, v: u8, a: u8) -> Self {
        Self::HSVA(h, s, v, a)
    }

    pub fn rgba_from_slice(color_slice: &[u8; 4]) -> Self {
        Self::new_rgba(
            color_slice[0],
            color_slice[1],
            color_slice[2],
            color_slice[3],
        )
    }

    pub fn from_str(color_name: &str) -> Self {
        match color_name {
            "black" => Self::new_rgba(0, 0, 0, 255),
            "white" => Self::new_rgba(255, 255, 255, 255),
            "red" => Self::new_rgba(255, 0, 0, 255),
            "blue" => Self::new_rgba(0, 0, 255, 255),
            "green" => Self::new_rgba(0, 255, 0, 255),
            "purple" => Self::new_rgba(170, 0, 140, 255),
            "whine_red" => Self::new_rgba(88, 24, 31, 255),
            "orange" => Self::new_rgba(220, 115, 0, 255),
            "grey" => Self::new_rgba(80, 80, 80, 255),
            "yellow" => Self::new_rgba(255, 255, 0, 255),
            "pink" => Self::new_rgba(230, 120, 140, 255),
            x => panic!("Unknown color name '{}'", x),
        }
    }

    fn hsva_to_rgba(hsva: [u8; 4]) -> [u8; 4] {
        let [h, s, v, a] = hsva;
        let h_f = h as f64 / 255.0;
        let s_f = s as f64 / 255.0;
        let v_f = v as f64 / 255.0;

        let c = v_f * s_f;
        let h_dash = h_f * 6.0;
        let x = c * (1.0 - (h_dash % 2.0 - 1.0).abs());

        let m = v_f - c;
        let c = ((c + m) * 255.0) as u8;
        let x = ((x + m) * 255.0) as u8;
        let m = (m * 255.0) as u8;

        match h_dash {
            f if f < 1.0 => [c, x, m, a],
            f if f < 2.0 => [x, c, m, a],
            f if f < 3.0 => [m, c, x, a],
            f if f < 4.0 => [m, x, c, a],
            f if f < 5.0 => [x, m, c, a],
            f if f <= 6.0 => [c, m, x, a],
            _ => panic!("Something went very wrong when converting from hsva to rgba. There is no possibility to end up here, but we managed. The only sollution is to end this. Everything goes dark and you die."),
        }
    }

    pub fn to_rgba(&self) -> Self {
        match self {
            Self::RGBA(..) => self.clone(),
            Self::HSVA(h, s, v, a) => {
                let [r, g, b, a] = Self::hsva_to_rgba([*h, *s, *v, *a]);
                Self::new_rgba(r, g, b, a)
            }
        }
    }

    pub fn blend(&self, other: &Self) -> Self {
        let [r_a, g_a, b_a, a_a] = self.to_rgba().to_slice();
        let [r_b, g_b, b_b, a_b] = other.to_rgba().to_slice();
        let a_a = a_a as f64 / 255.0;
        let a_b = a_b as f64 / 255.0;
        let a_c = a_a + (1.0 - a_a) * a_b;
        let r_c = (a_a * r_a as f64 + (1.0 - a_a) * a_b * r_b as f64) / a_c;
        let g_c = (a_a * g_a as f64 + (1.0 - a_a) * a_b * g_b as f64) / a_c;
        let b_c = (a_a * b_a as f64 + (1.0 - a_a) * a_b * b_b as f64) / a_c;

        Self::new_rgba(
            r_c.round() as u8,
            g_c.round() as u8,
            b_c.round() as u8,
            (a_c * 255.0).round() as u8,
        )
    }

    pub fn to_slice(&self) -> [u8; 4] {
        match self {
            Self::RGBA(r, g, b, a) => [*r, *g, *b, *a],
            Self::HSVA(h, s, v, a) => [*h, *s, *v, *a],
        }
    }
}
impl From<&str> for Color {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsva_to_rgba() {
        for _ in 0..=255 {
            Color::new_hsva(0, 255, 255, 255).to_rgba();
        }
    }
}
