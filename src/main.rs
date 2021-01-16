extern crate image;
extern crate meval;
extern crate pbr;
extern crate rand;

use anyhow::Result;
use clap::{App, Arg};
use image::{GenericImage, GenericImageView};
use pbr::ProgressBar;
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead};
use std::path::Path;
use text_io::read;

struct SortablePixel {
    rgba: image::Rgba<u8>,
    brightness: u32,
}

struct SortableInterval {
    pixels: Vec<SortablePixel>,
    positions: Vec<(u32, u32)>,
}

impl SortableInterval {
    fn new() -> SortableInterval {
        return SortableInterval {
            pixels: Vec::new(),
            positions: Vec::new(),
        };
    }
    fn add(&mut self, p: SortablePixel) {
        self.pixels.push(p);
    }
}

impl SortablePixel {
    fn _new(rgba: image::Rgba<u8>, brightness: u32) -> SortablePixel {
        SortablePixel { rgba, brightness }
    }
    fn from_rgba(x: u32, y: u32, rgba: image::Rgba<u8>) -> SortablePixel {
        SortablePixel {
            rgba,
            brightness: brightness(&rgba),
        }
    }
}

fn main() -> Result<()> {
    let matches = App::new("Pixel Sorter")
        .version("1.0")
        .author("Rishabh Bector <bector.rishabh@gmail.com>")
        .about("A pixel sorter written in Rust.")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .about("Sets a custom command file.")
                .takes_value(true),
        )
        .subcommand(App::new("shell").about("Starts an interactive command shell."))
        .get_matches();

    let mut img: Option<image::DynamicImage> = None;

    if let Some(ref _matches) = matches.subcommand_matches("shell") {
        loop {
            print!("> ");
            std::io::stdout().flush()?;
            let cmd_dirty: String = read!("{}\r");
            let cmd = cmd_dirty.replace("\n", "");
            let cmds: Vec<&str> = cmd.split(" ").collect();
            if cmds[0] == "open" {
                println!("Opening {}...", cmds[1]);
                img = Some(image::open(cmds[1])?);
                println!("Dimensions: {:?}", img.as_ref().expect("").dimensions());
                continue;
            }
            if cmds[0] == "exit" {
                println!("Quitting...");
                break;
            }

            run_command(&mut img, cmds)?;
        }
    } else {
        if let Some(f) = matches.value_of("file") {
            if let Ok(lines) = read_lines(f) {
                for line in lines {
                    if let Ok(ip) = line {
                        let cmd = ip.replace("\n", "");
                        let cmds: Vec<&str> = cmd.split(" ").collect();

                        if cmds[0] == "open" {
                            println!("Opening {}...", cmds[1]);
                            img = Some(image::open(cmds[1])?);
                            println!("Dimensions: {:?}", img.as_ref().expect("").dimensions());
                            continue;
                        }
                        if cmds[0] == "exit" {
                            println!("Quitting...");
                            break;
                        }

                        run_command(&mut img, cmds)?;
                    }
                }
            }
        } else {
            println!("No file provided. Run 'pixelsorter shell' to start a command shell.");
        }
    }
    Ok(())
}

fn run_command(img: &mut Option<image::DynamicImage>, cmds: Vec<&str>) -> Result<()> {
    match cmds[0] {
        "save" => {
            println!("Saving to {}...", cmds[1]);
            img.as_ref().expect("bruh").save(cmds[1])?;
        }
        "classic" => {
            let vertical: bool = read!("{}", cmds[1].bytes());
            let rev: bool = read!("{}", cmds[2].bytes());
            let rev_thresh: bool = read!("{}", cmds[3].bytes());
            let threshold: u32 = read!("{}", cmds[4].bytes());
            println!("Running: {:?}", cmds);
            run_classic(img, vertical, rev, rev_thresh, threshold);
        }
        "kernel" => {
            let rev: bool = read!("{}", cmds[1].bytes());
            let nx: u32 = read!("{}", cmds[4].bytes());
            let ny: u32 = read!("{}", cmds[5].bytes());
            println!("Running: {:?}", cmds);
            run_kernel(img, rev, nx, ny)
        }
        "vector" => {
            let rev: bool = read!("{}", cmds[1].bytes());
            let size: u32 = read!("{}", cmds[2].bytes());
            let amount: u32 = read!("{}", cmds[3].bytes());
            let spacing: u32 = read!("{}", cmds[4].bytes());
            let expression: String = read!("{}", cmds[5].bytes());
            println!("Running: {:?}", cmds);
            run_vectorfield(img, rev, size, amount, spacing, expression);
        }
        _ => {}
    }
    Ok(())
}

fn brightness(p: &image::Rgba<u8>) -> u32 {
    let sum: u32 = (p[0] as u32) + (p[1] as u32) + (p[2] as u32) + (p[3] as u32);
    return sum / 4;
}

fn transform_axis(in1: u32, size1: u32, size2: u32) -> u32 {
    (((in1 as f32) / (size1 as f32)) * (size2 as f32)) as u32
}

