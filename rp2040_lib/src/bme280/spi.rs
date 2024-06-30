use super::Interface;

use embedded_hal::blocking::spi::transfer::Default;
use embedded_hal::blocking::spi::write::Default as OtherDefault;
use embedded_hal::digital::v2::OutputPin;

use embedded_hal::prelude::_embedded_hal_blocking_spi_Transfer;
use embedded_hal::prelude::_embedded_hal_blocking_spi_Write;

use crate::bme280::ID_CODE;
use crate::bme280::ID_REG;

use crate::bme280::CALIBRATION_OFFSET_H1;
use crate::bme280::CALIBRATION_OFFSET_H2;
use crate::bme280::CALIBRATION_OFFSET_T_P;
use crate::bme280::CONFIG_REG;
use crate::bme280::CONFIG_WDATA;
use crate::bme280::CTRL_HUM_REG;
use crate::bme280::CTRL_HUM_WDATA;
use crate::bme280::CTRL_MEAS_REG;
use crate::bme280::CTRL_MEAS_WDATA;
use crate::bme280::PRESS_MSB_REG;

pub struct BME280<IF, GPIO>
where
    IF: Default<u8> + OtherDefault<u8>,
    GPIO: OutputPin,
{
    interface: SPIInterface<IF, GPIO>,
    buffer: [u8; 32],
    buffer2: [u8; 8],
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
    dig_h1: i8,
    dig_h2: i16,
    dig_h3: i8,
    dig_h4: i16,
    dig_h5: i16,
    dig_h6: i8,
    pub temp_raw: u32,
    pub pres_raw: u32,
    pub humi_raw: u32,
    t_fine: i32,
}

