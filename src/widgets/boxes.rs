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

use druid::{BoxConstraints, Color, Env, Event, EventCtx, kurbo, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget};
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

struct Layouting<'a> {
    remaining: Rect,
    result: Vec<(Rect, &'a Node)>,
    root_size: u64,
}

#[derive(PartialEq)]
enum LayoutDirection {
    Vertical,
    Horizontal
}

macro_rules! print_entity {
    ($x: expr) => {
        {
            println!("    -> {}: {:?}", stringify!($x), $x);
        }
    }
}

impl<'a> Layouting<'a> {
    /// gives the length of the shortest side of the remaining sub-rectangle
    fn short_length(&self) -> f64 {
        if self.remaining.aspect_ratio() > 1.0 {
            self.remaining.width()
        } else {
            self.remaining.height()
        }
    }

    fn long_length(&self) -> f64 {
        if self.remaining.aspect_ratio() > 1.0 {
            self.remaining.height()
        } else {
            self.remaining.width()
        }
    }

    fn layout_direction(&self) -> LayoutDirection {
        if self.remaining.aspect_ratio() > 1.0 {
            LayoutDirection::Horizontal
        } else {
            LayoutDirection::Vertical
        }
    }

    fn layout_row(&mut self, row: Vec<&'a Node>, ratio: f64) {
        let row_area = row.iter().fold(0.0, |acc, n| acc + (n.size as f64));
        let area_ratio = row_area / self.remaining.area();
        let long_length = area_ratio * self.long_length();

        println!("Layouting");
        //print_entity!(row);
        print_entity!(ratio);
        print_entity!(row_area);
        print_entity!(area_ratio);
        print_entity!(long_length);

        // We now know that all of the rectangles we're layouting will have one side
        // with length = `long_length`.

        // Now we need to compute the other sides. And we're always laying out against the short
        // side of the remaining rectangle.
        let mut cursor = if LayoutDirection::Vertical == self.layout_direction() {
            self.remaining.y0
        } else {
            self.remaining.x0
        };

        for node in row {
            let ratio = (node.size as f64) / row_area;
            assert!(ratio.is_finite());
            let short_length = ratio * self.short_length();

            //println!("Laying out {:?}", node);

            print_entity!(ratio);
            print_entity!(short_length);
            print_entity!(cursor);

            let rect = if LayoutDirection::Vertical == self.layout_direction() {
                Rect::from_origin_size((self.remaining.x0, cursor), (long_length, short_length))
            } else {
                Rect::from_origin_size((cursor, self.remaining.y0), (short_length, long_length))
            };
            print_entity!(rect);

            cursor += short_length;
            self.result.push((rect, node));
        }

        let x0 = self.remaining.x0;
        let y0 = self.remaining.y0;
        let x1 = self.remaining.x1;
        let y1 = self.remaining.y1;

        self.remaining = if LayoutDirection::Vertical == self.layout_direction() {
            Rect::new(x0 + long_length, y0, x1, y1)
        } else {
            Rect::new(x0, y0 + long_length, x1, y1)
        };

        print_entity!(self.remaining);
    }

    fn squarify(&mut self, children: &'a [Node], row: Vec<&'a Node>) {
        println!("squarify()");
        if children.is_empty() {
            let length = self.short_length();
            let current_worst = Self::worst(&row, length);
            self.layout_row(row, current_worst);
            return
        }

        let c = children.first().unwrap();
        let length = self.short_length();

        let mut next_row = row.clone();
        next_row.push(c);


        let current_worst = Self::worst(&row, length);
        let next_worst = Self::worst(&next_row, length);
        println!("{} >= {} -> {}", current_worst, next_worst, current_worst <= next_worst);
        if current_worst <= 0.0 || current_worst >= next_worst {
            self.squarify(&children[1..], next_row);
        } else {
            self.layout_row(row, current_worst);
            self.squarify(children, vec![]);
        }

    }



    /// Gives the highest aspect ratio of a list of rectangles (`row`),
    /// given the `length` of the side along which they are to be
    /// laid out.
    fn worst(nodes: &[&Node], length: f64) -> f64 {
        let sizes: Vec<_> = nodes.iter().map(|n| {
            print_entity!(n.size);
            //(n.size as f64 / self.root_size)
            n.size as f64
        }).collect();
        worst(&sizes, length)
    }

    fn start(&mut self, children: &'a [Node]) {


        match &children {
            &[] => {
                println!("Empty slice passed to layouting");
                return
            }

            [first, children @ ..]  => {
                self.squarify(children, vec![first]);
            }

            x => todo!("only one node")
        }
    }
}

/// Gives the highest aspect ratio of a list of rectangles (`row`),
/// given the `length` of the side along which they are to be
/// laid out.
fn worst(row: &[f64], length: f64) -> f64 {
    let sum: f64 = row.iter().sum();
    let max = row.iter().copied().fold(f64::MIN, f64::max);
    let min = row.iter().copied().fold(f64::MAX, f64::min);

    let lpow = length.powf(2.0);
    let spow = sum.powf(2.0);

    let left = (lpow * max) / spow;
    let right = spow / (lpow * min);

    let mut result = left.max(right);
    if !result.is_finite() {
        if left.is_finite() {
            result = left;
        } else {
            result = right;
        }
    };

    println!("worst()");
    print_entity!(row);
    print_entity!(length);
    print_entity!(sum);
    print_entity!(max);
    print_entity!(min);
    print_entity!(lpow);
    print_entity!(spow);
    print_entity!(left);
    print_entity!(right);
    print_entity!(result);

    assert!(result.is_finite());
    result
}


