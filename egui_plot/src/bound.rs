///  boundary value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndKind {
    Open,
    Closed,
}

/// A numeric bound that can be finite or (semi-)infinite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bound {
    NegInf,
    Finite(f64, EndKind),
    PosInf,
}

impl Bound {
    #[inline]
    pub fn closed(v: f64) -> Self {
        Self::Finite(v, EndKind::Closed)
    }
    #[inline]
    pub fn open(v: f64) -> Self {
        Self::Finite(v, EndKind::Open)
    }

    #[inline]
    fn cmp_key(self) -> (i8, f64, i8) {
        match self {
            Self::NegInf => (-1, f64::NEG_INFINITY, 0),
            Self::Finite(v, EndKind::Open) => (0, v, 0),
            Self::Finite(v, EndKind::Closed) => (0, v, 1),
            Self::PosInf => (1, f64::INFINITY, 0),
        }
    }

    #[inline]
    pub fn value(self) -> Option<f64> {
        match self {
            Self::Finite(v, _) => Some(v),
            _ => None,
        }
    }
}

/// A numeric interval on ℝ with optional openness and infinity on either side.
/// Invariant: after `normalize()`, `start <= end` (with openness deciding emptiness at equality).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub start: Bound,
    pub end: Bound,
}

impl Interval {
    #[inline]
    pub fn new(start: Bound, end: Bound) -> Self {
        let mut s = Self { start, end };
        s.normalize();
        s
    }

    // Constructors
    #[inline]
    pub fn closed(a: f64, b: f64) -> Self {
        Self::new(Bound::closed(a), Bound::closed(b))
    }
    #[inline]
    pub fn open(a: f64, b: f64) -> Self {
        Self::new(Bound::open(a), Bound::open(b))
    }
    #[inline]
    pub fn open_closed(a: f64, b: f64) -> Self {
        Self::new(Bound::open(a), Bound::closed(b))
    }
    #[inline]
    pub fn closed_open(a: f64, b: f64) -> Self {
        Self::new(Bound::closed(a), Bound::open(b))
    }

    /// (-∞, b] or (-∞, b) depending on `kind`
    #[inline]
    pub fn below(b: f64, kind: EndKind) -> Self {
        Self::new(Bound::NegInf, Bound::Finite(b, kind))
    }
    /// [a, +∞) or (a, +∞) depending on `kind`
    #[inline]
    pub fn above(a: f64, kind: EndKind) -> Self {
        Self::new(Bound::Finite(a, kind), Bound::PosInf)
    }
    /// (-∞, +∞)
    #[inline]
    pub fn all() -> Self {
        Self::new(Bound::NegInf, Bound::PosInf)
    }

    #[inline]
    pub fn normalize(&mut self) {
        if self.start.cmp_key() > self.end.cmp_key() {
            std::mem::swap(&mut self.start, &mut self.end);
        }
    }

    /// True if the interval contains `x`.
    pub fn contains(&self, x: f64) -> bool {
        if !x.is_finite() {
            return false;
        }
        let left_ok = match self.start {
            Bound::NegInf => true,
            Bound::Finite(v, EndKind::Closed) => x >= v,
            Bound::Finite(v, EndKind::Open) => x > v,
            Bound::PosInf => false,
        };
        let right_ok = match self.end {
            Bound::PosInf => true,
            Bound::Finite(v, EndKind::Closed) => x <= v,
            Bound::Finite(v, EndKind::Open) => x < v,
            Bound::NegInf => false,
        };
        left_ok && right_ok
    }

    /// Empty if start==end and at least one side is open, or inverted infinities.
    pub fn is_empty(&self) -> bool {
        match (self.start, self.end) {
            (Bound::Finite(a, sa), Bound::Finite(b, sb)) => {
                if a < b {
                    false
                } else if a > b {
                    true
                } else {
                    // a == b
                    sa == EndKind::Open || sb == EndKind::Open
                }
            }
            (Bound::PosInf, _) | (_, Bound::NegInf) => true,
            _ => false,
        }
    }

    /// Intersection of two intervals.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let mut s = *self;
        let mut o = *other;
        s.normalize();
        o.normalize();

        let start = if s.start.cmp_key() >= o.start.cmp_key() {
            s.start
        } else {
            o.start
        };
        let end = if s.end.cmp_key() <= o.end.cmp_key() {
            s.end
        } else {
            o.end
        };

        let out = Self { start, end };
        if out.is_empty() { None } else { Some(out) }
    }

    pub fn to_screen_y(self, tf: &crate::PlotTransform) -> (f32, f32) {
        let frame = tf.frame();
        let y0 = match self.start {
            Bound::NegInf => frame.bottom(),
            Bound::Finite(v, _) => tf.position_from_point(&crate::PlotPoint::new(0.0, v)).y,
            Bound::PosInf => frame.top(),
        };
        let y1 = match self.end {
            Bound::PosInf => frame.top(),
            Bound::Finite(v, _) => tf.position_from_point(&crate::PlotPoint::new(0.0, v)).y,
            Bound::NegInf => frame.bottom(),
        };
        (y0.min(y1), y0.max(y1))
    }

    pub fn to_screen_x(self, tf: &crate::PlotTransform) -> (f32, f32) {
        let frame = tf.frame();
        let x0 = match self.start {
            Bound::NegInf => frame.left(),
            Bound::Finite(v, _) => tf.position_from_point(&crate::PlotPoint::new(v, 0.0)).x,
            Bound::PosInf => frame.right(),
        };
        let x1 = match self.end {
            Bound::PosInf => frame.right(),
            Bound::Finite(v, _) => tf.position_from_point(&crate::PlotPoint::new(v, 0.0)).x,
            Bound::NegInf => frame.left(),
        };
        (x0.min(x1), x0.max(x1))
    }
}

impl From<(Option<f64>, Option<f64>)> for Interval {
    /// Helper for migrating existing APIs that used `Option<f64>` min/max (closed ends).
    fn from((lo, hi): (Option<f64>, Option<f64>)) -> Self {
        let start = lo.map(Bound::closed).unwrap_or(Bound::NegInf);
        let end = hi.map(Bound::closed).unwrap_or(Bound::PosInf);
        Self::new(start, end)
    }
}
