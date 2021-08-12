use crate::{Cursor, MouseButton, Parent, Point, Rect, WindowHandler, WindowOptions};

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
                sel!(acceptsFirstMouse:),
                accepts_first_mouse
                    as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id) -> base::BOOL,
            );
            class_decl.add_method(
                sel!(isFlipped),
                is_flipped
                    as extern "C" fn(_this: &mut runtime::Object, _: runtime::Sel) -> base::BOOL,
            );
            class_decl.add_method(
                sel!(mouseMoved:),
                mouse_moved as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(mouseDragged:),
                mouse_moved as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(rightMouseDragged:),
                mouse_moved as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(otherMouseDragged:),
                mouse_moved as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(mouseDown:),
                mouse_down as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(mouseUp:),
                mouse_up as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(rightMouseDown:),
                right_mouse_down as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(rightMouseUp:),
                right_mouse_up as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(otherMouseDown:),
                other_mouse_down as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(otherMouseUp:),
                other_mouse_up as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(scrollWheel:),
                scroll_wheel as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id),
            );
            class_decl.add_method(
                sel!(windowShouldClose:),
                window_should_close
                    as extern "C" fn(&mut runtime::Object, runtime::Sel, base::id) -> base::BOOL,
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
                    let window = Window::from_ns_view(ns_view);

                    if let Err(error) = window.window.close() {
                        window_errors.push(error);
                        continue;
                    }

                    if window.window.state.open.get() {
                        window_errors.push(WindowError::CouldNotClose);
                    }
                }

                if !window_errors.is_empty() {
                    return Err(ApplicationError::Close(window_errors));
                }

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
    CouldNotClose,
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

            #[allow(non_upper_case_globals)]
            let tracking_options = {
                const NSTrackingMouseEnteredAndExited: foundation::NSUInteger = 0x1;
                const NSTrackingMouseMoved: foundation::NSUInteger = 0x2;
                const NSTrackingActiveAlways: foundation::NSUInteger = 0x80;
                const NSTrackingInVisibleRect: foundation::NSUInteger = 0x200;
                const NSTrackingEnabledDuringMouseDrag: foundation::NSUInteger = 0x400;

                NSTrackingMouseEnteredAndExited
                    | NSTrackingMouseMoved
                    | NSTrackingActiveAlways
                    | NSTrackingInVisibleRect
                    | NSTrackingEnabledDuringMouseDrag
            };

            let tracking_area: base::id = msg_send![class!(NSTrackingArea), alloc];
            let tracking_area: base::id = msg_send![
                tracking_area,
                initWithRect: foundation::NSRect::new(
                    foundation::NSPoint::new(0.0, 0.0),
                    foundation::NSSize::new(0.0, 0.0),
                )
                options: tracking_options
                owner: ns_view
                userInfo: base::nil
            ];
            let () = msg_send![ns_view, addTrackingArea: tracking_area];
            let () = msg_send![tracking_area, autorelease];

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
                runloop::kCFRunLoopCommonModes,
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
                appkit::NSWindow::setDelegate_(ns_window, ns_view);
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
                    for row in (0..copy_height).rev() {
                        let src = &framebuffer[row * width..row * width + copy_width];
                        let dst =
                            &mut back_buffer[row * copy_width..row * copy_width + copy_width];
                        dst.copy_from_slice(src);
                    }

                    let data = slice::from_raw_parts(
                        back_buffer.as_ptr() as *const u8,
                        4 * copy_width * copy_height,
                    );
                    let data_provider = CGDataProvider::from_slice(data);

                    let image = CGImage::new(
                        copy_width,
                        copy_height,
                        8,
                        32,
                        4 * copy_width,
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

    pub fn set_cursor(&self, _cursor: Cursor) {}

    pub fn set_mouse_position(&self, position: Point) {
        use core_graphics::display::{
            CGAssociateMouseAndMouseCursorPosition, CGWarpMouseCursorPosition,
        };
        use core_graphics::geometry::CGPoint;

        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGSetLocalEventsSuppressionInterval(seconds: f64) -> core_graphics::base::CGError;
        }

        unsafe {
            if self.state.open.get() {
                let window: base::id = msg_send![self.state.ns_view, window];
                if window.is_null() {
                    return;
                }

                let point = foundation::NSPoint::new(position.x, position.y);
                let point: foundation::NSPoint =
                    msg_send![self.state.ns_view, convertPoint: point toView: base::nil];
                let point: foundation::NSPoint = msg_send![window, convertPointToScreen: point];

                let screens: base::id = msg_send![class!(NSScreen), screens];
                let primary_screen = foundation::NSArray::objectAtIndex(screens, 0);
                let screen_height = appkit::NSScreen::frame(primary_screen).size.height;

                CGWarpMouseCursorPosition(CGPoint::new(point.x, screen_height - point.y));
                CGAssociateMouseAndMouseCursorPosition(0);
                CGSetLocalEventsSuppressionInterval(0.0);
            }
        }
    }

    pub fn close(&self) -> Result<(), WindowError> {
        unsafe {
            if self.state.open.get() {
                let pool = foundation::NSAutoreleasePool::new(base::nil);

                appkit::NSView::removeFromSuperview(self.state.ns_view);

                let () = msg_send![pool, drain];
            }

            Ok(())
        }
    }

    pub fn application(&self) -> &crate::Application {
        &self.state.application
    }

    unsafe fn from_ns_view(ns_view: base::id) -> crate::Window {
        let state_ptr =
            *runtime::Object::get_ivar::<*mut c_void>(&*ns_view, WINDOW_STATE) as *mut WindowState;
        let state = Rc::from_raw(state_ptr);
        let _ = Rc::into_raw(state.clone());
        crate::Window { window: Window { state }, phantom: PhantomData }
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
        let window = Window::from_ns_view(info as base::id);
        window.window.state.handler.frame(&window);
    }
}

