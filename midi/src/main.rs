use std::io::{Read, BufReader, SeekFrom, Seek};
use std::fs::File;
use std::io;
use std::mem;
use byteorder::{ReadBytesExt, BigEndian};
use crate::MidiError::UnsupportedMetaEvent;
use std::string::FromUtf8Error;
use std::convert::TryFrom;

#[derive(Debug)]
enum MidiError {
    Io(io::Error),
    InvalidVariableLengthQuantity,
    InvalidChunkHeader,
    InvalidFileHeader,
    InvalidMetaEvent,
    UnsupportedFormat,
    UnsupportedEvent(u8),
    UnsupportedMetaEvent(u8),
    Utf8(FromUtf8Error),
}

impl From<io::Error> for MidiError {
    fn from(err: io::Error) -> Self {
        MidiError::Io(err)
    }
}

impl From<FromUtf8Error> for MidiError {
    fn from(err: FromUtf8Error) -> Self {
        MidiError::Utf8(err)
    }
}

fn main() -> Result<(), MidiError> {
    let file_name = "assets/bach_847_format0.mid";

    let f = File::open(file_name)?;
    let mut f = BufReader::new(f);

    let header_header = read_chunk_header(&mut f)?;

    if header_header.chunk_type != ChunkType::Header {
        return Err(MidiError::InvalidChunkHeader);
    }
    let file_header = read_file_header(&mut f, header_header.length)?;

    let track_header = read_chunk_header(&mut f)?;

    loop {
        let mtrk_event = read_mtrk_event(&mut f)?;

        match mtrk_event {
            MtrkEvent{event: Event::Meta(MetaEvent::Text(msg)), ..} => println!("Text: {}", msg),
            MtrkEvent{event: Event::Meta(MetaEvent::CopyrightNotice(msg)), ..} => println!("Copyright: {}", msg),
            MtrkEvent{event: Event::Meta(MetaEvent::SequenceTrackName(msg)), ..} => println!("Name: {}", msg),
            MtrkEvent{event: Event::Meta(MetaEvent::EndOfTrack), ..} => break,
            MtrkEvent{event: Event::Meta(MetaEvent::TimeSignature{numerator, denominator, clocks_per_metronome, thirty_seconds_per_quarter}), ..} => println!("Time Signature: {}/{}, metronome: {}, quarter: {}", numerator, denominator, clocks_per_metronome, thirty_seconds_per_quarter),
            MtrkEvent{event: Event::Meta(MetaEvent::KeySignature{key_signature, mode}), ..} => println!("Key: {:?}", match mode {
                Mode::Major => Key::from(MajorKey::from_key_signature(key_signature)),
                Mode::Minor => Key::from(MinorKey::from_key_signature(key_signature)),
            }),
            _ => {},
        }
    }

    Ok(())
}

#[derive(Eq, PartialEq)]
enum ChunkType {
    Header,
    Track,
}

struct ChunkHeader {
    chunk_type: ChunkType,
    length: u32,
}

#[derive(Eq, PartialEq)]
enum FileFormat {
    Single,
    Simultaneous,
    Sequential,
}

enum SmpteFormat {
    Fps24,
    Fps25,
    Fps29,
    Fps30,
}

enum Division {
    TicksPerQuarterNote(u32),
    TicksPerFrame(SmpteFormat, u32),
}

struct FileHeader {
    file_format: FileFormat,
    number_of_chunks: u32,
    division: Division,
}

struct MtrkEvent {
    delta_time: u32,
    event: Event,
}

enum Event {
    Meta(MetaEvent),
}

enum MetaEvent {
    Text(String),
    CopyrightNotice(String),
    SequenceTrackName(String),
    EndOfTrack,
    TimeSignature {
        numerator: u32,
        denominator: u32,
        clocks_per_metronome: u32,
        thirty_seconds_per_quarter: u32,
    },
    KeySignature {
        key_signature: KeySignature,
        mode: Mode,
    }
}

enum KeySignature {
    Flat7,
    Flat6,
    Flat5,
    Flat4,
    Flat3,
    Flat2,
    Flat1,
    None,
    Sharp1,
    Sharp2,
    Sharp3,
    Sharp4,
    Sharp5,
    Sharp6,
    Sharp7,
}

