use vst::editor::Editor;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use crate::{atomic_f64::AtomicF64, eq_core::units::Units};

use crate::EQEffectParameters;

use tuix::*;

mod channel_controls;
mod equi;
mod graph;

use equi::{EQEvent, EQUI};

use std::sync::Arc;

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;

static THEME: &str = include_str!("editor/theme.css");

pub struct EQPluginEditor {
    pub is_open: bool,
    pub params: Arc<EQEffectParameters>,
    pub sample_rate: Arc<AtomicF64>,
}

impl Editor for EQPluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        if self.is_open {
            return false;
        }

        self.is_open = true;

        let params = self.params.clone();
        let sample_rate = self.sample_rate.clone();

        let window_description = WindowDescription::new()
            .with_title("EQ PLUGIN")
            .with_inner_size(800, 600);
        let app = Application::new(window_description, move |state, window| {
            state.add_theme(THEME);

            EQUI::new(params.clone(), sample_rate.clone()).build(state, window, |builder| builder);
        });

        app.open_parented(&VstParent(parent));

        true
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
    }
}

struct VstParent(*mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0 as *mut ::std::ffi::c_void,
            ..MacOSHandle::empty()
        })
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }
}
