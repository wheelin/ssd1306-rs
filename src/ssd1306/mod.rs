extern crate i2cdev;

use self::i2cdev::core::*;
use self::i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

pub trait Display {
    fn initialize(&mut self) -> Result<(), String>;
    fn invert_display(&mut self, state: bool) -> Result<(), String>;
    fn draw_pixel(&mut self, x: i16, y: i16, color: u16) -> Result<(), String>;
    fn clear(&mut self) -> Result<(), String>;
    fn deinitialize(&mut self) -> Result<(), String>;
    fn update(&mut self) -> Result<(), String>;

    fn get_width(&self) -> u16;
    fn get_height(&self) -> u16;

    fn get_def_text_color(&self) -> u16;
    fn get_def_bg_color(&self) -> u16;
}

pub const BLACK: u16 = 0;
pub const WHITE: u16 = 1;
pub const LCD_WIDTH: u16 = 128;
pub const LCD_HEIGHT: u16 = 64;

const ADDR: u16 = 0x3C;
const COMMAND_MODE: u8 = 0x00; /* C0 and DC bit are 0 				 */
const DATA_MODE: u8 = 0x40; /* C0 bit is 0 and DC bit is 1 */

enum Command {
    InverseDisplay = 0xA7,
    NormalDisplay = 0xA6,
    Off = 0xAE,
    On = 0xAF,
    SetContrastLevel = 0x81,
    ActivateScroll = 0x2F,
    DeactivateScroll = 0x2E,
    SetAllOn = 0xA5,
    ResumeToGDDRAM = 0xA4,

    SetDisplayOffset = 0xD3,
    SetComPins = 0xDA,
    SetVComDetect = 0xDB,
    SetDisplayClockDiv = 0xD5,
    SetPrecharge = 0xD9,
    SetMultiplex = 0xA8,
    SetLowColumn = 0x00,
    SetHighColumn = 0x10,
    SetStartLine = 0x40,
    SetMemoryMode = 0x20,
    ComScanInc = 0xC0,
    ComScanDec = 0xC8,
    SegRemap = 0xA0,
    ChargePump = 0x8D,

    // Scrolling
    SetVerticalScrollArea = 0xA3,
    RightHorizontalScroll = 0x26,
    LeftHorizontalScroll = 0x27,
    VerticalAndRightHorizontalScroll = 0x29,
    VerticalAndLeftHorizontalScroll = 0x2A,
}

enum VccType {
    ExternalVcc = 0x01,
    InternalVcc = 0x02,
}

enum ScrollDirection {
    Left = 0,
    Right = 1,
}

enum ScrollFrameNumber {
    Frames2 = 0x07,
    Frames3 = 0x04,
    Frames4 = 0x05,
    Frames5 = 0x00,
    Frames25 = 0x06,
    Frames64 = 0x01,
    Frames128 = 0x02,
    Frames256 = 0x03,
}

enum MemoryMode {
    Horizontal = 0,
    Vertical = 1,
    Page = 2,
}

pub struct SSD1306 {
    lcd_width: u16,
    lcd_height: u16,
    vcc_type: VccType,
    poled_buf: Vec<u8>,
    old_poled_buf: Vec<u8>,
    i2c: LinuxI2CDevice,
}


impl SSD1306 {
    pub fn new() -> SSD1306 {
        let w = 128;
        let h = 64;
        SSD1306 {
            lcd_width: 128,
            lcd_height: 64,
            vcc_type: VccType::InternalVcc,
            poled_buf: vec![0;(w*h)/8],
            old_poled_buf: vec![0;(w*h)/8],
            i2c: LinuxI2CDevice::new("/dev/i2c-1", ADDR).unwrap_or_else(|_| {
                panic!("Cannot create i2c device for the display");
            }),
        }
    }