impl TryFrom<i8> for KeySignature {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            -7 => Ok(KeySignature::Flat7),
            -6 => Ok(KeySignature::Flat6),
            -5 => Ok(KeySignature::Flat5),
            -4 => Ok(KeySignature::Flat4),
            -3 => Ok(KeySignature::Flat3),
            -2 => Ok(KeySignature::Flat2),
            -1 => Ok(KeySignature::Flat1),
            0 => Ok(KeySignature::None),
            1 => Ok(KeySignature::Sharp1),
            2 => Ok(KeySignature::Sharp2),
            3 => Ok(KeySignature::Sharp3),
            4 => Ok(KeySignature::Sharp4),
            5 => Ok(KeySignature::Sharp5),
            6 => Ok(KeySignature::Sharp6),
            7 => Ok(KeySignature::Sharp7),
            _ => Err(())
        }
    }
}

enum Mode {
    Major,
    Minor,
}

impl TryFrom<u8> for Mode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mode::Major),
            1 => Ok(Mode::Minor),
            _ => Err(())
        }
    }
}

#[derive(Debug)]
enum MajorKey {
    CFlat,
    GFlat,
    DFlat,
    AFlat,
    EFlat,
    BFlat,
    F,
    C,
    G,
    D,
    A,
    E,
    B,
    FSharp,
    CSharp,
}

impl MajorKey {
    fn from_key_signature(key_signature: KeySignature) -> MajorKey {
        match key_signature {
            KeySignature::Flat7 => MajorKey::CFlat,
            KeySignature::Flat6 => MajorKey::GFlat,
            KeySignature::Flat5 => MajorKey::DFlat,
            KeySignature::Flat4 => MajorKey::AFlat,
            KeySignature::Flat3 => MajorKey::EFlat,
            KeySignature::Flat2 => MajorKey::BFlat,
            KeySignature::Flat1 => MajorKey::F,
            KeySignature::None => MajorKey::C,
            KeySignature::Sharp1 => MajorKey::G,
            KeySignature::Sharp2 => MajorKey::D,
            KeySignature::Sharp3 => MajorKey::A,
            KeySignature::Sharp4 => MajorKey::E,
            KeySignature::Sharp5 => MajorKey::B,
            KeySignature::Sharp6 => MajorKey::FSharp,
            KeySignature::Sharp7 => MajorKey::CSharp,
        }
    }
}

#[derive(Debug)]
enum MinorKey {
    AFlat,
    EFlat,
    BFlat,
    F,
    C,
    G,
    D,
    A,
    E,
    B,
    FSharp,
    CSharp,
    GSharp,
    DSharp,
    ASharp,
}

impl MinorKey {
    fn from_key_signature(key_signature: KeySignature) -> MinorKey {
        match key_signature {
            KeySignature::Flat7 => MinorKey::AFlat,
            KeySignature::Flat6 => MinorKey::EFlat,
            KeySignature::Flat5 => MinorKey::BFlat,
            KeySignature::Flat4 => MinorKey::F,
            KeySignature::Flat3 => MinorKey::C,
            KeySignature::Flat2 => MinorKey::G,
            KeySignature::Flat1 => MinorKey::D,
            KeySignature::None => MinorKey::A,
            KeySignature::Sharp1 => MinorKey::E,
            KeySignature::Sharp2 => MinorKey::B,
            KeySignature::Sharp3 => MinorKey::FSharp,
            KeySignature::Sharp4 => MinorKey::CSharp,
            KeySignature::Sharp5 => MinorKey::GSharp,
            KeySignature::Sharp6 => MinorKey::DSharp,
            KeySignature::Sharp7 => MinorKey::ASharp,
        }
    }
}

#[derive(Debug)]
enum Key {
    Major(MajorKey),
    Minor(MinorKey),
}

impl From<MajorKey> for Key {
    fn from(key: MajorKey) -> Self {
        Key::Major(key)
    }
}

