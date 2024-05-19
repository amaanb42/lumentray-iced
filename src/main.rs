use brightness::Brightness;
use futures::TryStreamExt;
use iced::widget::{column, container, slider, text};
use iced::{window, Element, Length, Sandbox, Settings};
use std::error::Error;
use std::{fs, thread, time};
use std::process::Command;

#[tokio::main]
async fn main() -> iced::Result {
    Command::new("sh")
        .arg("-c")
        .arg("echo 'ddcci 0x37' | sudo tee /sys/bus/i2c/devices/i2c-1/new_device")
        .output()
        .expect("Failed to execute process");

    // sleep call to give time for above command to create required directory
    thread::sleep(time::Duration::from_secs_f32(3.0));
    
    let settings: Settings<()> = iced::settings::Settings {
        window: window::Settings {
            size: iced::Size::new(300.0, 100.0),
            resizable: (true),
            decorations: (true),
            ..Default::default()
        },
        ..Default::default()
    };
    Slider::run(settings)
}

async fn set_brightness(x: u32) -> Result<(), brightness::Error> {
    brightness::brightness_devices()
        .try_for_each(|mut dev| async move {
            dev.set(x).await?;
            Ok(())
        })
        .await
}

fn get_brightness() -> Result<u8, Box<dyn Error>> {
    let backlight_dir = "/sys/class/backlight/ddcci1";

    // Read the contents of the "brightness" file
    let brightness_path = format!("{}/actual_brightness", backlight_dir);
    let brightness_str = fs::read_to_string(&brightness_path)?;

    // Parse the string to u8
    let brightness = brightness_str.trim().parse::<u8>()?;

    Ok(brightness)
}

#[derive(Debug, Clone)]
pub enum Message {
    SliderChanged(u8),
}

pub struct Slider {
    value: u8,
    default: u8,
    step: u8,
    shift_step: u8,
}

impl Sandbox for Slider {
    type Message = Message;

    fn new() -> Slider {
        Slider {
            value: get_brightness().unwrap(),
            default: get_brightness().unwrap(),
            step: 1,
            shift_step: 1,
        }
    }

    // function to set the overall application theme
    fn theme(&self) -> iced::Theme {
        iced::Theme::CatppuccinMocha
    }

    fn title(&self) -> String {
        String::from("LumenTray")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged(value) => {
                self.value = value;
                tokio::spawn(async move {
                    if let Err(e) = set_brightness(value as u32).await {
                        eprintln!("Error: {}", e);
                    }
                });
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let horizontal_slider = container(
            slider(0..=100, self.value, Message::SliderChanged)
                .default(self.default)
                .step(self.step)
                .shift_step(self.shift_step),
        )
        .width(250);

        let text = text(self.value);

        container(
            column![
                container(horizontal_slider).width(Length::Fill).center_x(),
                container(text).width(Length::Fill).center_x(),
            ]
            .spacing(15),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
