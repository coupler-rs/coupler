use crate::{Parent, Rect, WindowHandler, WindowOptions};

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::error::Error;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::rc::Rc;
use std::{fmt, mem, ptr, slice};

use cocoa::{appkit, base, foundation};
use core_foundation::{date, runloop};
use objc::{class, msg_send, sel, sel_impl};
use objc::{declare, runtime};
use raw_window_handle::{macos::MacOSHandle, HasRawWindowHandle, RawWindowHandle};

const TIMER_INTERVAL: f64 = 1.0 / 60.0;

const WINDOW_STATE: &str = "windowState";

#[derive(Debug)]
pub enum ApplicationError {
    ViewClassRegistration,
    GetEvent,
    Close(Vec<WindowError>),
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
    windows: RefCell<HashSet<base::id>>,
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

            class_decl.add_ivar::<*mut c_void>(WINDOW_STATE);

            class_decl.add_method(
                sel!(drawRect:),
                draw_rect as extern "C" fn(&mut runtime::Object, runtime::Sel, foundation::NSRect),
            );
            class_decl.add_method(
                sel!(dealloc),
                dealloc as extern "C" fn(&mut runtime::Object, runtime::Sel),
            );

            let class = class_decl.register();

            Ok(Application {
                inner: Rc::new(ApplicationInner {
                    open: Cell::new(true),
                    running: Cell::new(0),
                    class: class as *const runtime::Class as *mut runtime::Class,
                    windows: RefCell::new(HashSet::new()),
                }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.inner.running.set(0);
                self.inner.open.set(false);

                let mut window_errors = Vec::new();
                for ns_view in self.inner.windows.take() {
                    let state_ptr =
                        *runtime::Object::get_ivar::<*mut c_void>(&*ns_view, WINDOW_STATE)
                            as *mut WindowState;
                    let state = Rc::from_raw(state_ptr);
                    let _ = Rc::into_raw(state.clone());
                    let window = crate::Window { window: Window { state }, phantom: PhantomData };
                    if let Err(error) = window.window.close() {
                        window_errors.push(error);
                    }
                }

                runtime::objc_disposeClassPair(self.inner.class);

                if !window_errors.is_empty() {
                    return Err(ApplicationError::Close(window_errors));
                }
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
        if self.inner.open.get() {
            self.inner.running.set(self.inner.running.get().saturating_sub(1));
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
    InvalidWindowHandle,
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
    ns_window: Option<base::id>,
    ns_view: base::id,
    rect: Cell<Rect>,
    back_buffer: RefCell<Vec<u32>>,
    timer: runloop::CFRunLoopTimerRef,
    application: crate::Application,
    handler: Box<dyn WindowHandler>,
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

            let ns_window = if let Parent::None = options.parent {
                let style_mask = appkit::NSWindowStyleMask::NSTitledWindowMask
                    | appkit::NSWindowStyleMask::NSClosableWindowMask
                    | appkit::NSWindowStyleMask::NSMiniaturizableWindowMask
                    | appkit::NSWindowStyleMask::NSResizableWindowMask;

                let ns_window: base::id = msg_send![class!(NSWindow), alloc];
                let ns_window = appkit::NSWindow::initWithContentRect_styleMask_backing_defer_(
                    ns_window,
                    foundation::NSRect::new(
                        foundation::NSPoint::new(options.rect.x, options.rect.y),
                        foundation::NSSize::new(options.rect.width, options.rect.height),
                    ),
                    style_mask,
                    appkit::NSBackingStoreBuffered,
                    base::NO,
                );

                let title = foundation::NSString::init_str(
                    foundation::NSString::alloc(base::nil),
                    &options.title,
                );
                let () = msg_send![title, autorelease];
                appkit::NSWindow::setTitle_(ns_window, title);

                Some(ns_window)
            } else {
                None
            };

            let ns_view: base::id = msg_send![application.application.inner.class, alloc];
            let ns_view = appkit::NSView::initWithFrame_(
                ns_view,
                foundation::NSRect::new(
                    foundation::NSPoint::new(0.0, 0.0),
                    foundation::NSSize::new(options.rect.width, options.rect.height),
                ),
            );

            let timer = runloop::CFRunLoopTimerCreate(
                ptr::null(),
                date::CFAbsoluteTimeGetCurrent() + TIMER_INTERVAL,
                TIMER_INTERVAL,
                0,
                0,
                frame,
                &mut runloop::CFRunLoopTimerContext {
                    info: ns_view as *mut c_void,
                    ..mem::zeroed()
                },
            );
            runloop::CFRunLoopAddTimer(
                runloop::CFRunLoopGetCurrent(),
                timer,
                runloop::kCFRunLoopDefaultMode,
            );

            let back_buffer_size = options.rect.width as usize * options.rect.height as usize;
            let back_buffer = RefCell::new(vec![0xFF000000; back_buffer_size]);

            let state = Rc::new(WindowState {
                open: Cell::new(true),
                ns_window,
                ns_view,
                rect: Cell::new(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: options.rect.width,
                    height: options.rect.height,
                }),
                back_buffer,
                timer,
                application: application.clone(),
                handler: options.handler,
            });

            runtime::Object::set_ivar::<*mut c_void>(
                &mut *ns_view,
                WINDOW_STATE,
                Rc::into_raw(state.clone()) as *mut c_void,
            );

            application.application.inner.windows.borrow_mut().insert(ns_view);
            let window = crate::Window { window: Window { state }, phantom: PhantomData };
            window.window.state.handler.create(&window);

            if let Some(ns_window) = window.window.state.ns_window {
                appkit::NSWindow::setContentView_(ns_window, ns_view);
                let () = msg_send![ns_view, release];

                appkit::NSWindow::center(ns_window);
                appkit::NSWindow::makeKeyAndOrderFront_(ns_window, base::nil);
            } else if let Parent::Parent(parent) = options.parent {
                match parent.raw_window_handle() {
                    RawWindowHandle::MacOS(handle) => {
                        if handle.ns_view.is_null() {
                            return Err(WindowError::InvalidWindowHandle);
                        }

                        appkit::NSView::addSubview_(handle.ns_view as base::id, ns_view);
                        let () = msg_send![ns_view, release];
                    }
                    _ => {
                        return Err(WindowError::InvalidWindowHandle);
                    }
                }
            }

            let () = msg_send![pool, drain];

            Ok(window)
        }
    }

    pub fn request_display(&self) {
        unsafe {
            if self.state.open.get() {
                let () = msg_send![self.state.ns_view, setNeedsDisplay: base::YES];
            }
        }
    }

    pub fn request_display_rect(&self, rect: Rect) {
        unsafe {
            if self.state.open.get() {
                let () = msg_send![self.state.ns_view, setNeedsDisplayInRect: foundation::NSRect::new(
                    foundation::NSPoint::new(rect.x, rect.y),
                    foundation::NSSize::new(rect.width, rect.height),
                )];
            }
        }
    }

    pub fn update_contents(&self, framebuffer: &[u32], width: usize, height: usize) {
        use core_graphics::base::{
            kCGBitmapByteOrder32Host, kCGImageAlphaPremultipliedFirst, kCGRenderingIntentDefault,
        };
        use core_graphics::context::{CGBlendMode, CGContext};
        use core_graphics::geometry::{CGPoint, CGRect, CGSize};
        use core_graphics::{
            color_space::CGColorSpace, data_provider::CGDataProvider, image::CGImage,
        };

        unsafe {
            if self.state.open.get() {
                let width = width.min(framebuffer.len());
                let height = height.min(framebuffer.len() / width);

                let current_context: base::id =
                    msg_send![class!(NSGraphicsContext), currentContext];

                if !current_context.is_null() {
                    let context_ptr: *mut core_graphics::sys::CGContext =
                        msg_send![current_context, CGContext];
                    let context = CGContext::from_existing_context_ptr(context_ptr);

                    let color_space = CGColorSpace::create_device_rgb();

                    let window_width = self.state.rect.get().width as usize;
                    let window_height = self.state.rect.get().height as usize;
                    let copy_width = width.min(window_width as usize);
                    let copy_height = height.min(window_height as usize);
                    let mut back_buffer = self.state.back_buffer.borrow_mut();
                    for row in 0..copy_height {
                        let src = &framebuffer[row * width..row * width + copy_width];
                        let dst =
                            &mut back_buffer[row * window_width..row * window_width + copy_width];
                        dst.copy_from_slice(src);
                    }

                    let data = slice::from_raw_parts(
                        back_buffer.as_ptr() as *const u8,
                        4 * width * height,
                    );
                    let data_provider = CGDataProvider::from_slice(data);

                    let image = CGImage::new(
                        width,
                        height,
                        8,
                        32,
                        4 * width,
                        &color_space,
                        kCGBitmapByteOrder32Host | kCGImageAlphaPremultipliedFirst,
                        &data_provider,
                        false,
                        kCGRenderingIntentDefault,
                    );

                    context.set_blend_mode(CGBlendMode::Copy);
                    context.draw_image(
                        CGRect::new(
                            &CGPoint::new(0.0, 0.0),
                            &CGSize::new(width as f64, height as f64),
                        ),
                        &image,
                    );
                }
            }
        }
    }

    pub fn close(&self) -> Result<(), WindowError> {
        unsafe {
            if self.state.open.get() {
                let pool = foundation::NSAutoreleasePool::new(base::nil);

                if let Some(ns_window) = self.state.ns_window {
                    appkit::NSWindow::close(ns_window);
                } else {
                    appkit::NSView::removeFromSuperview(self.state.ns_view);
                }

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
        if self.state.open.get() {
            RawWindowHandle::MacOS(MacOSHandle {
                ns_window: self.state.ns_window.unwrap_or(ptr::null_mut()) as *mut c_void,
                ns_view: self.state.ns_view as *mut c_void,
                ..MacOSHandle::empty()
            })
        } else {
            RawWindowHandle::MacOS(MacOSHandle::empty())
        }
    }
}

extern "C" fn frame(_timer: runloop::CFRunLoopTimerRef, info: *mut c_void) {
    unsafe {
        let state_ptr =
            *runtime::Object::get_ivar::<*mut c_void>(&*(info as base::id), WINDOW_STATE)
                as *mut WindowState;
        let state = Rc::from_raw(state_ptr);
        let _ = Rc::into_raw(state.clone());
        let window = crate::Window { window: Window { state }, phantom: PhantomData };

        window.window.state.handler.frame(&window);
    }
}

extern "C" fn draw_rect(this: &mut runtime::Object, _: runtime::Sel, _rect: foundation::NSRect) {
    unsafe {
        let state_ptr =
            *runtime::Object::get_ivar::<*mut c_void>(&*this, WINDOW_STATE) as *mut WindowState;
        let state = Rc::from_raw(state_ptr);
        let _ = Rc::into_raw(state.clone());
        let window = crate::Window { window: Window { state }, phantom: PhantomData };

        window.window.state.handler.display(&window);
    }
}

extern "C" fn dealloc(this: &mut runtime::Object, _: runtime::Sel) {
    unsafe {
        let state_ptr =
            *runtime::Object::get_ivar::<*mut c_void>(this, WINDOW_STATE) as *mut WindowState;
        runtime::Object::set_ivar::<*mut c_void>(this, WINDOW_STATE, ptr::null_mut());
        let state = Rc::from_raw(state_ptr);
        let window = crate::Window { window: Window { state }, phantom: PhantomData };

        runloop::CFRunLoopTimerInvalidate(window.window.state.timer);

        window.window.state.open.set(false);
        let ns_view = window.window.state.ns_view;
        window.application().application.inner.windows.borrow_mut().remove(&ns_view);
        window.window.state.handler.destroy(&window);
        drop(window);

        let superclass = msg_send![this, superclass];
        let () = msg_send![super(this, superclass), dealloc];
    }
}