fn run_classic(
    img: &mut Option<image::DynamicImage>,
    vertical: bool,
    reverse: bool,
    reverse_threshold: bool,
    threshold: u32,
) {
    let im = img.as_mut().expect("msg: &str");
    let mut dimensions = im.dimensions();
    let mut pb = ProgressBar::new(dimensions.1 as u64);
    pb.format("╢▌▌░╟");
    match vertical {
        false => dimensions = (dimensions.0, dimensions.1),
        true => dimensions = (dimensions.1, dimensions.0),
    }
    for y in 0..dimensions.1 {
        let mut start: i32 = -1;
        for x in 0..dimensions.0 {
            let vy = match vertical {
                false => y,
                true => y,
            };
            let vx = match vertical {
                false => x,
                true => x,
            };
            let b = brightness(&get_pixel(vx, vy, im, vertical));
            if (b > threshold && !reverse_threshold) || (b < threshold && reverse_threshold) {
                if start == -1 {
                    start = vx as i32;
                }
            } else if (b < threshold && !reverse_threshold) || (b > threshold && reverse_threshold) {
                if start != -1 {
                    let stop = vx as i32;
                    let mut interval: SortableInterval = SortableInterval {
                        pixels: Vec::new(),
                        positions: Vec::new(),
                    };
                    for i in start..stop {
                        let pos: (u32, u32) = (i as u32, vy);
                        interval.pixels.push(SortablePixel::from_rgba(
                            i as u32,
                            vy,
                            get_pixel(pos.0, pos.1, im, vertical),
                        ));
                        interval.positions.push(pos);
                    }
                    sort_interval(&mut interval, im, reverse, vertical);
                    start = -1;
                }
            }
        }
        pb.inc();
    }
    pb.finish();
    println!("\n");
}

fn run_kernel(
    img: &mut Option<image::DynamicImage>,
    reverse: bool,
    numx: u32,
    numy: u32,
) {
    // Intervals are squares
    let im = img.as_mut().expect("msg: &str");
    let mut dimensions = im.dimensions();
    let mut pb = ProgressBar::new(numx as u64);
    pb.format("╢▌▌░╟");
    let width = dimensions.0 / numx;
    let height = dimensions.1 / numy;
    for x in 0..numx {
        pb.inc();
        for y in 0..numy {
            let mut interval: SortableInterval = SortableInterval {
                pixels: Vec::new(),
                positions: Vec::new(),
            };
            for kx in 0..width {
                for ky in 0..height {
                    let pos = (x * width + kx, y * height + ky);
                    interval.pixels.push(SortablePixel::from_rgba(
                        pos.0,
                        pos.1,
                        get_pixel(pos.0, pos.1, im, false),
                    ));
                    interval.positions.push(pos);
                }
            }
            println!("{}", interval.pixels.len());
            sort_interval(&mut interval, im, reverse, false);
        }
    }
    pb.finish();
    println!("\n");
}

fn run_vectorfield(
    img: &mut Option<image::DynamicImage>,
    rev: bool,
    size: u32,
    amount: u32,
    spacing: u32,
    expression: String,
) {
    // Intervals are created by finding the most similar nearby pixels
    let im = img.as_mut().expect("msg: &str");
    let mut dimensions = im.dimensions();
    let mut pb = ProgressBar::new(amount as u64);
    pb.format("╢▌▌░╟");
    let mut rng = rand::thread_rng();
    let expr: meval::Expr = expression.parse().unwrap();
    let func = expr.bind2("x", "y").unwrap();
    for i in 0..amount {
        pb.inc();
        let level = i / dimensions.1;
        let mut interval = SortableInterval::new();
        let mut pos = (0, 0);
        pos = (level * spacing, i % dimensions.1);
        let d = im.dimensions();
        for p in 0..size {
            if pos.0 >= d.0 || pos.1 >= d.1 {
                break;
            }
            interval.add(SortablePixel::from_rgba(
                pos.0,
                pos.1,
                get_pixel(pos.0, pos.1, im, false),
            ));
            interval.positions.push(pos);
            let dir = func(pos.0 as f64, pos.1 as f64);
            let vec = (dir.cos()*3.0, dir.sin()*3.0);
            pos = (pos.0+(vec.0 as u32), pos.1+(vec.1 as u32))
        }
        sort_interval(&mut interval, im, rev, false);
    }
    pb.finish();
    println!("\n");
}

fn find_next(dims: (u32, u32), pos: (u32, u32)) -> (u32, u32) {
    let mut out = (pos.0 + 1, pos.1 + 1);
    out
}

fn sort_interval(
    interval: &mut SortableInterval,
    img: &mut image::DynamicImage,
    reverse: bool,
    vertical: bool,
) {
    interval
        .pixels
        .sort_by(|a, b| b.brightness.cmp(&a.brightness));
    let length = interval.positions.len();
    let mut place = |pixel: usize, pos: usize| {
        put_pixel(
            interval.positions[pos].0,
            interval.positions[pos].1,
            interval.pixels[pixel].rgba,
            img,
            vertical,
        )
    };
    match reverse {
        false => {
            for pixel in 0..length {
                place(pixel, pixel)
            }
        }
        true => {
            for pixel in (0..length).rev() {
                place(pixel, length - pixel - 1)
            }
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_pixel(x: u32, y: u32, img: &mut image::DynamicImage, swap: bool) -> image::Rgba<u8> {
    match swap {
        false => img.get_pixel(x, y),
        true => img.get_pixel(y, x),
    }
}

fn put_pixel(x: u32, y: u32, rgba: image::Rgba<u8>, img: &mut image::DynamicImage, swap: bool) {
    match swap {
        false => img.put_pixel(x, y, rgba),
        true => img.put_pixel(y, x, rgba),
    }
}
