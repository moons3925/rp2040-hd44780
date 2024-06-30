#![no_std]
#![no_main]

use fugit::RateExtU32;
use hal::pac;
use hal::uart::{DataBits, StopBits, UartConfig};
use rp2040_hal::spi::Enabled;
use rp2040_hal::Clock;
use rp_pico::entry;

use panic_halt as _;
use rp2040_hal as hal;

use rp2040_lib::my_macro::UART_TRANSMITTER;
use rp2040_lib::print;
use rp2040_lib::println;

use rp2040_lib::bme280::spi::BME280;

use rp2040_hal::gpio::bank0::Gpio4;
use rp2040_hal::gpio::bank0::Gpio5;
use rp2040_hal::gpio::bank0::Gpio6;
use rp2040_hal::gpio::bank0::Gpio7;
use rp2040_hal::gpio::FunctionSio;
use rp2040_hal::gpio::Pin;
use rp2040_hal::gpio::PullDown;
use rp2040_hal::gpio::{FunctionSpi, SioOutput};

use crate::pac::SPI0;
use rp2040_hal::Spi;

use hd44780_driver::bus::FourBitBus;
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};

use rp2040_hal::gpio::bank0::Gpio20;
use rp2040_hal::gpio::bank0::Gpio21;
use rp2040_hal::gpio::bank0::Gpio22;
use rp2040_hal::gpio::bank0::Gpio26;
use rp2040_hal::gpio::bank0::Gpio27;
use rp2040_hal::gpio::bank0::Gpio28;

type Lcd = HD44780<
    FourBitBus<
        Pin<Gpio28, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio27, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio26, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio22, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio21, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio20, FunctionSio<SioOutput>, PullDown>,
    >,
>;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::delay::DelayUs;

fn lcd_display<D: DelayUs<u16> + DelayMs<u8>>(
    lcd: &mut Lcd,
    delay: &mut D,
    tup: (&f64, &f64, &f64),
) {
    let _ = lcd.set_cursor_pos(0x00, delay);
    let _ = lcd.write_str("Temp : ", delay);

    let tens_digit: u8 = ((*tup.0 as i32 / 10) % 10) as u8 | b'0';
    let ones_digit: u8 = ((*tup.0 as i32) % 10) as u8 | b'0';
    let tenths_digit: u8 = (((*tup.0 * 10.0) as i32) % 10) as u8 | b'0';

    let _ = lcd.write_char(b' ' as char, delay);
    let _ = lcd.write_char(b' ' as char, delay);
    let _ = lcd.write_char(tens_digit as char, delay);
    let _ = lcd.write_char(ones_digit as char, delay);
    let _ = lcd.write_char(b'.' as char, delay);
    let _ = lcd.write_char(tenths_digit as char, delay);
    let _ = lcd.write_char(b' ' as char, delay);
    let _ = lcd.write_char(0xdf as char, delay);
    let _ = lcd.write_char(b'C' as char, delay);

    let _ = lcd.set_cursor_pos(0x40, delay);
    let _ = lcd.write_str("Humi : ", delay);

    let tens_digit: u8 = ((*tup.1 as i32 / 10) % 10) as u8 | b'0';
    let ones_digit: u8 = ((*tup.1 as i32) % 10) as u8 | b'0';
    let tenths_digit: u8 = (((*tup.1 * 10.0) as i32) % 10) as u8 | b'0';

    let _ = lcd.write_char(b' ' as char, delay);
    let _ = lcd.write_char(b' ' as char, delay);
    let _ = lcd.write_char(tens_digit as char, delay);
    let _ = lcd.write_char(ones_digit as char, delay);
    let _ = lcd.write_char(b'.' as char, delay);
    let _ = lcd.write_char(tenths_digit as char, delay);
    let _ = lcd.write_char(b' ' as char, delay);
    let _ = lcd.write_char(b'%' as char, delay);

    let _ = lcd.set_cursor_pos(0x14, delay);
    let _ = lcd.write_str("Pres : ", delay);

    let thousands_digit: u8 = ((*tup.2 as i32 / 1000) % 10) as u8 | b'0';
    let hundreds_digit: u8 = ((*tup.2 as i32 / 100) % 10) as u8 | b'0';
    let tens_digit: u8 = ((*tup.2 as i32 / 10) % 10) as u8 | b'0';
    let ones_digit: u8 = ((*tup.2 as i32) % 10) as u8 | b'0';
    let tenths_digit: u8 = (((*tup.2 * 10.0) as i32) % 10) as u8 | b'0';

    let _ = lcd.write_char(thousands_digit as char, delay);
    let _ = lcd.write_char(hundreds_digit as char, delay);
    let _ = lcd.write_char(tens_digit as char, delay);
    let _ = lcd.write_char(ones_digit as char, delay);
    let _ = lcd.write_char(b'.' as char, delay);
    let _ = lcd.write_char(tenths_digit as char, delay);
    let _ = lcd.write_char(b' ' as char, delay);
    let _ = lcd.write_char(b'h' as char, delay);
    let _ = lcd.write_char(b'P' as char, delay);
    let _ = lcd.write_char(b'a' as char, delay);
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (pins.gpio0.reconfigure(), pins.gpio1.reconfigure());
    let uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let (_, uart_tx) = uart.split();

    critical_section::with(|_| unsafe {
        UART_TRANSMITTER = Some(uart_tx);
    });

    let spi_mosi = pins.gpio7.into_function::<hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio4.into_function::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.gpio6.into_function::<hal::gpio::FunctionSpi>();
    let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));

    let cs = pins.gpio5.into_push_pull_output();

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        10.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let mut bme280 = BME280::<
        Spi<
            Enabled,
            SPI0,
            (
                Pin<Gpio7, FunctionSpi, PullDown>,
                Pin<Gpio4, FunctionSpi, PullDown>,
                Pin<Gpio6, FunctionSpi, PullDown>,
            ),
        >,
        Pin<Gpio5, FunctionSio<SioOutput>, PullDown>,
    >::new(spi, cs);

    // LCD Display

    let rs = pins.gpio28.into_push_pull_output();
    let en = pins.gpio27.into_push_pull_output();
    let d4 = pins.gpio26.into_push_pull_output();
    let d5 = pins.gpio22.into_push_pull_output();
    let d6 = pins.gpio21.into_push_pull_output();
    let d7 = pins.gpio20.into_push_pull_output();

    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut delay).unwrap();

    let _ = lcd.reset(&mut delay);
    let _ = lcd.clear(&mut delay);
    let _ = lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        },
        &mut delay,
    );

    // DeviceのIDコード(0x60)を正しく読めれば成功としている
    if bme280.init() {
        println!("BME280 initialization successful.");
        println!("BME280 ID = 0x60.\r\n");
    } else {
        println!("BME280 initialization failed.\r\n");
    }

    loop {
        bme280.read_data();

        let (temp, humi, pres) = bme280.get_elements();

        println!("T = {:.2} ℃", temp);
        println!("H = {:.2} %", humi);
        println!("P = {:.2} hPa\r\n", pres);

        lcd_display(&mut lcd, &mut delay, (&temp, &humi, &pres));
    }
}
