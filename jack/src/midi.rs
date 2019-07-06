use baselisk_core::shared::event::midi::RawMidi;

pub fn raw_midi_from_jack(raw_event: &jack::RawMidi) -> RawMidi {
    RawMidi {
        time: raw_event.time as usize,
        status: raw_event.bytes[0],
        data1: raw_event.bytes[1],
        data2: raw_event.bytes[2],
    }
}



