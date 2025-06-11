#[derive(Debug, Clone, Copy)]
pub struct Matrix3x3 {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix3x3 {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn from_components(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    pub fn apply_to_point(&self, x: f32, y: f32) -> (f32, f32) {
        let x_prime = self.a * x + self.c * y + self.e;
        let y_prime = self.b * x + self.d * y + self.f;
        (x_prime, y_prime)
    }

    pub fn apply_to_origin(&self) -> (f32, f32) {
        (self.e, self.f)
    }
}

#[derive(Debug, Clone)]
pub struct PdfState {
    ctm: Matrix3x3,
    pub tm: Matrix3x3,
    leading: f32,
    pub m: (f32, f32),
}

impl Default for PdfState {
    fn default() -> Self {
        Self::new()
    }
}
impl PdfState {
    pub fn new() -> Self {
        Self {
            tm: Matrix3x3::identity(),
            ctm: Matrix3x3::identity(),
            leading: 0.0,
            m: (0.0, 0.0),
        }
    }

    pub fn bt(&mut self) {
        self.tm = Matrix3x3::identity();
    }

    pub fn et(&mut self) {
        // No-op
    }

    pub fn tl(&mut self, leading: f32) {
        self.leading = leading;
    }

    pub fn td(&mut self, tx: f32, ty: f32) {
        let translation = Matrix3x3::from_components(1.0, 0.0, 0.0, 1.0, tx, ty);
        self.tm = self.tm.multiply(&translation);
    }

    pub fn td_capital(&mut self, tx: f32, ty: f32) {
        self.leading = -ty;
        self.td(tx, ty);
    }

    pub fn tm(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let m = Matrix3x3::from_components(a, b, c, d, e, f);
        self.tm = m;
    }

    pub fn t_star(&mut self) {
        self.td(0.0, -self.leading);
    }

    pub fn cm(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.ctm = Matrix3x3::from_components(a, b, c, d, e, f);
    }

    pub fn m(&mut self, tx: f32, ty: f32) {
        self.m = (tx, ty);
    }

    pub fn current_position(&self) -> (f32, f32) {
        // can be wrong. but i think we apply the tm on the ctm and not the other way around
        let combined = self.ctm.multiply(&self.tm);
        let f = combined.apply_to_origin();
        f
    }

    pub fn l(&self, to: (f32, f32)) -> ((f32, f32), (f32, f32)) {
        let from = self.ctm.apply_to_point(self.m.0, self.m.1);
        let to = self.ctm.apply_to_point(to.0, to.1);
        (from, to)
    }

    pub fn re(&self, from: (f32, f32), to: (f32, f32)) -> ((f32, f32), (f32, f32)) {
        let from = self.ctm.apply_to_point(from.0, from.1);
        let to = self.ctm.apply_to_point(to.0, to.1);
        (from, to)
    }
}