type BoxData = AppState;
impl Boxes {
    fn foo_rect(&mut self, root: &Node, bounds: Rect, parent: Option<Rect>) {
        match &root.children {
            Some(children) => {
                // let mut row = vec![];
                // for c in children {
                //     row.push(c.size);
                // }

                let mut l = Layouting {
                    remaining: bounds.clone(),
                    result: vec![],
                    root_size: root.size,

                };

                l.start(&children);

                for (rect, node) in l.result {
                    self.foo_rect(node, rect, Some(bounds));
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

                // let mut remaining_size = root.size;
                // let mut area = bounds;
                // for c in children {
                //     let ratio: f64 = c.size as f64 / remaining_size as f64;
                //     let (left, right) = divide_rect(area, ratio);
                //     self.foo_rect(c, left, Some(bounds));
                //     area = right;
                //     remaining_size -= c.size;
                // }



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
            // let capture_rect = Rect {
            //     x0: 0.0,
            //     y0: 0.0,
            //     x1: rect.x1*2.0,
            //     y1: rect.y1*2.0
            // };

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

    fn make_leaf(size: u64) -> Node {
        Node {
            size,
            path: format!("{}", size),
            children: None
        }
    }

    fn assert_eq_rect(actual: &Rect, expected: &Rect) {
        assert_approx_eq!(actual.x0, expected.x0);
        assert_approx_eq!(actual.y0, expected.y0);
        assert_approx_eq!(actual.x1, expected.x1);
        assert_approx_eq!(actual.y1, expected.y1);
    }

    #[test]
    fn test_layouting() {
        let leafs: Vec<_> = vec![6, 6, 4, 3, 2, 2, 1].into_iter().map(make_leaf).collect();
        let mut layout = Layouting {
            remaining: Rect::new(0.0, 0.0, 6.0, 4.0),
            result: vec![]
        };

        layout.start(&leafs);

        let (rect, _) = layout.result.get(0).unwrap();
        assert_eq_rect(rect, &Rect::new(0.0, 0.0, 3.0, 2.0));

        let (rect, _) = layout.result.get(1).unwrap();
        assert_eq_rect(rect, &Rect::new(0.0, 2.0, 3.0, 4.0));

        let (rect, _) = layout.result.get(2).unwrap();
        assert_eq_rect(rect, &Rect::new(
            3.0,
            0.0,
            3.0 + (4.0/7.0)*3.0,
            (7.0/12.0)*4.0
        ));

        let (rect, _) = layout.result.get(3).unwrap();
        assert_eq_rect(rect, &Rect::new(
            3.0 + (4.0/7.0) * 3.0,
            0.0,
            6.0,
            (7.0/12.0)*4.0
        ));

        let (rect, _) = layout.result.get(4).unwrap();
        assert_eq_rect(rect, &Rect::new(
            3.0,
            (7.0/12.0)*4.0,
            3.0 + (2.0/5.0)*3.0,
            4.0,
        ));

        let (rect, _) = layout.result.get(5).unwrap();
        assert_eq_rect(rect, &Rect::new(
            3.0 + (2.0/5.0)*3.0,
            (7.0/12.0)*4.0,
            3.0 + (2.0/5.0)*3.0 + (2.0/5.0) * 3.0,
            4.0,
        ));

        let (rect, _) = layout.result.get(6).unwrap();
        assert_eq_rect(rect, &Rect::new(
            3.0 + (2.0/5.0)*3.0 + (2.0/5.0) * 3.0,
            (7.0/12.0)*4.0,
            6.0,
            4.0,
        ));

    }

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

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_worst() {
        // Layout for the sequence 6, 6, 4, 3, 2, 2, 1 (sum = 24)
        // in a rectangle 6 x 4 units

        // First layout step

        let row = vec![6.0];
        let res = worst(&row, 4.0);
        assert_approx_eq!(res, 8.0/3.0);

        let row = vec![6.0, 6.0];
        let res = worst(&row, 4.0);
        assert_approx_eq!(res, 3.0/2.0);

        let row = vec![6.0, 6.0, 4.0];
        let res = worst(&row, 4.0);
        assert_approx_eq!(res, 4.0/1.0);

        // Second layout step

        let row = vec![4.0];
        let res = worst(&row, 3.0);
        assert_approx_eq!(res, 9.0/4.0);

        let row = vec![4.0, 3.0];
        let res = worst(&row, 3.0);
        assert_approx_eq!(res, 49.0/27.0);

        let row = vec![4.0, 3.0, 2.0];
        let res = worst(&row, 3.0);
        assert_approx_eq!(res, 9.0/2.0);

        // Third layout step

        let row = vec![2.0];
        let res = worst(&row, 5.0/3.0);
        assert_approx_eq!(res, 25.0/18.0);

        let row = vec![2.0, 2.0];
        let res = worst(&row, 5.0/3.0);
        assert_approx_eq!(res, 144.0/50.0);

        // Fourth layout step

        let row = vec![2.0];
        let res = worst(&row, 5.0/3.0);
        assert_approx_eq!(res, 25.0/18.0);

        // let row = vec![2.0, 1.0];
        // let res = worst(&row, 0.6);
        // assert_approx_eq!(res, 25.0/9.0);

    }

}
