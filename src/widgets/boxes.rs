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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::swap;

use druid::{
    BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Rect, RenderContext, Size, UpdateCtx, Widget,
};
use druid::widget::prelude::Data;
use druid::piet::InterpolationMode;
use druid::piet::PietImage;

use crate::AppState;
use crate::scanning::Node;

#[derive(Debug, Clone, Data)]
pub struct FileBox {
    pub path: String,
    pub size: u64,
    pub rect: Rect,
    pub parent: Option<Rect>,
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


// fn d2xy(n: u64, d: u64) -> (u64, u64) {
//     let mut t = d;
//     let mut x = 0;
//     let mut y = 0;
//     let mut rx = 0;
//     let mut ry = 0;
//     let mut s = 1;
//
//     while s < n {
//         s *= 2;
//         rx = 1 & (t/2);
//         ry = 1 & (t ^ rx);
//         (x, y) = rot(n, x, y, rx, ry);
//         x += s * rx;
//         y += s * ry;
//         t /= 4;
//     }
//
//     (x, y)
// }
//
// fn rot(n: u64, mut x: u64, mut y: u64, rx: u64, ry: u64) -> (u64, u64) {
//     if ry == 0 {
//         if rx == 1 {
//             x = n - 1 - x;
//             y = n - 1 - y;
//         }
//         swap(&mut x , &mut y);
//     }
//     (x, y)
// }

/// Gives the highest aspect ratio of a list of rectangles (`row`),
/// given the `length` of the side along which they are to be
/// laid out.
fn worst(row: &[u64], length: f64) -> f64 {
    let sum: u64 = row.iter().sum();
    let max = (*row.iter().max().unwrap()) as f64;
    let min = (*row.iter().min().unwrap()) as f64;

    let lpow = length.powf(2.0) as f64;
    let spow = sum.pow(2) as f64;

    let left = (lpow * max) / spow;
    let right = spow / (lpow * min);

    left.max(right)
}

struct Layouting {
    remaining: Rect,

}

impl Layouting {
    /// gives the length of the shortest side of the remaining sub-rectangle
    fn width(&self) -> f64 {
        if self.remaining.aspect_ratio() > 1.0 {
            self.remaining.width()
        } else {
            self.remaining.height()
        }
    }

    fn layoutrow(&self, row: &[u64]) {
        todo!()
    }

    fn squarify(&self, children: &[u64], row: Vec<u64>, w: f64) {
        let c = children.first().unwrap();

        let mut next_row = row.clone();
        next_row.push(*c);
        if worst(&row, w) <= worst(&next_row, w) {
            self.squarify(&children[1..], next_row, w);
        } else {
            self.layoutrow(&row);
            self.squarify(children, vec![], self.width());
        }

    }
}



type BoxData = AppState;
impl Boxes {
    fn foo_rect(&mut self, root: &Node, bounds: Rect, parent: Option<Rect>) {
        match &root.children {
            Some(children) => {
                let mut row = vec![];
                for c in children {
                    row.push(c.size);
                }

                // println!("{}", worst(&row, 5));
                // println!("{}", worst(&row, 50));
                // println!("{}", worst(&row, 500));

                // let mut smallest_size = u64::MAX;
                // let mut size_sum = 0;
                // for c in children {
                //     let mut size = c.size / 4096;
                //     if size < 1 {
                //         size = 1;
                //     }
                //
                //     size_sum += size;
                //     if smallest_size > size {
                //         smallest_size = size;
                //     }
                // }
                //
                // size_sum /= smallest_size;
                // let mut sqrt = (size_sum as f64).sqrt().ceil() as u64;
                // if sqrt < 2 {
                //     sqrt = 2;
                // }
                //
                // println!("sqrt {}", sqrt);
                // println!("test {:?}", d2xy(sqrt / 2, sqrt));

                let mut remaining_size = root.size;
                let mut area = bounds;
                for c in children {
                    let ratio: f64 = c.size as f64 / remaining_size as f64;
                    let (left, right) = divide_rect(area, ratio);
                    self.foo_rect(c, left, Some(bounds));
                    area = right;
                    remaining_size -= c.size;
                }
            }
            None => {
                self.boxes.push(FileBox {
                    path: root.path.to_owned(),
                    size: root.size,
                    rect: bounds,
                    parent
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
            Event::WindowCloseRequested => {}
            Event::WindowDisconnected => {}
            Event::ImeStateChange => {}
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
                    self.foo_rect(node, size.to_rect(), None);
                }
            }
            LifeCycle::HotChanged(_) => {}
            LifeCycle::FocusChanged(_) => {}
            LifeCycle::Internal(_) => {}
            LifeCycle::DisabledChanged(_) => {}
            LifeCycle::BuildFocusChain => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &BoxData, data: &BoxData, _env: &Env) {
        if old_data.node != data.node {
            println!("Node data changed, recalculating");
            self.boxes = vec![];
            self.cached_image = None;

            if let Some(node) = &data.node {
                self.foo_rect(node, ctx.size().to_rect(), None);
            }

            ctx.request_paint();
        }

        if let Some(_file) = &data.selected_file {
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

            // TODO: I currently have no idea why I have to specify the capture rect twice as large
            // as the draw rect, will have to test more once I'm home again and have a non-retina
            // display, feels like it's related to that.
            let capture_rect = Rect {
                x0: 0.0,
                y0: 0.0,
                x1: rect.x1*2.0,
                y1: rect.y1*2.0
            };

            // let img = ctx.render_ctx.capture_image_area(capture_rect).unwrap();
            // self.cached_image = Some(img);
        }

        match &data.selected_file {
            None => {}
            Some(file) => {
                ctx.fill(file.rect, &Color::WHITE);
                if let Some(parent) = &file.parent {
                    ctx.stroke(parent, &Color::WHITE, 3.0);
                }

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
