use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use openweathermap::update;
use rusttype::Font;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use st7789_rs::color::Color;
use st7789_rs::ST7789;

const DC: u8 = 25;
const CS: u8 = 8;
const BL: u8 = 18;
const RST: u8 = 27;

struct Symbols {}
impl Symbols {
  const CLOUDY: &'static str = "󰖐";
  const FOGGY: &'static str = "󰖑";
  const DRIZZLE: &'static str = "";
  const STORM: &'static str = "";
  const NIGHT: &'static str = "󰖔";
  const P_CLOUDY: &'static str = "󰖕";
  const RAIN: &'static str = "󰖗";
  const SNOW: &'static str = "󰖘";
  const SUNNY: &'static str = "󰖙";
}

fn main() {
  let mut display = setup_display(
    CS, DC, BL, RST,
    0, 35
  );

  run(&mut display);
}

fn run(display: &mut ST7789) {
  let key_binding = std::fs::read_to_string("data/key.txt")
    .expect("couldn't read key file!");
  let key= key_binding
    .as_str();

  let receiver = openweathermap::init(
    "Windsor, CA",
    "metric", "en",
      key,
    10
  );

  let background = Color::new(0xe0d0ff);
  let foreground = Color::new(0x000000);

  let font_binding = std::fs::read("data/JetBrainsMonoNerdFont-ExtraBold.ttf")
    .expect("Couldn't read font file!");
  let font = Font::try_from_vec(font_binding)
    .expect("Couldn't parse font data!");

  let running_main = Arc::new(Mutex::new(true));
  let running_handler = running_main.clone();

  let mut signals = Signals::new(&[SIGINT, SIGTERM])
    .expect("Couldn't make signal handler!");

  thread::spawn(move || {
    for signal in signals.forever() {
      match signal {
        SIGINT | SIGTERM => {
          match running_handler.lock() {
            Ok(mut handler) => *handler = false,
            Err(_e) => panic!("Couldn't lock mutex!")
          }
        }
        _ => ()
      }
    }
  });

  while *(running_main.lock().unwrap()) {
    let weather_data = match update(&receiver) {
      Some(update) => match update {
        Ok(weather) => Some(weather),
        Err(e) => {
          println!("Error: {}", e);
          None
        }
      }
      None => None
    };

    match weather_data {
      Some(data) => {
        let icon = get_icon(&data.weather[0].icon);
        let temp = format!("{}°", data.main.temp as i8);
        let desc = format!("{}", data.weather[0].description);
        let time = format!("Updated: {}", chrono::Local::now()
          .format("%-I:%M %p"));

        display.draw_clear(&background);

        display.draw_text(
          icon.as_str(),
          &font,
          30,
          -5,
          &foreground,
          128.0
        );

        display.draw_text(
          temp.as_str(),
          &font,
          128,
          15,
          &foreground,
          80.0
        );

        display.draw_text(
          desc.as_str(),
          &font,
          30,
          105,
          &foreground,
          48.0
        );

        display.draw_text(
          time.as_str(),
          &font,
          128,
          85,
          &foreground,
          20.0
        );

        display.display();
      }
      None => sleep(Duration::from_secs_f64(0.5))
    }
  }

  println!("Cleaning up display");
  display.draw_clear(&Color::new(0x000000));
  display.cleanup();
  println!("Goodbye");
}

fn get_icon(cond: &String) -> String {
  return match cond.as_str() {
    "01d" => Symbols::SUNNY,
    "01n" => Symbols::NIGHT,
    "02d" | "02n" => Symbols::P_CLOUDY,
    "03d" | "03n" | "04d" | "04n" => Symbols::CLOUDY,
    "09d" | "09n" => Symbols::RAIN,
    "10d" | "10b" => Symbols::DRIZZLE,
    "11d" | "11n" => Symbols::STORM,
    "13d" | "13n" => Symbols::SNOW,
    "50d" | "50n" => Symbols::FOGGY,
    _ => "X"
  }.to_string();
}

fn setup_display(
  cs: u8, dc: u8, bl: u8, rst: u8,
  off_x: i16, off_y: i16
) -> ST7789 {
  let mut display = ST7789::new(
    0, 0,
    cs, dc, bl,
    60_000_000
  )
    .with_reset(rst)
    .with_offset(off_x, off_y);

  display.init();
  display.draw_clear(&Color::new(0x000000));
  display.display();

  return display;
}