impl From<MinorKey> for Key {
    fn from(key: MinorKey) -> Self {
        Key::Minor(key)
    }
}

// struct Scale(Tonic, Mode);
//
// impl Scale {
//     fn from_key_signature(signs: CircleOfFifth, mode: Mode) -> Scale {
//         match (signs, mode) {
//             (CircleOfFifth::Flat7, Mode::Major) =>
//             (CircleOfFifth::Flat6, Mode::Major) => {}
//             (CircleOfFifth::Flat5, Mode::Major) => {}
//             (CircleOfFifth::Flat4, Mode::Major) => {}
//             (CircleOfFifth::Flat3, Mode::Major) => {}
//             (CircleOfFifth::Flat2, Mode::Major) => {}
//             (CircleOfFifth::Flat1, Mode::Major) => {}
//             (CircleOfFifth::None, Mode::Major) => {}
//             (CircleOfFifth::Sharp1, Mode::Major) => {}
//             (CircleOfFifth::Sharp2, Mode::Major) => {}
//             (CircleOfFifth::Sharp3, Mode::Major) => {}
//             (CircleOfFifth::Sharp4, Mode::Major) => {}
//             (CircleOfFifth::Sharp5, Mode::Major) => {}
//             (CircleOfFifth::Sharp6, Mode::Major) => {}
//             (CircleOfFifth::Sharp7, Mode::Major) => {}
//         }
//     }
// }

fn read_variable_length_quantity(reader: &mut impl Read) -> Result<u32, MidiError> {
    let mut quantity = 0;
    let mut bytes_read = 0;

    while bytes_read <= 4 {
        let curr = reader.read_u8()?;
        bytes_read += 1;

        quantity = (quantity << 7) | (curr & 0b0111_1111) as u32;

        if (curr & 0b1000_0000) == 0 {
            return Ok(quantity);
        }

    }

    Err(MidiError::InvalidVariableLengthQuantity)
}

fn read_chunk_header(reader: &mut impl Read) -> Result<ChunkHeader, MidiError> {
    const TYPE_SIZE: usize = 4;
    let mut type_buf = [0u8; TYPE_SIZE];

    const HEADER_TYPE: &[u8; 4] = b"MThd";
    const TRACK_TYPE: &[u8; 4] = b"MTrk";

    reader.read_exact(&mut type_buf)?;

    let chunk_type = match &type_buf {
        HEADER_TYPE => Some(ChunkType::Header),
        TRACK_TYPE => Some(ChunkType::Track),
        _ => None,
    }.ok_or(MidiError::InvalidChunkHeader)?;

    let length = reader.read_u32::<BigEndian>()?;

    if chunk_type == ChunkType::Header && length < 6 {
        Err(MidiError::InvalidChunkHeader)
    } else {
        Ok(ChunkHeader {
            chunk_type,
            length,
        })
    }
}

fn read_file_header<T>(reader: &mut T, length: u32) -> Result<FileHeader, MidiError>
    where
        T: Read + Seek,
{
    if length < 6 {
        return Err(MidiError::InvalidChunkHeader);
    }

    let file_format = reader.read_u16::<BigEndian>()?;

    let file_format = match file_format {
        0 => Some(FileFormat::Single),
        1 => Some(FileFormat::Simultaneous),
        2 => Some(FileFormat::Sequential),
        _ => None,
    }.ok_or(MidiError::InvalidFileHeader)?;

    let number_of_chunks = reader.read_u16::<BigEndian>()?;

    if file_format == FileFormat::Single && number_of_chunks != 1 {
        Err(MidiError::InvalidFileHeader)
    } else {
        let division = reader.read_u16::<BigEndian>()?;
        let division = parse_division(division)?;

        reader.seek(SeekFrom::Current((length - 6) as i64))?;

        Ok(FileHeader {
            file_format,
            number_of_chunks: number_of_chunks as u32,
            division,
        })
    }
}

fn parse_division(division: u16) -> Result<Division, MidiError> {
    if division & (1 << 15) > 0 {
        Err(MidiError::UnsupportedFormat)
    } else {
        Ok(Division::TicksPerQuarterNote(division as u32))
    }
}

