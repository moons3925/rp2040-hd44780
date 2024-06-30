pub mod i2c;
pub mod spi;

const OSRS_H: u8 = 0x3; // 湿度 x4 サンプリング
const CTRL_HUM_WDATA: u8 = OSRS_H;

const OSRS_T: u8 = 0x3; // 温度 x4 サンプリング
const OSRS_P: u8 = 0x3; // 気圧 x4 サンプリング
const MODE: u8 = 0x3; // ノーマルモード
const CTRL_MEAS_WDATA: u8 = OSRS_T << 5 | OSRS_P << 2 | MODE;

const T_SB: u8 = 0x4; // スタンバイ時間 500msec
const FILTER: u8 = 0x2; // フィルター 4
const SPI3W_EN: u8 = 0; // SPI3Wire enable = desable
const CONFIG_WDATA: u8 = T_SB << 5 | FILTER << 2 | SPI3W_EN;

const CTRL_HUM_REG: u8 = 0xf2;
const CTRL_MEAS_REG: u8 = 0xf4;
const CONFIG_REG: u8 = 0xf5;
const ID_REG: u8 = 0xd0;
const ID_CODE: u8 = 0x60;

pub const DEVICE_ADDRESS: u8 = 0x76;

const CALIBRATION_OFFSET_T_P: u8 = 0x88;
const CALIBRATION_OFFSET_H1: u8 = 0xa1;
const CALIBRATION_OFFSET_H2: u8 = 0xe1;
const PRESS_MSB_REG: u8 = 0xf7;

trait Interface {
    fn read_register(&mut self, register: u8) -> u8;
    fn write_register(&mut self, register: u8, value: u8);
    fn write(&mut self, value: &[u8]);
    fn read_trim(&mut self, buffer: &mut [u8; 32]);
    fn read_data(&mut self, buffer: &mut [u8; 8]);
}