extern "C" fn draw_rect(this: &mut runtime::Object, _: runtime::Sel, _rect: foundation::NSRect) {
    unsafe {
        let window = Window::from_ns_view(this);
        window.window.state.handler.display(&window);
    }
}

extern "C" fn accepts_first_mouse(
    _this: &mut runtime::Object,
    _: runtime::Sel,
    _event: base::id,
) -> base::BOOL {
    base::YES
}

extern "C" fn is_flipped(_this: &mut runtime::Object, _: runtime::Sel) -> base::BOOL {
    base::YES
}

extern "C" fn mouse_moved(this: &mut runtime::Object, _: runtime::Sel, event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        let point = appkit::NSEvent::locationInWindow(event);
        let point = appkit::NSView::convertPoint_fromView_(this as base::id, point, base::nil);
        let point = Point { x: point.x, y: point.y };
        window.window.state.handler.mouse_move(&window, point);
    }
}

extern "C" fn mouse_down(this: &mut runtime::Object, _: runtime::Sel, _event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        window.window.state.handler.mouse_down(&window, MouseButton::Left);
    }
}

extern "C" fn mouse_up(this: &mut runtime::Object, _: runtime::Sel, _event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        window.window.state.handler.mouse_up(&window, MouseButton::Left);
    }
}

extern "C" fn right_mouse_down(this: &mut runtime::Object, _: runtime::Sel, _event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        window.window.state.handler.mouse_down(&window, MouseButton::Right);
    }
}

extern "C" fn right_mouse_up(this: &mut runtime::Object, _: runtime::Sel, _event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        window.window.state.handler.mouse_up(&window, MouseButton::Right);
    }
}

fn mouse_button_from_number(button_number: foundation::NSInteger) -> Option<MouseButton> {
    match button_number {
        0 => Some(MouseButton::Left),
        1 => Some(MouseButton::Right),
        2 => Some(MouseButton::Middle),
        3 => Some(MouseButton::Back),
        4 => Some(MouseButton::Forward),
        _ => None,
    }
}

extern "C" fn other_mouse_down(this: &mut runtime::Object, _: runtime::Sel, event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        if let Some(mouse_button) = mouse_button_from_number(appkit::NSEvent::buttonNumber(event)) {
            window.window.state.handler.mouse_down(&window, mouse_button);
        }
    }
}

extern "C" fn other_mouse_up(this: &mut runtime::Object, _: runtime::Sel, event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        if let Some(mouse_button) = mouse_button_from_number(appkit::NSEvent::buttonNumber(event)) {
            window.window.state.handler.mouse_down(&window, mouse_button);
        }
    }
}

extern "C" fn scroll_wheel(this: &mut runtime::Object, _: runtime::Sel, event: base::id) {
    unsafe {
        let window = Window::from_ns_view(this);
        let dx = appkit::NSEvent::scrollingDeltaX(event);
        let dy = appkit::NSEvent::scrollingDeltaY(event);
        let (dx, dy) = if appkit::NSEvent::hasPreciseScrollingDeltas(event) == base::YES {
            (dx, dy)
        } else {
            (32.0 * dx, 32.0 * dy)
        };
        window.window.state.handler.scroll(&window, dx, dy);
    }
}

extern "C" fn window_should_close(
    this: &mut runtime::Object,
    _: runtime::Sel,
    _sender: base::id,
) -> base::BOOL {
    unsafe {
        let window = Window::from_ns_view(this);
        window.window.state.handler.request_close(&window);

        base::NO
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

        if let Some(ns_window) = window.window.state.ns_window {
            appkit::NSWindow::close(ns_window);
        }

        window.window.state.open.set(false);
        let ns_view = window.window.state.ns_view;
        window.application().application.inner.windows.borrow_mut().remove(&ns_view);
        window.window.state.handler.destroy(&window);
        drop(window);

        let superclass = msg_send![this, superclass];
        let () = msg_send![super(this, superclass), dealloc];
    }
}
