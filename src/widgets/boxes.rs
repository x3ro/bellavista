// Bellavista
// Copyright (C) 2021  Lucas Jenß
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::scanning::Node;
use crate::AppState;
use druid::{
    BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Rect, RenderContext, Size, UpdateCtx, Widget,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use druid::widget::prelude::Data;
use piet::{ImageFormat, Image, InterpolationMode};
use piet_common::{Piet, PietImage};

#[derive(Debug, Clone, Data)]
pub struct FileBox {
    pub path: String,
    pub size: u64,
    pub rect: Rect,
}

//#[derive(Debug)]
pub struct Boxes {
    pub boxes: Vec<FileBox>,
    pub cached_image: Option<PietImage>
    //pub active: Option<FileBox>,
}

fn color_for_path(path: &String) -> Color {


    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    Color::rgba8((hasher.finish() % 255) as u8, 0x00, 255 as u8, 255)
}

fn divide_rect(source: Rect, ratio: f64) -> (Rect, Rect) {
    if ratio < 0.0 || ratio > 1.0 {
        panic!("Ratio was out of bounds: {}", ratio)
    }

    // aspect ratio -> height / width
    if source.aspect_ratio() < 1.2 {
        let left = Rect {
            x0: source.x0,
            y0: source.y0,
            x1: source.x1 - source.width() * (1.0 - ratio),
            y1: source.y1,
        };

        let right = Rect {
            x0: source.x0 + source.width() * ratio,
            y0: source.y0,
            x1: source.x1,
            y1: source.y1,
        };

        (left, right)
    } else {
        let left = Rect {
            x0: source.x0,
            y0: source.y0,
            x1: source.x1,
            y1: source.y1 - source.height() * (1.0 - ratio),
        };

        let right = Rect {
            x0: source.x0,
            y0: source.y0 + source.height() * ratio,
            x1: source.x1,
            y1: source.y1,
        };

        (left, right)
    }
}

type BoxData = AppState;
impl Boxes {
    fn foo_rect(&mut self, root: &Node, bounds: Rect) {
        match &root.children {
            Some(children) => {
                let mut remaining_size = root.size;
                let mut area = bounds;
                for c in children {
                    let ratio: f64 = c.size as f64 / remaining_size as f64;
                    let (left, right) = divide_rect(area, ratio);
                    self.foo_rect(c, left);
                    area = right;
                    remaining_size -= c.size;
                }
            }
            None => {
                self.boxes.push(FileBox {
                    path: root.path.to_owned(),
                    size: root.size,
                    rect: bounds,
                });
            }
        }
    }
}

impl Widget<BoxData> for Boxes {
    fn event(&mut self, _ctx: &mut EventCtx, event: &Event, data: &mut BoxData, _env: &Env) {
        match event {
            Event::WindowConnected => {}
            Event::WindowSize(_) => {}
            Event::MouseDown(_) => {}
            Event::MouseUp(_) => {}
            Event::MouseMove(e) => {
                for b in &self.boxes {
                    if b.rect.contains(e.pos) {
                        data.selected_file = Some(b.clone());
                        break;
                    }
                }
            }
            Event::Wheel(_) => {}
            Event::KeyDown(_) => {}
            Event::KeyUp(_) => {}
            Event::Paste(_) => {}
            Event::Zoom(_) => {}
            Event::Timer(_) => {}
            Event::AnimFrame(_) => {}
            Event::Command(_) => {}
            Event::Notification(_) => {}
            Event::Internal(_) => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &BoxData,
        _env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {}
            LifeCycle::Size(size) => {
                if let Some(node) = &data.node {
                    self.cached_image = None;
                    self.boxes = vec![];
                    self.foo_rect(node, size.to_rect());
                }
            }
            LifeCycle::HotChanged(_) => {}
            LifeCycle::FocusChanged(_) => {}
            LifeCycle::Internal(_) => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &BoxData, data: &BoxData, _env: &Env) {
        if old_data.node != data.node {
            println!("Node data changed, recalculating");
            self.boxes = vec![];

            if let Some(node) = &data.node {
                self.foo_rect(node, ctx.size().to_rect());
            }

            ctx.request_paint();
        }

        if let Some(file) = &data.selected_file {
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &BoxData,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &BoxData, _env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();

        if let Some(cached_image) = &self.cached_image {
            ctx.draw_image(cached_image, rect, InterpolationMode::NearestNeighbor);
        } else {
            let debug_col = Color::from_hex_str("#ff00ff").unwrap();
            ctx.fill(rect, &debug_col);

            for b in &self.boxes {
                let color = color_for_path(&b.path);
                ctx.fill(b.rect, &color);
            }

            let img = ctx.render_ctx.save_image(rect).unwrap();
            self.cached_image = Some(img);
        }

        match &data.selected_file {
            None => {}
            Some(file) => {
                ctx.fill(file.rect, &Color::WHITE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_rect_horizontal() {
        let source = Rect {
            x0: 0.0,
            y0: 0.0,
            x1: 100.0,
            y1: 100.0,
        };

        let (first, second) = divide_rect(source, 0.3);
        assert_eq!(
            first,
            Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 30.0,
                y1: 100.0,
            }
        );

        assert_eq!(
            second,
            Rect {
                x0: 30.0,
                y0: 0.0,
                x1: 100.0,
                y1: 100.0,
            }
        );
    }

    #[test]
    fn test_divide_rect_vertical() {
        let source = Rect {
            x0: 0.0,
            y0: 0.0,
            x1: 100.0,
            y1: 200.0,
        };

        let (first, second) = divide_rect(source, 0.3);
        assert_eq!(
            first,
            Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 60.0,
            }
        );

        assert_eq!(
            second,
            Rect {
                x0: 0.0,
                y0: 60.0,
                x1: 100.0,
                y1: 200.0,
            }
        );
    }
}
