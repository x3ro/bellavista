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

mod scanning;

use druid::widget::prelude::*;

use druid::{
    commands, AppDelegate, AppLauncher, Application, Command, DelegateCtx, FileDialogOptions,
    Handled, InternalEvent, Point, Target, TextAlignment, WidgetPod, WindowDesc,
};

use druid::widget::{Button, Flex, Label};
use scanning::Node;

use crate::widgets::boxes::FileBox;
use bytesize::ByteSize;
use std::path::Path;
use std::rc::Rc;
use std::{env, io};

mod widgets;

#[derive(Clone, Data)]
struct AppState {
    node: Option<Rc<Node>>,
    selected_file: Option<FileBox>,
}

struct ManagerWidget<T: Widget<AppState>> {
    child: WidgetPod<AppState, T>,
}

impl<T: Widget<AppState>> ManagerWidget<T> {
    fn open_dialog_cmd() -> impl Into<Command> {
        let open_dialog_options = FileDialogOptions::new()
            .name_label("Source")
            .select_directories()
            .title("Which directory do you wish to scan?")
            .button_text("Scan");

        Command::new(
            druid::commands::SHOW_OPEN_PANEL,
            open_dialog_options.clone(),
            Target::Auto,
        )
    }
}

impl<T: Widget<AppState>> Widget<AppState> for ManagerWidget<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        match event {
            Event::WindowConnected => {
                if data.node.is_none() {
                    ctx.submit_command(Self::open_dialog_cmd())
                }
            }
            Event::WindowSize(_) => {}
            Event::MouseDown(e) => {
                if let Some(file) = &data.selected_file {
                    if file.rect.contains(e.pos) {
                        let mut clipboard = Application::global().clipboard();
                        clipboard.put_string(file.path.to_owned());
                    }
                }
            }
            Event::MouseUp(_) => {}
            Event::MouseMove(_) => {}
            Event::Wheel(_) => {}
            Event::KeyDown(_) => {}
            Event::KeyUp(_) => {}
            Event::Paste(_) => {}
            Event::Zoom(_) => {}
            Event::Timer(_) => {}
            Event::AnimFrame(_) => {}
            Event::Command(_) => {}
            Event::Notification(_) => {}
            Event::Internal(e) => match e {
                InternalEvent::MouseLeave => {
                    data.selected_file = None;
                    //ctx.request_paint();
                }
                InternalEvent::TargetedCommand(_) => {}
                InternalEvent::RouteTimer(_, _) => {}
            },
        }
        self.child.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppState, data: &AppState, env: &Env) {
        if data.node.is_none() {
            ctx.submit_command(Self::open_dialog_cmd())
        }
        self.child.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, &bc.loosen(), data, env);
        self.child.set_origin(ctx, data, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}

pub fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let scan_result = if args.len() == 2 {
        Some(Rc::new(scanning::scan(Path::new(&args[1]))?))
    } else {
        None
    };

    let state = AppState {
        selected_file: None,
        node: scan_result,
    };

    let main_window = WindowDesc::new(build_root)
        .title("Bellavista")
        .window_size((600.0, 400.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .use_simple_logger()
        .launch(state)
        .expect("Failed to launch application");

    Ok(())
}

struct Delegate;
impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            data.node = Some(Rc::new(scanning::scan(file_info.path()).unwrap()));
            //ctx.submit_command(Command::new(druid::commands::))
            return Handled::Yes;
        }
        Handled::No
    }
}

fn build_root() -> impl Widget<AppState> {
    let boxes = widgets::boxes::Boxes { boxes: vec![], cached_image: None };

    let label = Label::new(|data: &AppState, _env: &Env| match &data.selected_file {
        Some(file) => format!("{} ({})", file.path, ByteSize(file.size)),
        _ => "Hover over an element too see it's path and size".to_string(),
    })
    .with_text_size(16.0)
    .with_text_alignment(TextAlignment::Start);

    let open = Button::<AppState>::new("Reset").on_click(move |_, data, _| {
        data.node = None;
        data.selected_file = None;
    });

    let child = Flex::column().with_flex_child(boxes, 1.0).with_child(
        Flex::row()
            .with_child(label)
            .with_flex_spacer(1.0)
            .with_child(open),
    );

    ManagerWidget {
        child: WidgetPod::new(child),
    }
}
