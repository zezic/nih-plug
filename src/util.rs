// nih-plug: plugins, but rewritten in Rust
// Copyright (C) 2022 Robbert van der Helm
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub const MINUS_INFINITY_DB: f32 = -100.0;

/// Convert decibels to a voltage gain ratio, treating anything below -100 dB as minus infinity.
pub fn db_to_gain(dbs: f32) -> f32 {
    if dbs > MINUS_INFINITY_DB {
        10.0f32.powf(dbs * 0.05)
    } else {
        0.0
    }
}

/// Convert a voltage gain ratio to decibels. Gain ratios that aren't positive will be treated as
/// [MINUS_INFINITY_DB].
pub fn gain_to_db(gain: f32) -> f32 {
    if gain > 0.0 {
        gain.log10() * 20.0
    } else {
        MINUS_INFINITY_DB
    }
}

/// Convert a MIDI note ID to a frequency at A4 = 440 Hz equal temperament and middle C = note 60 =
/// C4.
pub fn midi_note_to_freq(pitch: u8) -> f32 {
    2.0f32.powf((pitch as f32 - 69.0) / 12.0) * 440.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_to_gain_positive() {
        assert_eq!(db_to_gain(3.0), 1.4125376);
    }

    #[test]
    fn test_db_to_gain_negative() {
        assert_eq!(db_to_gain(-3.0), 1.4125376f32.recip());
    }

    #[test]
    fn test_db_to_gain_minus_infinity() {
        assert_eq!(db_to_gain(-100.0), 0.0);
    }

    #[test]
    fn test_gain_to_db_positive() {
        assert_eq!(gain_to_db(4.0), 12.041201);
    }

    #[test]
    fn test_gain_to_db_negative() {
        assert_eq!(gain_to_db(0.25), -12.041201);
    }

    #[test]
    fn test_gain_to_db_minus_infinity_zero() {
        assert_eq!(gain_to_db(0.0), MINUS_INFINITY_DB);
    }

    #[test]
    fn test_gain_to_db_minus_infinity_negative() {
        assert_eq!(gain_to_db(-2.0), MINUS_INFINITY_DB);
    }
}
