use egui::{Color32, Painter, Rect, Shape, Stroke, Style, epaint::PathShape, pos2, vec2};
use egui_snarl::ui::{PinWireInfo, SnarlPin, SnarlStyle, WireStyle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PinShape {
    #[default]
    Circle,
    Diamond,
}

#[derive(Copy, Clone, Debug)]
pub enum PinSide {
    Left,
    Right,
}

#[derive(Default)]
pub struct PinInfo {
    pub side: Option<PinSide>,
    pub fill: Option<Color32>,
    pub shape: Option<PinShape>,
    pub wire_color: Option<Color32>,
    pub wire_style: Option<WireStyle>,
}

impl PinInfo {
    #[must_use]
    pub fn with_fill(mut self, color: Color32) -> Self {
        self.fill = Some(color);
        self
    }

    #[must_use]
    pub fn with_side(mut self, side: PinSide) -> Self {
        self.side = Some(side);
        self
    }
}

impl SnarlPin for PinInfo {
    fn pin_rect(&self, x: f32, y0: f32, y1: f32, size: f32) -> Rect {
        let y = (y0 + y1) * 0.5;

        let x_offset = match self.side {
            Some(PinSide::Left) => -size * 0.5,
            Some(PinSide::Right) => size * 0.5,
            None => 0.0,
        };

        let pin_pos = pos2(x + x_offset, y);
        Rect::from_center_size(pin_pos, vec2(size, size))
    }

    fn draw(
        self,
        snarl_style: &SnarlStyle,
        _style: &Style,
        rect: Rect,
        painter: &Painter,
    ) -> PinWireInfo {
        let shape = self.shape.unwrap_or_default();

        let default_fill = Color32::from_rgb(100, 150, 200);
        let default_stroke = Stroke::new(1.0, Color32::from_rgb(60, 90, 120));

        let fill = self.fill.or(snarl_style.pin_fill).unwrap_or(default_fill);
        let stroke = snarl_style.pin_stroke.unwrap_or(default_stroke);

        draw_pin(painter, shape, fill, stroke, rect);

        PinWireInfo {
            color: self.wire_color.unwrap_or(fill),
            style: self
                .wire_style
                .unwrap_or_else(|| snarl_style.wire_style.unwrap_or_default()),
        }
    }
}

fn draw_pin(painter: &Painter, shape: PinShape, fill: Color32, stroke: Stroke, rect: Rect) {
    let center = rect.center();
    let size = f32::min(rect.width(), rect.height());

    match shape {
        PinShape::Circle => {
            painter.circle(center, size / 2.0, fill, stroke);
        }
        PinShape::Diamond => {
            let points = vec![
                center + vec2(0.0, -0.70) * size,
                center + vec2(-0.70, 0.0) * size,
                center + vec2(0.0, 0.70) * size,
                center + vec2(0.70, 0.0) * size,
            ];

            painter.add(Shape::Path(PathShape {
                points,
                closed: true,
                fill,
                stroke: stroke.into(),
            }));
        }
    }
}