impl<IF, GPIO> BME280<IF, GPIO>
where
    IF: Default<u8> + OtherDefault<u8>,
    GPIO: OutputPin,
{
    pub fn new(spi: IF, cs: GPIO) -> Self {
        Self {
            interface: SPIInterface { spi, cs },
            buffer: [0; 32],
            buffer2: [0; 8],
            dig_t1: 0,
            dig_t2: 0,
            dig_t3: 0,
            dig_p1: 0,
            dig_p2: 0,
            dig_p3: 0,
            dig_p4: 0,
            dig_p5: 0,
            dig_p6: 0,
            dig_p7: 0,
            dig_p8: 0,
            dig_p9: 0,
            dig_h1: 0,
            dig_h2: 0,
            dig_h3: 0,
            dig_h4: 0,
            dig_h5: 0,
            dig_h6: 0,
            temp_raw: 0,
            pres_raw: 0,
            humi_raw: 0,
            t_fine: 0,
        }
    }
    pub fn init(&mut self) -> bool {
        let _ = self.interface.cs.set_high(); // デバイスを非選択にしておく

        let id = self.interface.read_register(ID_REG);
        if id != ID_CODE {
            return false;
        }

        self.interface.write_register(CTRL_HUM_REG, CTRL_HUM_WDATA);
        self.interface
            .write_register(CTRL_MEAS_REG, CTRL_MEAS_WDATA);
        self.interface.write_register(CONFIG_REG, CONFIG_WDATA);
        self.read_trim();
        true
    }
    pub fn read_trim(&mut self) {
        let _ = self.interface.read_trim(&mut self.buffer);

        self.dig_t1 = (self.buffer[1] as u16) << 8 | self.buffer[0] as u16;
        self.dig_t2 = (self.buffer[3] as i16) << 8 | self.buffer[2] as i16;
        self.dig_t3 = (self.buffer[5] as i16) << 8 | self.buffer[4] as i16;

        self.dig_p1 = (self.buffer[7] as u16) << 8 | self.buffer[6] as u16;
        self.dig_p2 = (self.buffer[9] as i16) << 8 | self.buffer[8] as i16;
        self.dig_p3 = (self.buffer[11] as i16) << 8 | self.buffer[10] as i16;
        self.dig_p4 = (self.buffer[13] as i16) << 8 | self.buffer[12] as i16;
        self.dig_p5 = (self.buffer[15] as i16) << 8 | self.buffer[14] as i16;
        self.dig_p6 = (self.buffer[17] as i16) << 8 | self.buffer[16] as i16;
        self.dig_p7 = (self.buffer[19] as i16) << 8 | self.buffer[18] as i16;
        self.dig_p8 = (self.buffer[21] as i16) << 8 | self.buffer[20] as i16;
        self.dig_p9 = (self.buffer[23] as i16) << 8 | self.buffer[22] as i16;

        self.dig_h1 = self.buffer[24] as i8;
        self.dig_h2 = (self.buffer[26] as i16) << 8 | self.buffer[25] as i16;
        self.dig_h3 = self.buffer[27] as i8;

        self.dig_h4 = (self.buffer[28] as i16) << 4 | (self.buffer[29] as i16) & 0xf;
        self.dig_h5 = (self.buffer[29] as i16) << 4 | ((self.buffer[30] as i16) >> 4) & 0xf;
        self.dig_h6 = self.buffer[31] as i8;
    }
    pub fn read_data(&mut self) {
        let _ = self.interface.read_data(&mut self.buffer2);
        self.pres_raw = (self.buffer2[0] as u32) << 12
            | (self.buffer2[1] as u32) << 4
            | (self.buffer2[2] as u32) >> 4;
        self.temp_raw = (self.buffer2[3] as u32) << 12
            | (self.buffer2[4] as u32) << 4
            | (self.buffer2[5] as u32) >> 4;
        self.humi_raw = (self.buffer2[6] as u32) << 8 | self.buffer2[7] as u32;
    }
    pub fn read_register(&mut self, register: u8) -> u8 {
        self.interface.read_register(register)
    }

    pub fn calibration_temperature(&mut self, adc_t: i32) -> f64 {
        let var1: i32;
        let var2: i32;
        let t: i32;
        var1 = (((adc_t >> 3) - ((self.dig_t1 as i32) << 1)) * (self.dig_t2 as i32)) >> 11;
        var2 = ((adc_t >> 4) - (self.dig_t1 as i32) * ((adc_t >> 4) - (self.dig_t1 as i32)) >> 12)
            * ((self.dig_t3 as i32) >> 14);
        self.t_fine = var1 + var2;
        t = (self.t_fine * 5 + 128) >> 8;
        (t as f64) / 100.0
    }

    pub fn calibration_humidity(&mut self, adc_h: i32) -> f64 {
        let mut v_x1_u32r: i32;
        v_x1_u32r = self.t_fine - 76800i32;
        v_x1_u32r = (((adc_h << 14)
            - (((self.dig_h4 as i32) << 20) - ((self.dig_h5 as i32) * v_x1_u32r))
            + 16384i32)
            >> 15)
            * (((((((v_x1_u32r * (self.dig_h6 as i32)) >> 10)
                * (((v_x1_u32r * (self.dig_h3 as i32)) >> 11) + 32768i32))
                >> 10)
                + 2097152i32)
                * (self.dig_h2 as i32)
                + 8192)
                >> 14);
        v_x1_u32r = v_x1_u32r
            - (((((v_x1_u32r >> 15) * (v_x1_u32r >> 15)) >> 7) * (self.dig_h1 as i32)) >> 4);
        if v_x1_u32r < 0 {
            v_x1_u32r = 0;
        }
        if v_x1_u32r > 419430400 {
            v_x1_u32r = 419430400;
        }
        ((v_x1_u32r >> 12) as f64) / 1024.0
    }

    pub fn calibration_pressure(&mut self, adc_p: i32) -> f64 {
        let mut var1: i64;
        let mut var2: i64;
        let mut p: i64;
        var1 = (self.t_fine as i64) - 128000;
        var2 = var1 * var1 * (self.dig_p6 as i64);
        var2 = var2 + ((var1 * (self.dig_p5 as i64)) << 17);
        var2 = var2 + ((self.dig_p4 as i64) << 35);
        var1 = (var1 * var1 * ((self.dig_p3 as i64) >> 8)) + ((var1 * (self.dig_p2 as i64)) << 12);
        var1 = ((((1 as i64) << 47) + var1) * (self.dig_p1 as i64)) >> 33;
        if var1 == 0 {
            return 0.0;
        }
        p = 1048576 - (adc_p as i64);
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = ((self.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2 = ((self.dig_p8 as i64) * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((self.dig_p7 as i64) << 4);
        ((p / 256) as f64) / 100.0
    }
    pub fn get_elements(&mut self) -> (f64, f64, f64) {
        let t = self.calibration_temperature(self.temp_raw as i32);
        let h = self.calibration_humidity(self.humi_raw as i32);
        let p = self.calibration_pressure(self.pres_raw as i32);
        (t, h, p)
    }
}

pub struct SPIInterface<IF, GPIO>
where
    IF: Default<u8> + OtherDefault<u8>,
    GPIO: OutputPin,
{
    pub spi: IF,
    pub cs: GPIO,
}

impl<IF, GPIO> Interface for SPIInterface<IF, GPIO>
where
    IF: Default<u8> + OtherDefault<u8>,
    GPIO: OutputPin,
{
    fn read_register(&mut self, register: u8) -> u8 {
        let _ = self.cs.set_low();
        let mut buf: [u8; 1] = [register];
        let _ = self.spi.write(&buf);
        let mut value: u8 = 0;
        if let Ok(n) = self.spi.transfer(&mut buf) {
            value = n[0];
        }
        let _ = self.cs.set_high();
        value
    }
    fn write_register(&mut self, register: u8, value: u8) {
        let _ = self.cs.set_low();
        let _ = self.spi.write(&[(register & 0x7f), value]);
        let _ = self.cs.set_high();
    }
    fn write(&mut self, value: &[u8]) {
        let _ = self.spi.write(&value);
    }

    fn read_trim(&mut self, buffer: &mut [u8; 32]) {
        let mut buf: [u8; 1] = [CALIBRATION_OFFSET_T_P];
        let _ = self.cs.set_low();
        let _ = self.spi.write(&buf);
        for i in 0..24 {
            if let Ok(n) = self.spi.transfer(&mut buf) {
                buffer[i] = n[0];
            }
        }
        let _ = self.cs.set_high();
        let _ = self.cs.set_low();
        //        buf[0] = CALIBRATION_OFFSET_H1;
        let mut buf: [u8; 1] = [CALIBRATION_OFFSET_H1];
        let _ = self.spi.write(&buf);
        for i in 24..25 {
            if let Ok(n) = self.spi.transfer(&mut buf) {
                buffer[i] = n[0];
            }
        }
        let _ = self.cs.set_high();
        let _ = self.cs.set_low();
        //        buf[0] = CALIBRATION_OFFSET_H2;
        let mut buf: [u8; 1] = [CALIBRATION_OFFSET_H2];
        let _ = self.spi.write(&buf);
        for i in 25..32 {
            if let Ok(n) = self.spi.transfer(&mut buf) {
                buffer[i] = n[0];
            }
        }
        let _ = self.cs.set_high();
    }
    fn read_data(&mut self, buffer: &mut [u8; 8]) {
        let mut buf: [u8; 1] = [PRESS_MSB_REG];
        let _ = self.cs.set_low();
        let _ = self.spi.write(&buf);
        for i in 0..8 {
            if let Ok(n) = self.spi.transfer(&mut buf) {
                buffer[i] = n[0];
            }
        }
        let _ = self.cs.set_high();
    }
}
