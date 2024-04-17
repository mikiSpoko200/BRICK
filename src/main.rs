#[allow(dead_code)]

mod bitmap;
mod graphics;

// External imports
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::Graphics::Gdi as Gdi,
    Win32::System::LibraryLoader::{ GetModuleHandleA },
};

// Internal imports
use bitmap::{ Bitmap, Pixel };

// static mut naming convention: https://github.com/rust-lang/rust/pull/37162
static mut GLOBAL_BITMAP : Option<Bitmap> = None;
static mut ITER_COUNT: u8 = 0;


extern "system" fn window_procedure(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message as u32 {
            WM_QUIT => {
                //println!("WM_QUIT!");
                LRESULT(0)
            },
            WM_DESTROY => {
                //println!("WM_DESTROY!");
                PostQuitMessage(0);
                LRESULT(0)
            },
            WM_CLOSE => {
                //println!("WM_CLOSE!");
                PostQuitMessage(0);
                LRESULT(0)
            },
            WM_PAINT => {
                //println!("WM_PAINT!");
                let mut paint_struct = Gdi::PAINTSTRUCT::default();
                let device_context = Gdi::BeginPaint(window, &mut paint_struct);
                let mut rect = RECT::default();
                GetClientRect(window, &mut rect);
                GLOBAL_BITMAP.as_mut().unwrap().blit(&device_context, &rect);
                Gdi::EndPaint(window, &paint_struct);
                LRESULT(0)
            },
            WM_SIZE => {
                //println!("WM_SIZE!");

                LRESULT(0)
            }
            _ => { DefWindowProcA(window, message, wparam, lparam) }
        }
    }
}

/// # Safety
///
/// This function uses global static buffer GLOBAL_BITMAP
pub unsafe fn render_wierd_gradient(blue_offset: u8, green_offset: u8) {
    let bitmap = GLOBAL_BITMAP.as_ref().unwrap();
    for y in 0..bitmap.height {
        for x in 0..bitmap.width {
            let blue = (x as u8).overflowing_add(blue_offset).0;
            let green = (y as u8).overflowing_add(green_offset).0;
            GLOBAL_BITMAP.as_mut().unwrap().set_pixel(x as usize, y as usize, Pixel::new(0, green, blue));
        }
    }
}


fn main() -> Result<()> {
    let window_style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_VISIBLE;
    unsafe {
        let instance = GetModuleHandleA(None);
        assert!(!instance.is_invalid(), "Invalid instance handle.");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW),
            hInstance: instance,
            lpszClassName: PSTR(b"3DEngine\0".as_ptr() as _),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_procedure),
            cbWndExtra: 8,
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        assert_ne!(atom, 0, "Window class registration failed.");

        // region make client area 960 x 540
        let mut desired_client_rect_size = RECT {
            left: 300,
            right: 300 + graphics::WIDTH,
            top: 300,
            bottom: 300 + graphics::HEIGHT,
        };

        AdjustWindowRectEx(&mut desired_client_rect_size, window_style, false, WINDOW_EX_STYLE::default());

        // endregion

        let handle = CreateWindowExA(
                Default::default(),
                "3DEngine",
                "3DEngine",
                window_style, // WS_OVERLAPPEDWINDOW | WS_VISIBLE
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                desired_client_rect_size.right - desired_client_rect_size.left,
                desired_client_rect_size.bottom - desired_client_rect_size.top,
                None,
                None,
                instance,
                std::ptr::null_mut(),
            );

        let mut running = true;
        let mut message = MSG::default();

        let mut mesh = graphics::Mesh::default();
        GLOBAL_BITMAP = Some(Bitmap::default());


        while running {
            while PeekMessageA(&mut message, HWND(0), 0, 0, PM_REMOVE).into() {
                if message.message == WM_QUIT {
                    running = false;
                }
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
            // render_wierd_gradient(ITER_COUNT, ITER_COUNT.overflowing_mul(2).0);
            let device_context = Gdi::GetDC(handle);
            let mut rect = RECT::default();
            GetClientRect(handle, &mut rect);

            GLOBAL_BITMAP.as_mut().unwrap().clear_buffer();
            mesh.update(GLOBAL_BITMAP.as_mut().unwrap());


            GLOBAL_BITMAP.as_mut().unwrap().blit(&device_context, &rect);
            Gdi::ReleaseDC(handle, device_context);
            ITER_COUNT = ITER_COUNT.overflowing_add(2).0;
        }
        Ok(())
    }
}
