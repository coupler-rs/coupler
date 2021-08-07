use crate::{Rect, WindowOptions};

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use cocoa::{appkit, base, foundation};
use objc::{class, msg_send, sel, sel_impl};
use objc::{declare, runtime};
use raw_window_handle::{macos::MacOSHandle, HasRawWindowHandle, RawWindowHandle};

#[derive(Debug)]
pub enum ApplicationError {
    ViewClassRegistration,
    GetEvent,
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ApplicationError {}

#[derive(Clone)]
pub struct Application {
    inner: Rc<ApplicationInner>,
}

struct ApplicationInner {
    open: Cell<bool>,
    running: Cell<usize>,
    class: *mut runtime::Class,
}

impl Application {
    pub fn open() -> Result<Application, ApplicationError> {
        unsafe {
            let class_name = format!("window-{}", uuid::Uuid::new_v4().to_simple());

            let mut class_decl =
                if let Some(class_decl) = declare::ClassDecl::new(&class_name, class!(NSView)) {
                    class_decl
                } else {
                    return Err(ApplicationError::ViewClassRegistration);
                };

            let class = class_decl.register();

            Ok(Application {
                inner: Rc::new(ApplicationInner {
                    open: Cell::new(true),
                    running: Cell::new(0),
                    class: class as *const runtime::Class as *mut runtime::Class,
                }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.inner.running.set(0);
                self.inner.open.set(false);

                runtime::objc_disposeClassPair(self.inner.class);
            }

            Ok(())
        }
    }

    pub fn start(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                let depth = self.inner.running.get();
                self.inner.running.set(depth + 1);

                let app = appkit::NSApp();
                let until_date = msg_send![class!(NSDate), distantFuture];
                while self.inner.open.get() && self.inner.running.get() > depth {
                    let pool = foundation::NSAutoreleasePool::new(base::nil);

                    let event =
                        appkit::NSApplication::nextEventMatchingMask_untilDate_inMode_dequeue_(
                            app,
                            appkit::NSEventMask::NSAnyEventMask.bits(),
                            until_date,
                            foundation::NSDefaultRunLoopMode,
                            base::YES,
                        );

                    if event.is_null() {
                        self.inner.running.set(depth);
                        let () = msg_send![pool, drain];
                        return Err(ApplicationError::GetEvent);
                    } else {
                        appkit::NSApplication::sendEvent_(app, event);
                    }

                    let () = msg_send![pool, drain];
                }
            }

            Ok(())
        }
    }

    pub fn stop(&self) {
        unsafe {
            if self.inner.open.get() {
                self.inner.running.set(self.inner.running.get().saturating_sub(1));
            }
        }
    }

    pub fn poll(&self) {
        unsafe {
            let app = appkit::NSApp();
            let until_date = msg_send![class!(NSDate), now];
            while self.inner.open.get() {
                let pool = foundation::NSAutoreleasePool::new(base::nil);

                let event = appkit::NSApplication::nextEventMatchingMask_untilDate_inMode_dequeue_(
                    app,
                    appkit::NSEventMask::NSAnyEventMask.bits(),
                    until_date,
                    foundation::NSDefaultRunLoopMode,
                    base::YES,
                );

                if event.is_null() {
                    let () = msg_send![pool, drain];
                    break;
                } else {
                    appkit::NSApplication::sendEvent_(app, event);
                }

                let () = msg_send![pool, drain];
            }
        }
    }

    pub fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        None
    }
}

#[derive(Debug)]
pub enum WindowError {
    ApplicationClosed,
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WindowError {}

#[derive(Clone)]
pub struct Window {
    state: Rc<WindowState>,
}

struct WindowState {
    open: Cell<bool>,
    window: base::id,
    view: base::id,
    application: crate::Application,
}

impl Window {
    pub fn open(
        application: &crate::Application,
        options: WindowOptions,
    ) -> Result<crate::Window, WindowError> {
        unsafe {
            if !application.application.inner.open.get() {
                return Err(WindowError::ApplicationClosed);
            }

            let pool = foundation::NSAutoreleasePool::new(base::nil);

            let style_mask = appkit::NSWindowStyleMask::NSTitledWindowMask
                | appkit::NSWindowStyleMask::NSClosableWindowMask
                | appkit::NSWindowStyleMask::NSMiniaturizableWindowMask
                | appkit::NSWindowStyleMask::NSResizableWindowMask;

            let window: base::id = msg_send![class!(NSWindow), alloc];
            appkit::NSWindow::initWithContentRect_styleMask_backing_defer_(
                window,
                foundation::NSRect::new(
                    foundation::NSPoint::new(options.rect.x, options.rect.y),
                    foundation::NSSize::new(options.rect.width, options.rect.height),
                ),
                style_mask,
                appkit::NSBackingStoreBuffered,
                base::NO,
            );

            let view: base::id = msg_send![application.application.inner.class, alloc];
            appkit::NSView::initWithFrame_(
                view,
                foundation::NSRect::new(
                    foundation::NSPoint::new(0.0, 0.0),
                    foundation::NSSize::new(options.rect.width, options.rect.height),
                ),
            );

            appkit::NSWindow::setContentView_(window, view);
            appkit::NSWindow::center(window);
            appkit::NSWindow::makeKeyAndOrderFront_(window, base::nil);

            let () = msg_send![view, release];

            let () = msg_send![pool, drain];

            let state = Rc::new(WindowState {
                open: Cell::new(true),
                window: window,
                view,
                application: application.clone(),
            });

            Ok(crate::Window { window: Window { state }, phantom: PhantomData })
        }
    }

    pub fn request_display(&self) {}

    pub fn request_display_rect(&self, rect: Rect) {}

    pub fn update_contents(&self, framebuffer: &[u32], width: usize, height: usize) {}

    pub fn close(&self) -> Result<(), WindowError> {
        unsafe {
            if self.state.open.get() {
                let pool = foundation::NSAutoreleasePool::new(base::nil);

                appkit::NSWindow::close(self.state.window);

                let () = msg_send![pool, drain];
            }

            Ok(())
        }
    }

    pub fn application(&self) -> &crate::Application {
        &self.state.application
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::MacOS(MacOSHandle { ..MacOSHandle::empty() })
    }
}
