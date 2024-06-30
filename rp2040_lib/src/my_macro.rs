// println! , print! マクロを使うためのコード
// 出力先はUARTなので、main.rs等でUARTを初期化するコードが必要
// ペリフェラルを初期化後 UART_TRANSMITTER には slpit()で分割した uart_tx 等を some(T) を使って代入する

use core::fmt;
use core::fmt::Write;
use embedded_hal::prelude::_embedded_hal_serial_Write;
use rp2040_hal as hal;
use rp2040_hal::gpio::bank0::{Gpio0, Gpio1};
use rp2040_hal::pac;

pub static mut UART_TRANSMITTER: Option<hal::uart::Writer<pac::UART0, UartPins>> = None;

type UartPins = (
    hal::gpio::Pin<Gpio0, hal::gpio::FunctionUart, hal::gpio::PullNone>,
    hal::gpio::Pin<Gpio1, hal::gpio::FunctionUart, hal::gpio::PullNone>,
);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::my_macro::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn _print(args: fmt::Arguments) {
    let mut writer = UartWriter {};
    writer.write_fmt(args).unwrap();
}

struct UartWriter;

impl core::fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            write_byte(c);
        }
        Ok(())
    }
}

fn write_byte(c: u8) {
    unsafe {
        if let Some(ref mut writer) = UART_TRANSMITTER.as_mut() {
            let _ = nb::block!(writer.write(c));
        }
    }
}
