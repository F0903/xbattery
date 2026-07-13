const A4_MIDI: i32 = 69;
const A4_FREQUENCY_HZ: f64 = 440.0;
const MIN_MIDI: i32 = 0;
const MAX_MIDI: i32 = 127;
const MAX_CENTS: f64 = 100.0;

pub(crate) fn frequency(note: &str) -> Result<f32, &'static str> {
    let bytes = note.as_bytes();
    let Some(letter) = bytes.first().copied() else {
        return Err("note must not be empty");
    };

    let semitone = match letter.to_ascii_uppercase() {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return Err("note must start with A through G"),
    };

    let mut cursor = 1;
    let accidental = match bytes.get(cursor).copied() {
        Some(b'#') => {
            cursor += 1;
            1
        }
        Some(b'b' | b'B') => {
            cursor += 1;
            -1
        }
        _ => 0,
    };

    let octave_start = cursor;
    if bytes.get(cursor) == Some(&b'-') {
        cursor += 1;
    }
    let digit_start = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == digit_start {
        return Err("note must include an octave");
    }

    let octave = note[octave_start..cursor]
        .parse::<i32>()
        .map_err(|_| "note octave is invalid")?;
    let cents = parse_cents(&note[cursor..])?;
    let midi = 12_i64 * (i64::from(octave) + 1) + i64::from(semitone + accidental);
    let pitch = midi as f64 + cents / 100.0;
    if !(f64::from(MIN_MIDI)..=f64::from(MAX_MIDI)).contains(&pitch) {
        return Err("note is outside the MIDI range C-1 through G9");
    }

    let semitones_from_a4 = pitch - f64::from(A4_MIDI);
    let frequency = A4_FREQUENCY_HZ * 2.0_f64.powf(semitones_from_a4 / 12.0);
    Ok(frequency as f32)
}

fn parse_cents(suffix: &str) -> Result<f64, &'static str> {
    if suffix.is_empty() {
        return Ok(0.0);
    }

    let Some(value) = suffix
        .strip_suffix(['c', 'C'])
        .filter(|value| value.starts_with(['+', '-']))
    else {
        return Err("detuning must look like +9c or -12.5c");
    };

    let cents = value
        .parse::<f64>()
        .map_err(|_| "detuning cents are invalid")?;
    if !cents.is_finite() || cents.abs() > MAX_CENTS {
        return Err("detuning must be between -100c and +100c");
    }

    Ok(cents)
}

#[cfg(test)]
mod tests {
    use super::frequency;

    #[test]
    fn converts_scientific_pitch_to_frequency() {
        assert_eq!(frequency("A4").unwrap(), 440.0);
        assert_close(frequency("C4").unwrap(), 261.625_58);
        assert_close(frequency("F#4").unwrap(), 369.994_42);
        assert_close(frequency("Bb4").unwrap(), 466.163_76);
    }

    #[test]
    fn accepts_case_insensitive_enharmonic_notes() {
        assert_close(frequency("f#4").unwrap(), frequency("Gb4").unwrap());
        assert_close(frequency("B#3").unwrap(), frequency("C4").unwrap());
        assert_close(frequency("Cb4").unwrap(), frequency("B3").unwrap());
    }

    #[test]
    fn applies_cents_detuning() {
        assert_close(frequency("A4+100c").unwrap(), frequency("A#4").unwrap());
        assert!(frequency("C5+9.07c").unwrap() > frequency("C5").unwrap());
        assert!(frequency("C5-9.07c").unwrap() < frequency("C5").unwrap());
    }

    #[test]
    fn accepts_midi_boundaries() {
        assert!(frequency("C-1").is_ok());
        assert!(frequency("G9").is_ok());
        assert!(frequency("Cb-1").is_err());
        assert!(frequency("G#9").is_err());
        assert!(frequency("C-1-100c").is_err());
        assert!(frequency("G9+100c").is_err());
    }

    #[test]
    fn rejects_invalid_note_syntax() {
        for note in [
            "",
            "H4",
            "C",
            "C##4",
            "C4.5",
            "C4junk",
            " C4",
            "C4 ",
            "C4+100.1c",
            "C4+infc",
            "C2147483647",
            "C-2147483648",
        ] {
            assert!(frequency(note).is_err(), "accepted invalid note {note:?}");
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }
}