    pub fn begin(&mut self) -> Result<(), String> {
        // initialize display
        let multiplex = 0x3F;
        let compins = 0x12;
        let contrast = match self.vcc_type {
            VccType::ExternalVcc => 0x9F,
            VccType::InternalVcc => 0xCF,
        };
        let chargepump = match self.vcc_type {
            VccType::ExternalVcc => 0x10,
            VccType::InternalVcc => 0x14,
        };
        let precharge = match self.vcc_type {
            VccType::ExternalVcc => 0x22,
            VccType::InternalVcc => 0xF1,
        };

        self.send_command(Command::Off as u8);

        self.send_command(Command::SetDisplayClockDiv as u8);
        self.send_command(0x80);

        self.send_command(Command::SetMultiplex as u8);
        self.send_command(multiplex);

        self.send_command(Command::SetDisplayOffset as u8);
        self.send_command(0x00);

        self.send_command((Command::SetStartLine as u8) | 0x0);

        self.send_command(Command::ChargePump as u8);
        self.send_command(chargepump);

        self.send_command(Command::SetMemoryMode as u8);
        self.send_command(MemoryMode::Horizontal as u8);

        self.send_command((Command::SegRemap as u8) | 0x01);

        self.send_command(Command::ComScanDec as u8);

        self.send_command(Command::SetComPins as u8);
        self.send_command(compins);

        self.send_command(Command::SetContrastLevel as u8);
        self.send_command(contrast);

        self.send_command(Command::SetPrecharge as u8);
        self.send_command(precharge);

        self.send_command(Command::SetVComDetect as u8);
        self.send_command(0x40);

        self.send_command(Command::ResumeToGDDRAM as u8);

        self.send_command(Command::NormalDisplay as u8);

        self.send_command(0x21);
        self.send_command(0x00);
        self.send_command(127);

        self.send_command(0x22);
        self.send_command(0);
        self.send_command(7);

        self.stop_scroll();
        self.clear();

        self.send_command(Command::On as u8);

        Ok(())
    }

    fn send_command(&mut self, c: u8) {
        match self.i2c.smbus_write_byte_data(COMMAND_MODE, c) {
            Ok(_) => (),
            Err(x) => panic!(format!("{:?}", x)),
        };
    }

    fn send_data(&mut self, d: u8) {
        match self.i2c.smbus_write_byte_data(DATA_MODE, d) {
            Ok(_) => (),
            Err(x) => panic!(format!("{:?}", x)),
        };
    }

    pub fn clear(&mut self) {
        self.poled_buf = vec![0; ((self.lcd_width * self.lcd_height)/8) as usize];
        self.old_poled_buf = self.poled_buf.clone();
    }

    pub fn invert(&mut self, state: bool) {
        if state {
            self.send_command(Command::InverseDisplay as u8);
        } else {
            self.send_command(Command::NormalDisplay as u8);
        }
    }

    pub fn display_all(&mut self) {
        self.send_command((Command::SetLowColumn as u8) | 0x00);
        self.send_command((Command::SetHighColumn as u8) | 0x00);
        self.send_command((Command::SetStartLine as u8) | 0x00);

        for i in 0..(self.lcd_width * self.lcd_height / 8) {
            let data = self.poled_buf[i as usize];
            self.send_data(data);
        }
    }

    pub fn display(&mut self) {
        let mut first_change = 0;
        let mut last_change = self.lcd_width * self.lcd_height / 8;
        for i in 0..(self.lcd_width * self.lcd_height / 8) {
            if self.poled_buf[i as usize] != self.old_poled_buf[i as usize] {
                first_change = i;
                break;
            }
        }
        for i in (0..(self.lcd_width * self.lcd_height / 8)).rev() {
            if self.poled_buf[i as usize] != self.old_poled_buf[i as usize] {
                last_change = i + 1;
                break;
            }
        }
        let start_column = first_change % 128;
        let start_page = ((first_change as f32) / 128.0).floor() as u8;
        let end_column = last_change % 128;
        let end_page = ((last_change as f32) / 128.0).floor() as u8;

        //println!("#################################################################");
        //println!("Current first page : {}, current first column : {}",
        //         start_page,
        //         start_column);
        //println!("-----------------------------------------------------------------");
        //println!("Current last page : {}, current last column : {}",
        //         end_page,
        //         end_column);

        self.send_command(0x21);
        self.send_command(start_column as u8);
        self.send_command(end_column as u8);

        self.send_command(0x22);
        self.send_command(start_page as u8);
        self.send_command(end_page as u8);

        for i in first_change..last_change {
            let current_column = i % 128;
            let current_page = ((i as f32) / 128.0).floor() as u8;
            if current_column >= start_column && current_column <= end_column &&
               current_page >= start_page && current_page <= end_page {

                let data = self.poled_buf[i as usize];
                self.send_data(data);
            }
        }

        self.old_poled_buf = self.poled_buf.clone();
        self.poled_buf = vec![0; ((self.lcd_width*self.lcd_height)/8) as usize];
    }

