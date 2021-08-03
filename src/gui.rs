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
use gtk::prelude::*;
use gtk::Application;
use log::*;
use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct UiContext {
    pub regs: Registers,
    pub cmd: serde_json::Value,
}

/**
 * this is the UI for our in-emulator debugger
 *
 * > uses https://github.com/linebender/druid, requires sudo apt-get install libgtk-3-dev
 */
pub struct DebuggerUi {
    hidden: bool,
    pub r_s_chn: (Sender<UiContext>, Receiver<UiContext>),
    pub app: Application, //from_ui: &'a (Sender<UiContext>, Receiver<UiContext>),
}

impl DebuggerUi {
    /*pub fn test_signal(&mut self) {
        AppState.get_external_handle();
    }*/

    pub fn start_comm_thread(&mut self) -> std::thread::JoinHandle<()> {
        let comm_thread = std::thread::spawn(move || {
            debug!("comm thread running");
            loop {
                let start = std::time::Instant::now();
                let pause = std::time::Duration::from_millis(1000);
                debug!("comm thread spinning");

                while start.elapsed() < pause {
                    std::thread::yield_now();
                }
            }
            debug!("comm thread terminated");
        });
        comm_thread
    }

    pub fn run(&mut self) {
        println!("connect!");
        self.app.connect_activate(build_ui);
        println!("run!");
        let res = self.app.run();
        println!("res={}", res);
    }
}

fn build_ui(app: &gtk::Application) {
    println!("buildui!");
    let window = gtk::ApplicationWindow::new(app);
    window.set_title("First GTK+ Program");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let button = gtk::Button::with_label("Click me!");

    window.add(&button);

    window.show_all();
    println!("hello!");
}

/**
 * gets an ui instance
 */
pub fn new() -> DebuggerUi {
    let (r, s) = crossbeam_channel::unbounded();
    let app = gtk::Application::new(Some("org.gtk.example"), Default::default());
    let d = DebuggerUi {
        hidden: false,
        r_s_chn: (r, s),
        app: app,
    };
    d
}
