use clap::*;
use image::io::Reader as ImageReader;
use std::fs;
use std::io;

#[cfg(any(target_os = "linux"))]
mod fx;

fn main() -> io::Result<()> {
    match cli().get_matches().subcommand() {
        Some(("convert", matches)) => {
            convert(
                matches.get_one::<String>("IN").unwrap(),
                matches.get_one::<String>("OUT").unwrap(),
            )?;
        }
        Some(("fx", _matches)) => {
            #[cfg(any(target_os = "linux"))]
            {
                use klaptik::Point;
                use klaptik::RenderRequest;

                fn parse(val: &str) -> u32 {
                    if val.starts_with("0x") {
                        u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap()
                    } else {
                        u32::from_str_radix(val, 10).unwrap()
                    }
                }

                match _matches.subcommand() {
                    Some(("render", matches)) => {
                        let req = RenderRequest::new(
                            Point::new(
                                parse(matches.get_one::<String>("X").unwrap()) as _,
                                parse(matches.get_one::<String>("Y").unwrap()) as _,
                            ),
                            parse(matches.get_one::<String>("SPRITE").unwrap()) as _,
                            parse(matches.get_one::<String>("GLYPH").unwrap()) as _,
                        );

                        fx::send_render_request(
                            matches.get_one::<String>("DEVICE").unwrap(),
                            parse(matches.get_one::<String>("ADDRESS").unwrap()) as _,
                            req,
                        );
                    }
                    Some(("read", matches)) => {
                        let val = fx::read_register(
                            matches.get_one::<String>("DEVICE").unwrap(),
                            parse(matches.get_one::<String>("ADDRESS").unwrap()) as _,
                            parse(matches.get_one::<String>("REG").unwrap()) as _,
                        );
                        print!("0x");
                        for byte in val.to_be_bytes() {
                            print!("{0:0>2x}", byte);
                        }
                        println!("");
                    }
                    Some(("write", matches)) => {
                        fx::write_register(
                            matches.get_one::<String>("DEVICE").unwrap(),
                            parse(matches.get_one::<String>("ADDRESS").unwrap()) as _,
                            parse(matches.get_one::<String>("REG").unwrap()) as _,
                            parse(matches.get_one::<String>("VAL").unwrap()),
                        );
                    }
                    Some(("upload", matches)) => {
                        fx::upload_sprite(
                            matches.get_one::<String>("DEVICE").unwrap(),
                            parse(matches.get_one::<String>("ADDRESS").unwrap()) as _,
                            parse(matches.get_one::<String>("SPRITE").unwrap()) as _,
                            parse(matches.get_one::<String>("GLYPH_WIDTH").unwrap()) as _,
                            parse(matches.get_one::<String>("GLYPH_HEIGHT").unwrap()) as _,
                            &fs::read(matches.get_one::<String>("BITMAP").unwrap()).unwrap(),
                        );
                    }
                    Some(("delete", matches)) => {
                        fx::delete_sprite(
                            matches.get_one::<String>("DEVICE").unwrap(),
                            parse(matches.get_one::<String>("ADDRESS").unwrap()) as _,
                            parse(matches.get_one::<String>("SPRITE").unwrap()) as _,
                        );
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn cli() -> Command {
    let cli = Command::new("klaptik")
        .about("Klaptik CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("convert")
                .about("Convert image to raw sprite")
                .arg(arg!(-i <IN> "Input file"))
                .arg(arg!(-o <OUT> "Output file"))
                .arg_required_else_help(true),
        );

    if cfg!(any(target_os = "linux")) {
        cli.subcommand(
            Command::new("fx")
                .about("Klaptik-FX Utils")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("render")
                        .about("Send render request")
                        .arg(arg!(-d <DEVICE> "I2C device").default_value("/dev/i2c-1"))
                        .arg(arg!(-a <ADDRESS> "I2C address").default_value("0x2b"))
                        .arg(arg!(-x <X> "X").default_value("0"))
                        .arg(arg!(-y <Y> "Y").default_value("0"))
                        .arg(arg!(-s <SPRITE> "Sprite id").required(true))
                        .arg(arg!(-g <GLYPH> "Glyph index").required(true)),
                )
                .subcommand(
                    Command::new("read")
                        .about("Read register")
                        .arg(arg!(-d <DEVICE> "I2C device").default_value("/dev/i2c-1"))
                        .arg(arg!(-a <ADDRESS> "I2C address").default_value("0x2a"))
                        .arg(arg!(-r <REG> "Register").required(true)),
                )
                .subcommand(
                    Command::new("write")
                        .about("Write register")
                        .arg(arg!(-d <DEVICE> "I2C device").default_value("/dev/i2c-1"))
                        .arg(arg!(-a <ADDRESS> "I2C address").default_value("0x2a"))
                        .arg(arg!(-r <REG> "Register").required(true))
                        .arg(arg!(-v <VAL> "Value").required(true)),
                )
                .subcommand(
                    Command::new("upload")
                        .about("Upload sprite")
                        .arg(arg!(-d <DEVICE> "I2C device").default_value("/dev/i2c-1"))
                        .arg(arg!(-a <ADDRESS> "I2C address").default_value("0x2a"))
                        .arg(arg!(-s <SPRITE> "Sprite ID").required(true))
                        .arg(arg!(-w <GLYPH_WIDTH> "Glyphs width").required(true))
                        .arg(arg!(-h <GLYPH_HEIGHT> "Glyphs height").required(true))
                        .arg(arg!(-b <BITMAP> "Sprite bitmap").required(true)),
                )
                .subcommand(
                    Command::new("delete")
                        .about("Delete sprite")
                        .arg(arg!(-d <DEVICE> "I2C device").default_value("/dev/i2c-1"))
                        .arg(arg!(-a <ADDRESS> "I2C address").default_value("0x2a"))
                        .arg(arg!(-s <SPRITE> "Sprite ID").required(true)),
                ),
        )
    } else {
        cli
    }
}

fn convert(input: &String, output: &String) -> io::Result<()> {
    let img = ImageReader::open(input)?
        .decode()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let grayscale = img.grayscale();
    let width = img.width() as usize;
    let height = img.height() as usize;
    let raw = grayscale.as_bytes();
    let mut bin = vec![];
    for y in 0..height / 8 {
        for x in 0..width {
            let byte = (0..8).fold(0, |acc, shift| {
                if raw[x + (y * 8 + shift) * width] >= 32 {
                    acc | 1 << shift
                } else {
                    acc
                }
            });
            bin.push(byte);
        }
    }
    fs::write(output, &bin)
}