    pub fn start_scroll_right(&mut self, start: u8, stop: u8) {
        self.send_command(Command::RightHorizontalScroll as u8);
        self.send_command(0x00);
        self.send_command(start);
        self.send_command(0x00);
        self.send_command(stop);
        self.send_command(0x01);
        self.send_command(0xFF);
        self.send_command(Command::ActivateScroll as u8);
    }

    pub fn start_scroll_left(&mut self, start: u8, stop: u8) {
        self.send_command(Command::LeftHorizontalScroll as u8);
        self.send_command(0x00);
        self.send_command(start);
        self.send_command(0x00);
        self.send_command(stop);
        self.send_command(0x01);
        self.send_command(0xFF);
        self.send_command(Command::ActivateScroll as u8);
    }

    pub fn start_scroll_diag_right(&mut self, start: u8, stop: u8) {
        let h = self.lcd_height - 1;
        self.send_command(Command::SetVerticalScrollArea as u8);
        self.send_command(0x00);
        self.send_command(h as u8);
        self.send_command(Command::VerticalAndRightHorizontalScroll as u8);
        self.send_command(0x00);
        self.send_command(start);
        self.send_command(0x00);
        self.send_command(stop);
        self.send_command(0x01);
        self.send_command(Command::ActivateScroll as u8);
    }

    pub fn start_scroll_diag_left(&mut self, start: u8, stop: u8) {
        let w = self.lcd_width - 1;
        self.send_command(Command::SetVerticalScrollArea as u8);
        self.send_command(0x00);
        self.send_command(w as u8);
        self.send_command(Command::VerticalAndLeftHorizontalScroll as u8);
        self.send_command(0x00);
        self.send_command(start);
        self.send_command(0x00);
        self.send_command(stop);
        self.send_command(0x01);
        self.send_command(Command::ActivateScroll as u8);
    }

    pub fn stop_scroll(&mut self) {
        self.send_command(Command::DeactivateScroll as u8);
    }
}

impl Display for SSD1306 {
    fn initialize(&mut self) -> Result<(), String> {
        self.invert(false);
        self.begin()
    }

    fn invert_display(&mut self, state: bool) -> Result<(), String> {
        self.invert(state);
        Ok(())
    }

    fn draw_pixel(&mut self, x: i16, y: i16, color: u16) -> Result<(), String> {
        if x > (self.lcd_width as i16) - 1 || y > (self.lcd_height as i16) - 1 || x < 0 || y < 0 {
            return Ok(());
        }
        if color != BLACK {
            self.poled_buf[(x + (y / 8) * self.lcd_width as i16) as usize] |= 1 << (y % 8);
        } else {
            self.poled_buf[(x + (y / 8) * self.lcd_width as i16) as usize] &= !(1 << (y % 8));
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        self.clear();
        self.display_all();
        Ok(())
    }

    fn deinitialize(&mut self) -> Result<(), String> {
        self.send_command(Command::Off as u8);
        Ok(())
    }

    fn update(&mut self) -> Result<(), String> {
        self.display();
        Ok(())
    }

    fn get_width(&self) -> u16 {
        self.lcd_width
    }

    fn get_height(&self) -> u16 {
        self.lcd_height
    }

    fn get_def_text_color(&self) -> u16 {
        WHITE
    }

    fn get_def_bg_color(&self) -> u16 {
        BLACK
    }
}
