//! Scary song playback through a UART MP3 player module.

use embedded_io::Write;

/// The first track on the SD card, used as the scare sound.
pub const FIRST_TRACK: u16 = 1;

/// Anything that can start playback of a track.
pub trait AudioPlayer {
    type Error;

    fn play_track(&mut self, track: u16) -> Result<(), Self::Error>;
}

const FRAME_START: u8 = 0x7E;
const FRAME_END: u8 = 0xEF;
const FRAME_VERSION: u8 = 0xFF;
const FRAME_LENGTH: u8 = 0x06;
const CMD_PLAY_TRACK: u8 = 0x03;

/// A DFPlayer-style MP3 module driven over a UART.
pub struct DfPlayer<Writer> {
    writer: Writer,
}

impl<Writer> DfPlayer<Writer> {
    /// Creates a player from a writable UART.
    pub fn new(writer: Writer) -> Self {
        Self { writer }
    }
}

impl<Writer: Write> AudioPlayer for DfPlayer<Writer> {
    type Error = Writer::Error;

    fn play_track(&mut self, track: u16) -> Result<(), Self::Error> {
        let frame = play_track_frame(track);
        self.writer.write_all(&frame)
    }
}

/// Builds the DFPlayer command frame that plays `track` from the root folder.
fn play_track_frame(track: u16) -> [u8; 10] {
    let [folder, number] = track.to_be_bytes();
    let checksum = frame_checksum(CMD_PLAY_TRACK, folder, number);
    [
        FRAME_START,
        FRAME_VERSION,
        FRAME_LENGTH,
        CMD_PLAY_TRACK,
        0x00,
        folder,
        number,
        (checksum >> 8) as u8,
        checksum as u8,
        FRAME_END,
    ]
}

/// Two's complement checksum over version, length, command and parameters.
fn frame_checksum(command: u8, parameter_high: u8, parameter_low: u8) -> u16 {
    let sum = u16::from(FRAME_VERSION)
        + u16::from(FRAME_LENGTH)
        + u16::from(command)
        + u16::from(parameter_high)
        + u16::from(parameter_low);
    0u16.wrapping_sub(sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{IoError, TestWriter};

    #[test]
    fn first_track_frame_matches_the_datasheet_example() {
        assert_eq!(
            play_track_frame(FIRST_TRACK),
            [0x7E, 0xFF, 0x06, 0x03, 0x00, 0x00, 0x01, 0xFE, 0xF7, 0xEF]
        );
    }

    #[test]
    fn sends_the_play_command_through_the_writer() {
        let mut player = DfPlayer::new(TestWriter::default());

        player.play_track(7).unwrap();

        assert_eq!(
            player.writer.bytes,
            [0x7E, 0xFF, 0x06, 0x03, 0x00, 0x00, 0x07, 0xFE, 0xF1, 0xEF]
        );
    }

    #[test]
    fn splits_tracks_above_255_into_folder_and_number() {
        let mut player = DfPlayer::new(TestWriter::default());

        player.play_track(0x0102).unwrap();

        assert_eq!(
            player.writer.bytes,
            [0x7E, 0xFF, 0x06, 0x03, 0x00, 0x01, 0x02, 0xFE, 0xF5, 0xEF]
        );
    }

    #[test]
    fn reports_a_failing_writer() {
        let mut player = DfPlayer::new(TestWriter {
            fail: true,
            ..TestWriter::default()
        });

        assert_eq!(player.play_track(FIRST_TRACK), Err(IoError));
    }
}
