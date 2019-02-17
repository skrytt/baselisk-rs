extern crate ansi_term;
extern crate portaudio;
extern crate portmidi;

use std::fmt;

/// This View struct represents the interface through which the view of the model
/// may be queried.
///
/// The audio (PortAudio) and midi (PortMidi) device lists are generated on
/// audio_thread_context startup only. If the actual devices change, the user would need
/// need to restart the program to get the updated lists.
///
/// However, the nodes data can change during program execution.
pub struct View {
    pub audio: AudioView,
    pub midi: MidiView,
    pub graph: EngineView,
}

impl View {
    pub fn new(audio: &portaudio::PortAudio, midi: &portmidi::PortMidi) -> View {
        View {
            audio: AudioView::new(audio),
            midi: MidiView::new(midi),
            graph: EngineView::new(),
        }
    }
}

/// AudioView represents information relating to our use of PortAudio that should
/// be available to the user.
pub struct AudioView {
    devices_text: String,
}

impl AudioView {
    fn new(audio: &portaudio::PortAudio) -> AudioView {
        let default_output_device_index = audio.default_output_device().unwrap();
        let devices = audio.devices().unwrap();

        let mut device_texts: Vec<String> = Vec::new();

        for device in devices {
            if let Ok(device) = device {
                let (idx, info) = device;

                let mut device_text = format!("{}) {}:", i32::from(idx), info.name,);

                if info.max_output_channels > 0 {
                    device_text = format!(
                        "{}{},",
                        device_text,
                        ansi_term::Colour::Blue
                            .paint(format!(" {} outputs", info.max_output_channels.to_string()))
                    );
                } else {
                    device_text = format!("{} 0 outputs,", device_text);
                }

                device_text = format!("{} {} Hz", device_text, info.default_sample_rate);

                if idx == default_output_device_index {
                    device_text = format!("{} [default output]", device_text,);
                };
                device_texts.push(device_text);
            }
        }
        AudioView {
            devices_text: device_texts.join("\n"),
        }
    }
}

impl fmt::Display for AudioView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.devices_text).unwrap();
        Ok(())
    }
}

/// MidiView represents information relating to our use of PortMidi that should
/// be available to the user.
pub struct MidiView {
    devices_texts: Vec<String>,
    selected: Option<usize>,
}

impl MidiView {
    fn new(midi: &portmidi::PortMidi) -> MidiView {
        let mut result = MidiView {
            devices_texts: Vec::new(),
            selected: None,
        };
        for dev in midi.devices().unwrap() {
            result.devices_texts.push(format!("{}", dev));
        };
        result
    }

    pub fn select_device(&mut self, device: usize) {
        self.selected = Some(device);
    }
}

impl fmt::Display for MidiView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, device_text) in self.devices_texts.iter().enumerate() {
            write!(f, "{}", device_text).unwrap();
            if let Some(selected) = self.selected {
                if i == selected {
                    write!(f, " [selected]")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct EngineView {
}

impl EngineView {
    pub fn new() -> EngineView {
        EngineView {
        }
    }
}

impl fmt::Display for EngineView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TODO: fmt::Display for EngineView")?;
        Ok(())
    }
}
