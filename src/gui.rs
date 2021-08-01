/*
 * Filename: /src/gui.rs
 * Project: rv6502emu
 * Created Date: Thursday, January 1st 1970, 1:00:00 am
 * Author: valerino <xoanino@gmail.com>
 * Copyright (c) 2021 valerino
 *
 * MIT License
 *
 * Copyright (c) 2021 valerino
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is furnished to do
 * so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use crate::cpu::{Cpu, Registers};
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use druid::widget::prelude::*;
use druid::widget::{Align, Button, Flex, Label, TextBox};
use druid::{
    AppDelegate, AppLauncher, Command, Data, DelegateCtx, Env, ExtEventSink, Handled, Lens,
    LocalizedString, Selector, Target, Widget, WidgetExt, WindowDesc, WindowId,
};
use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use std::time::Duration;
const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: &str = "Hello world";
const FINISH_SLOW_FUNCTION: Selector<u32> = Selector::new("finish_slow_function");

#[derive(Clone, Default, Data, Lens)]
struct AppState {
    processing: bool,
    value: u32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiContext {
    pub regs: Registers,
    pub cmd: serde_json::Value,
}

fn build_root_widget() -> impl Widget<AppState> {
    // a label that will determine its text based on the current app data.
    let label = Label::new(|data: &AppState, _env: &Env| format!("Hello World!"));
    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(AppState::name);

    let button = Button::new("Start slow increment")
        .on_click(|ctx, data: &mut AppState, _env| {
            data.processing = true;
            println!("CLICKED!!!!!!");
            // In order to make sure that the other thread can communicate with the main thread we
            // have to pass an external handle to the second thread.
            // Using this handle we can send commands back to the main thread.
            // wrapped_slow_function(ctx.get_external_handle(), data.value);
        })
        .padding(5.0);

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox)
        .with_child(button);

    // center the two widgets in the available space
    Align::centered(layout)
}

/**
 * this is the UI for our in-emulator debugger
 *
 * > uses https://github.com/linebender/druid, requires sudo apt-get install libgtk-3-dev
 */
pub struct DebuggerUi<'a> {
    hidden: bool,
    to_ui: &'a (Sender<UiContext>, Receiver<UiContext>),
    from_ui: &'a (Sender<UiContext>, Receiver<UiContext>),
}

struct Delegate {
    to_ui: (Sender<UiContext>, Receiver<UiContext>),
    from_ui: (Sender<UiContext>, Receiver<UiContext>),
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        /*
        if let Some(number) = cmd.get(FINISH_SLOW_FUNCTION) {
            // If the command we received is `FINISH_SLOW_FUNCTION` handle the payload.
            data.processing = false;
            data.value = *number;
            Handled::Yes
        } else {
            Handled::No
        }
        */
        Handled::Yes
    }

    fn window_added(
        &mut self,
        id: WindowId,
        data: &mut AppState,
        env: &Env,
        ctx: &mut DelegateCtx<'_>,
    ) {
        println!("window added!");
        let c = UiContext {
            regs: Registers::new(),
            cmd: serde_json::from_str("{}").unwrap(),
        };

        println!("sending!");
        self.from_ui.0.send(c);
    }
}

impl<'a> DebuggerUi<'a> {
    /*pub fn test_signal(&mut self) {
        AppState.get_external_handle();
    }*/

    pub fn run(&mut self) -> std::thread::JoinHandle<()> {
        let c_to_ui = self.to_ui.clone();
        let c_from_ui = self.from_ui.clone();
        let h = std::thread::spawn(move || {
            println!("ui thread handle={:?}", std::thread::current());
            // describe the main window
            let main_window = WindowDesc::new(build_root_widget)
                .title(WINDOW_TITLE)
                .window_size((400.0, 400.0));

            // start the application
            AppLauncher::with_window(main_window)
                .delegate(Delegate {
                    to_ui: c_to_ui,
                    from_ui: c_from_ui,
                })
                .launch(AppState::default())
                .expect("Failed to launch application");
        });
        h
    }
}

/**
 * gets an ui instance
 */
pub fn new<'a>(
    to_ui: &'a (Sender<UiContext>, Receiver<UiContext>),
    from_ui: &'a (Sender<UiContext>, Receiver<UiContext>),
) -> DebuggerUi<'a> {
    let ui = DebuggerUi {
        hidden: false,
        to_ui: to_ui,
        from_ui: from_ui,
    };
    ui
}