fn read_mtrk_event(reader: &mut impl Read) -> Result<MtrkEvent, MidiError> {
    let delta_time = read_variable_length_quantity(reader)?;

    let code = reader.read_u8()?;

    match code {
        0xff => read_meta_event(reader).map(|x| MtrkEvent {
            delta_time,
            event: Event::Meta(x),
        }),
        _ => Err(MidiError::UnsupportedEvent(code)),
    }
}

fn read_meta_event(reader: &mut impl Read) -> Result<MetaEvent, MidiError> {
    let code = reader.read_u8()?;

    match code {
        0x01 => read_text(reader).map(|x| MetaEvent::Text(x)),
        0x02 => read_text(reader).map(|x| MetaEvent::CopyrightNotice(x)),
        0x03 => read_text(reader).map(|x| MetaEvent::SequenceTrackName(x)),
        0x2f => {
            if reader.read_u8()? == 0x00 {
                Ok(MetaEvent::EndOfTrack)
            } else {
                Err(MidiError::InvalidMetaEvent)
            }
        },
        0x58 => {
            if reader.read_u8()? == 0x04 {
                let numerator = reader.read_u8()?;
                let denominator = reader.read_u8()?;

                let clocks_per_metronome = reader.read_u8()?;
                let thirty_seconds_per_quarter = reader.read_u8()?;

                Ok(MetaEvent::TimeSignature {
                    numerator: numerator as u32,
                    denominator: 1 << (denominator as u32),
                    clocks_per_metronome: clocks_per_metronome as u32,
                    thirty_seconds_per_quarter: thirty_seconds_per_quarter as u32,
                })
            } else {
                Err(MidiError::InvalidMetaEvent)
            }
        },
        0x59 => {
            if reader.read_u8()? == 0x02 {
                let sf = reader.read_i8()?;
                let sf = KeySignature::try_from(sf).map_err(|_| MidiError::InvalidMetaEvent)?;
                let mi = reader.read_u8()?;
                let mi = Mode::try_from(mi).map_err(|_| MidiError::InvalidMetaEvent)?;

                Ok(MetaEvent::KeySignature {
                    key_signature: sf,
                    mode: mi,
                })
            } else {
                Err(MidiError::InvalidMetaEvent)
            }
        },
        _ => Err(UnsupportedMetaEvent(code))
    }
}

fn read_text(reader: &mut impl Read) -> Result<String, MidiError> {
    let length = read_variable_length_quantity(reader)?;

    let mut buf = vec![0u8; length as usize];
    reader.read_exact(&mut buf)?;

    let str = String::from_utf8_lossy(&buf).into_owned();

    Ok(str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    macro_rules! read_variable_length_quantity_valid_values_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, input) = $value;
                    let mut cursor = Cursor::new(&input);
                    let actual = read_variable_length_quantity(&mut cursor).unwrap();
                    assert_eq!(expected, actual);
                }
            )*
        }
    }

    read_variable_length_quantity_valid_values_tests! {
        read_variable_length_quantity_0: (0x00000000, [0x00]),
        read_variable_length_quantity_1: (0x00000040, [0x40]),
        read_variable_length_quantity_2: (0x0000007f, [0x7f]),
        read_variable_length_quantity_3: (0x00000080, [0x81, 0x00]),
        read_variable_length_quantity_4: (0x00002000, [0xc0, 0x00]),
        read_variable_length_quantity_5: (0x00003fff, [0xff, 0x7f]),
        read_variable_length_quantity_6: (0x00004000, [0x81, 0x80, 0x00]),
        read_variable_length_quantity_7: (0x00100000, [0xc0, 0x80, 0x00]),
        read_variable_length_quantity_8: (0x001fffff, [0xff, 0xff, 0x7f]),
        read_variable_length_quantity_9: (0x00200000, [0x81, 0x80, 0x80, 00]),
        read_variable_length_quantity_10: (0x08000000, [0xc0, 0x80, 0x80, 0x00]),
        read_variable_length_quantity_11: (0x0fffffff, [0xff, 0xff, 0xff, 0x7f]),
    }
}