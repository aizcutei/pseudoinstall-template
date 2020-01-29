#![windows_subsystem = "windows"]

use libc;
use winapi::ctypes::c_void;
use winapi::um::winnt::TOKEN_QUERY;
use std::ptr::null_mut;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::processthreadsapi::OpenProcessToken;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::TokenElevation;
use winapi::um::winnt::HANDLE;
use winapi::um::winnt::TOKEN_ELEVATION;

use std::{io, fs, process, mem};
use std::fs::File;
use std::path::Path;
use std::os::windows;
use druid::{Application, AppLauncher, Data, Lens, LensWrap, LocalizedString, UnitPoint, Widget, WindowDesc};
use druid::widget::{Align, Button, Checkbox, Container, Flex, Label, Padding, WidgetExt};

#[derive(Clone, Data, Lens)]

struct AppState {
    c32: bool,
    c64: bool,
    ctype1: bool,
    ctype2: bool,
    cadmin: bool,
    ctext: String,
    name: String,
}

impl AppState {
    fn mklink(&mut self) -> io::Result<()> {
        let src_path = Path::new("a");
        let dir_path = Path::new("b");
        if self.ctype1 {
            if self.c64 {
                windows::fs::symlink_dir(&src_path, &dir_path)?;
            }
            if self.c32 {
                windows::fs::symlink_dir(&src_path, &dir_path)?;
            }
        }
        if self.ctype2 {
            windows::fs::symlink_dir(&src_path, &dir_path)?;
        }
        Ok(())
    }

    fn rmlink(&mut self) -> io::Result<()> {
        let dir_path = Path::new("b");
        fs::remove_dir_all(&dir_path)?;
        Ok(())
    }

    fn exit(&mut self) {
        process::exit(1);
    }
}

fn main() {
    let astext = match check_admin() {
        true => "In Admin mode".to_string(),
        false => "Please run in Admin mode！".to_string(),
    };
    let data = AppState{
        c32: false,
        c64: true,
        ctype1: true,
        ctype2: false,
        cadmin: check_admin(),
        ctext: astext,
        name: "Install".to_string(),
    };
    let window_title = LocalizedString::new("Install");
    let main_window = WindowDesc::new(window_main).title(window_title).window_size((300.0, 400.0));
    AppLauncher::with_window(main_window).launch(data).expect("Error!");

}

fn window_main() -> impl Widget<AppState> {

    let label = LensWrap::new(Label::new(|data: &String, _env: &_| data.clone()), AppState::ctext).center();

    let text_install = LocalizedString::new("Install");
    let text_uninstall = LocalizedString::new("Uninstall");
    let text_exit = LocalizedString::new("Exit");

    let check_c32 = LensWrap::new(Checkbox::new(), AppState::c32);
    let ctext_c32 = LocalizedString::new("checkfor32bit");
    let label_c32 = Label::new(ctext_c32);
    let row_l1 = Flex::row()
        .with_child(check_c32, 0.0)
        .with_child(Padding::new(5.0, label_c32), 1.0);

    let check_c64 = LensWrap::new(Checkbox::new(), AppState::c64);
    let ctext_c64 = LocalizedString::new("checkfor64bit");
    let label_c64 = Label::new(ctext_c64);
    let row_r1 = Flex::row()
        .with_child(check_c64, 0.0)
        .with_child(Padding::new(5.0, label_c64), 1.0);

    let row_1 = Flex::row()
        .with_child(Label::new(""), 1.0)
        .with_child(Padding::new(0.0, row_l1), 1.0)
        .with_child(Padding::new(5.0, row_r1), 1.0)
        .with_child(Label::new(""), 1.0);

    let check_ctype1 = LensWrap::new(Checkbox::new(), AppState::ctype1);
    let ctext_ctype1 = LocalizedString::new("type1");
    let label_ctype1 = Label::new(ctext_ctype1);
    let row_l2 = Flex::row()
        .with_child(check_ctype1, 0.0)
        .with_child(Padding::new(5.0, label_ctype1), 1.0);
    let check_ctype2 = LensWrap::new(Checkbox::new(), AppState::ctype2);
    let ctext_ctype2 = LocalizedString::new("type2");
    let label_ctype2 = Label::new(ctext_ctype2);
    let row_r2 = Flex::row()
        .with_child(check_ctype2, 0.0)
        .with_child(Padding::new(0.0, label_ctype2), 1.0);

    let row_2 = Flex::row()
        .with_child(Label::new(""), 1.0)
        .with_child(Padding::new(0.0, row_l2), 1.0)
        .with_child(Padding::new(5.0, row_r2), 1.0)
        .with_child(Label::new(""), 1.0);

    let button_i = Button::new(text_install, |_ctx, data: &mut AppState, _env| {
        data.mklink();
    });/*
        .fix_width(100.0).fix_height(100.0)
        .align_vertical(UnitPoint::CENTER);
        */
    let button_u = Button::new(text_uninstall, |_ctx, data: &mut AppState, _env| {
        data.rmlink();
    });/*
        .fix_width(100.0).fix_height(100.0)
        .align_vertical(UnitPoint::CENTER); 
        */ 
    let row_3 = Flex::row()
        .with_child(button_i, 1.0)
        .with_child(button_u, 1.0);

    let button_e = Button::new(text_exit, |_ctx, data: &mut AppState, _env| {
        data.exit();
    });/*
        .fix_height(100.0).fix_width(250.0)
        .align_vertical(UnitPoint::CENTER);
        */
    

    let mut col = Flex::column();

    col.add_child(label, 1.0);
    col.add_child(Container::new(Label::new("")), 1.0);

    col.add_child(Padding::new(1.0, row_1), 1.0);
    col.add_child(Padding::new(1.0, row_2), 1.0);


    col.add_child(row_3, 1.0);
    col.add_child(button_e, 1.0);
    col.add_child(Container::new(Label::new("")), 1.0);

    col
}

fn check_admin() -> bool {

    let mut handle: HANDLE = null_mut();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) };

    let elevation = unsafe { libc::malloc(mem::size_of::<TOKEN_ELEVATION>()) as *mut c_void };
    let size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
    let mut ret_size = size;
    unsafe {
        GetTokenInformation(
            handle,
            TokenElevation,
            elevation,
            size as u32,
            &mut ret_size,
        )
    };
    let elevation_struct: TOKEN_ELEVATION = unsafe{ *(elevation as *mut TOKEN_ELEVATION)};

    if !handle.is_null() {
        unsafe {
            CloseHandle(handle);
        }
    }

    elevation_struct.TokenIsElevated == 1

}