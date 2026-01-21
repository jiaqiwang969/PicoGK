//! Easing functions

/// Supported easing curves
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingKind {
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
}

/// Easing function implementations
pub struct Easing;

impl Easing {
    pub fn ease_sine_in(x: f32) -> f32 {
        1.0 - (x * std::f32::consts::PI / 2.0).cos()
    }

    pub fn ease_sine_out(x: f32) -> f32 {
        (x * std::f32::consts::PI / 2.0).sin()
    }

    pub fn ease_sine_in_out(x: f32) -> f32 {
        -((std::f32::consts::PI * x).cos() - 1.0) / 2.0
    }

    pub fn ease_quad_in(x: f32) -> f32 {
        x * x
    }

    pub fn ease_quad_out(x: f32) -> f32 {
        1.0 - (1.0 - x) * (1.0 - x)
    }

    pub fn ease_quad_in_out(x: f32) -> f32 {
        if x < 0.5 {
            2.0 * x * x
        } else {
            1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
        }
    }

    pub fn ease_cubic_in(x: f32) -> f32 {
        x * x * x
    }

    pub fn ease_cubic_out(x: f32) -> f32 {
        1.0 - (1.0 - x).powi(3)
    }

    pub fn ease_cubic_in_out(x: f32) -> f32 {
        if x < 0.5 {
            4.0 * x * x * x
        } else {
            1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
        }
    }

    pub fn easing_function(x: f32, kind: EasingKind) -> f32 {
        match kind {
            EasingKind::Linear => x,
            EasingKind::SineIn => Self::ease_sine_in(x),
            EasingKind::SineOut => Self::ease_sine_out(x),
            EasingKind::SineInOut => Self::ease_sine_in_out(x),
            EasingKind::QuadIn => Self::ease_quad_in(x),
            EasingKind::QuadOut => Self::ease_quad_out(x),
            EasingKind::QuadInOut => Self::ease_quad_in_out(x),
            EasingKind::CubicIn => Self::ease_cubic_in(x),
            EasingKind::CubicOut => Self::ease_cubic_out(x),
            EasingKind::CubicInOut => Self::ease_cubic_in_out(x),
        }
    }
}
