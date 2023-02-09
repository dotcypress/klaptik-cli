use i2cdev::core::I2CDevice;
use i2cdev::linux::*;
use klaptik::{RenderRequest, SpriteId};

pub fn send_render_request(dev: &str, addr: u16, req: RenderRequest) {
    let mut dev = LinuxI2CDevice::new(dev, addr).unwrap();
    dev.write(&req.as_bytes()).unwrap();
}

pub fn read_register(dev: &str, addr: u16, reg: u8) -> u32 {
    let mut dev = LinuxI2CDevice::new(dev, addr).unwrap();
    dev.write(&[0x00, reg]).unwrap();

    let mut scratch = [0; 4];
    dev.read(&mut scratch).unwrap();
    u32::from_le_bytes(scratch)
}

pub fn write_register(dev: &str, addr: u16, reg: u8, val: u32) {
    let mut dev = LinuxI2CDevice::new(dev, addr).unwrap();
    dev.write(&[0x80, reg]).unwrap();
    dev.write(&val.to_le_bytes()).unwrap();
}

pub fn upload_sprite(
    dev: &str,
    addr: u16,
    id: SpriteId,
    glyph_width: u8,
    glyph_height: u8,
    bitmap: &[u8],
) {
    let glyph_len = glyph_width as usize * glyph_height as usize / 8;
    let glyphs = bitmap.len() / glyph_len;
    let mut dev = LinuxI2CDevice::new(dev, addr).unwrap();
    dev.write(&[0x81, id]).unwrap();
    dev.write(&[id, glyph_width, glyph_height, glyphs as _])
        .unwrap();
    for chunk in bitmap.chunks(255) {
        dev.write(chunk).unwrap();
    }
}

pub fn delete_sprite(dev: &str, addr: u16, id: SpriteId) {
    let mut dev = LinuxI2CDevice::new(dev, addr).unwrap();
    dev.write(&[0x82, id]).unwrap();
    dev.write(&[id, b'd', b'e', b'l']).unwrap();
}
