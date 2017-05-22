extern crate ssd1306_driver;

use ssd1306_driver::ssd1306::*;

fn main() {
    let mut display = SSD1306::new();
    display.begin().unwrap();
    display.draw_pixel(45, 23, BLACK).unwrap();
    println!("Hello, world!");
}
