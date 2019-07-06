
use baselisk_core::shared::event::midi::RawMidi;

pub fn raw_midi_from_vst(raw_event: &vst::event::MidiEvent) -> RawMidi {
    RawMidi {
        time: raw_event.delta_frames as usize,
        status: raw_event.data[0],
        data1: raw_event.data[1],
        data2: raw_event.data[2],
    }
}
