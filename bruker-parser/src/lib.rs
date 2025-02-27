#![allow(unused)]
use app_core::string_error::ErrorStringExt;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
    path::Path,
};

const HEADER_SIZE_BYTES: usize = 504;
const META_BLOCK_SIZE: usize = 12;
const INITIAL_CURSOR_POS: usize = 24;

/// Definition of a data block (a range in the bytes of the Opus file) that
/// holds a particular kind (`BlockKind`) of data.
#[derive(Debug, Clone, Copy)]
struct BlockDefinition {
    kind: BlockKind,
    offset: usize,
    size: usize,
}

/// Enum used to differentiate the different kinds of data stored in the blocks
/// that make up an Opus file.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BlockKind {
    AB,
    ABDataParameter,
    Acquisition,
    AcquisitionRf,
    CurveFit,
    FourierTransformation,
    FourierTransformationRf,
    History,
    IgRf,
    IgRfDataParameter,
    IgSm,
    IgSmDataParameter,
    InfoBlock,
    Instrument,
    InstrumentRf,
    IntegrationMethod,
    Optik,
    OptikRf,
    PhRf,
    PhRfDataParameter,
    PhSm,
    PhSmDataParameter,
    PwRf,
    PwRfDataParameter,
    PwSm,
    PwSmDataParameter,
    Sample,
    ScRf,
    ScRfDataParameter,
    ScSm,
    ScSmDataParameter,
    Signature,
    TextInformation,
    Invalid,
}

impl From<(u8, u8)> for BlockKind {
    // Parse two bytes, which encode data type and channel type,
    // into a `BlockKind`.
    fn from(value: (u8, u8)) -> Self {
        use BlockKind as K;
        let (data_type, channel_type) = value;
        match (data_type, channel_type) {
            // data_type = 0 => text data
            (0, 8) => K::InfoBlock,
            (0, 104) => K::History,
            (0, 152) => K::CurveFit,
            (0, 168) => K::Signature,
            (0, 240) => K::IntegrationMethod,
            (0, _) => K::TextInformation,
            (7, 4) => K::ScSm,
            (7, 8) => K::IgSm,
            (7, 12) => K::PhSm,
            (7, 56) => K::PwSm,
            (11, 4) => K::ScRf,
            (11, 8) => K::IgRf,
            (11, 12) => K::PhRf,
            (11, 56) => K::PwRf,
            (15, _) => K::AB,
            (23, 4) => K::ScSmDataParameter,
            (23, 8) => K::IgSmDataParameter,
            (23, 12) => K::PhSmDataParameter,
            (23, 56) => K::PwSmDataParameter,
            (27, 4) => K::ScRfDataParameter,
            (27, 8) => K::IgRfDataParameter,
            (27, 12) => K::PhRfDataParameter,
            (27, 56) => K::PwRfDataParameter,
            (31, _) => K::ABDataParameter,
            (32, _) => K::Instrument,
            (40, _) => K::InstrumentRf,
            (48, _) => K::Acquisition,
            (56, _) => K::AcquisitionRf,
            (64, _) => K::FourierTransformation,
            (72, _) => K::FourierTransformationRf,
            (96, _) => K::Optik,
            (104, _) => K::OptikRf,
            (160, _) => K::Sample,
            _ => K::Invalid,
        }
    }
}

impl BlockDefinition {
    /// Read a single block definition from a slice of bytes, starting at
    /// position `cursor`.
    fn from_buffer(buf: &[u8; HEADER_SIZE_BYTES], cursor: usize) -> Self {
        // Read data and channel type and parse them into a BlockKind.
        let data_type = buf[cursor];
        let channel_type = if data_type == 0 {
            buf[cursor + 2]
        } else {
            buf[cursor + 1]
        };
        let kind: BlockKind = (data_type, channel_type).into();

        // Read chunk size and offset of data block holding data.
        let size = bytes_to_u32(&buf[cursor + 4..cursor + 8]) as usize;
        let offset = bytes_to_u32(&buf[cursor + 8..cursor + 12]) as usize;

        Self { kind, offset, size }
    }

    fn read_from_file(&self, file: &mut File) -> Result<Vec<f32>, String> {
        let mut bytes = Vec::new();
        file.seek(SeekFrom::Start(self.offset as u64))
            .err_to_string("unable to seek through file")?;
        // TODO: quadrupeling the size is maybe not needed for all data types.
        file.take((self.size * 4) as u64).read_to_end(&mut bytes);
        let mut f32_buf = [0u8; 4];
        Ok(bytes
            .chunks_exact(4)
            .map(|bytes| {
                bytes.iter().enumerate().for_each(|(i, x)| f32_buf[i] = *x);
                f32::from_le_bytes(f32_buf)
            })
            .collect())
    }
}

/// Read all available block definitions from a slice of bytes.
fn read_block_definitions(buf: &[u8; HEADER_SIZE_BYTES]) -> Vec<BlockDefinition> {
    let mut blks = Vec::new();
    let mut cursor = INITIAL_CURSOR_POS;
    while cursor < (HEADER_SIZE_BYTES - META_BLOCK_SIZE) {
        let blk = BlockDefinition::from_buffer(&buf, cursor);

        if blk.offset == 0 {
            break;
        }

        blks.push(blk);

        cursor += META_BLOCK_SIZE;
    }
    blks
}

fn parse_bruker_file(path: &Path) -> Result<(), String> {
    let mut file = File::open(path).err_to_string("failed to open file")?;
    let mut header_buf = [0u8; HEADER_SIZE_BYTES];
    file.read_exact(&mut header_buf)
        .err_to_string("failed to read header")?;

    // TODO: to prevent trying to load huge files, check
    // block sizes and bail out if one block is too large.
    let blks = read_block_definitions(&header_buf);

    let (Some(absorbance_definition), Some(absorbance_param_definition)) =
        blks.iter().fold((None, None), |mut acc, b| match b.kind {
            BlockKind::AB => {
                acc.0 = Some(b);
                acc
            }
            BlockKind::ABDataParameter => {
                acc.1 = Some(b);
                acc
            }
            _ => acc,
        })
    else {
        return Err("file does not contain absorbance data".to_string());
    };

    dbg!(absorbance_definition);
    dbg!(absorbance_param_definition);

    let data = absorbance_definition.read_from_file(&mut file);

    let mut out = File::create("parsed.dat").unwrap();
    for x in data.unwrap().iter() {
        write!(out, "{}\n", x);
    }

    Ok(())
}

// ------------------------------- Helpers -----------------------------------

/// Turn a slice of bytes into a u32. Slice must hold exactly 4 bytes.
fn bytes_to_u32(bytes: &[u8]) -> u32 {
    assert!(bytes.len() == 4);
    bytes
        .iter()
        .take(4)
        .enumerate()
        .map(|(i, x)| (*x as u32) << i * 8)
        .sum()
}

// def read_chunk(data: bytes, block_meta):
//     p1 = block_meta.offset
//     p2 = p1 + 4 * block_meta.chunk_size
//     return data[p1:p2]

// -------------------------------- Tests ------------------------------------

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test() {
        let path = PathBuf::from("test-data.0");
        parse_bruker_file(&path);

        // let bytes = [0x24, 0x7b, 0x00, 0x00];
        // dbg!(bytes_to_int(&bytes));
    }
}
