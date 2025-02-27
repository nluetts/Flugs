#![allow(unused)]
use app_core::string_error::ErrorStringExt;
use std::{
    collections::HashMap,
    fmt::Write as fmtWrite,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
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
        let size = u32::from_le_bytes([
            buf[cursor + 4],
            buf[cursor + 5],
            buf[cursor + 6],
            buf[cursor + 7],
        ]) as usize;
        let offset = u32::from_le_bytes([
            buf[cursor + 8],
            buf[cursor + 9],
            buf[cursor + 10],
            buf[cursor + 11],
        ]) as usize;

        Self { kind, offset, size }
    }

    /// Read a data block from the file, defined by self. This is currently
    /// only tested on AB blocks and may fail on other blocks.
    fn read_block_data_from_file(&self, file: &mut File) -> Result<Vec<f32>, String> {
        let mut bytes = Vec::new();
        file.seek(SeekFrom::Start(self.offset as u64))
            .err_to_string("unable to seek through file")?;
        file.take((self.size * 4) as u64).read_to_end(&mut bytes);

        let mut res = Vec::with_capacity(self.size);
        let mut i = 0;

        while i + 4 < bytes.len() {
            res.push(f32::from_le_bytes([
                bytes[i],
                bytes[i + 1],
                bytes[i + 2],
                bytes[i + 3],
            ]));
            i += 4;
        }
        Ok(res)
    }

    fn read_params_from_file(&self, file: &mut File) -> Result<HashMap<String, OpusParam>, String> {
        let mut bytes = Vec::new();
        file.seek(SeekFrom::Start(self.offset as u64))
            .err_to_string("unable to seek through file")?;
        file.take((self.size * 4) as u64).read_to_end(&mut bytes);

        let mut params = HashMap::new();
        let mut i = 0;

        // Seven bytes define the parameter, at least one byte needed for data,
        // thus we need at least 8 bytes in the last iteration.
        while i + 8 < bytes.len() {
            let param_name: String = bytes[i..i + 3].iter().map(|b| *b as char).collect();

            if param_name.as_str() == "END" {
                break;
            }

            let param_kind = u16::from_le_bytes([bytes[i + 4], bytes[i + 5]]);
            let param_size = u16::from_le_bytes([bytes[i + 6], bytes[i + 7]]);

            let end_idx = i + 8 + 2 * (param_size as usize);
            // Make sure we do not access out of bounds.
            if end_idx > bytes.len() {
                break;
            }

            let param_bytes = &bytes[i + 8..end_idx];

            use OpusParam as O;
            let param_value = match param_kind {
                0 => O::Integer(u32::from_le_bytes([
                    param_bytes[0],
                    param_bytes[1],
                    param_bytes[2],
                    param_bytes[3],
                ])),

                1 => O::Float(f64::from_le_bytes([
                    param_bytes[0],
                    param_bytes[1],
                    param_bytes[2],
                    param_bytes[3],
                    param_bytes[4],
                    param_bytes[5],
                    param_bytes[6],
                    param_bytes[7],
                ])),

                2 | 3 | 4 => O::Text(
                    param_bytes
                        .iter()
                        .filter(|&&b| b != 0)
                        .map(|b| *b as char)
                        .collect(),
                ),

                _ => return Err("failed to parse parameter, invalid type".to_string()),
            };

            params.insert(param_name, param_value);
            i += 8 + 2 * (param_size as usize);
        }

        Ok(params)
    }
}

#[derive(Debug, Clone)]
enum OpusParam {
    Integer(u32),
    Float(f64),
    Text(String),
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

#[derive(Debug, Clone)]
pub struct OpusAbsorbanceData {
    pub wavenumber: Vec<f64>,
    pub absorbance: Vec<f64>,
}

impl OpusAbsorbanceData {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).err_to_string("failed to open file")?;
        let mut header_buf = [0u8; HEADER_SIZE_BYTES];
        file.read_exact(&mut header_buf)
            .err_to_string("failed to read meta data")?;

        let blks = read_block_definitions(&header_buf);

        let (Some(absorbance_definition), Some(absorbance_param_definition)) =
            // Collect absorbance data block definition and data parameter definition
            // into tuple, or return error if these definitions are not found.
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

        let absorbance = absorbance_definition
            .read_block_data_from_file(&mut file)?
            .iter()
            .map(|x| *x as f64)
            .collect();
        let params = absorbance_param_definition.read_params_from_file(&mut file)?;

        use OpusParam as O;
        let Some(O::Float(xmin)) = params.get("LXV").cloned() else {
            return Err("no data on x-range found".to_string());
        };
        let Some(O::Float(xmax)) = params.get("FXV").cloned() else {
            return Err("no data on x-range found".to_string());
        };

        let step = (xmax - xmin) / (absorbance_definition.size as f64);

        let mut wavenumber = Vec::with_capacity(absorbance_definition.size);
        wavenumber.push(xmax);

        // Create the wavenumber data. Keep in mind that in Opus higher
        // wavenumber is left, lower wavenumber right.
        let mut x = xmax;
        while x >= xmin {
            x -= step;
            wavenumber.push(x);
        }

        Ok(OpusAbsorbanceData {
            wavenumber,
            absorbance,
        })
    }

    fn to_csv(&self, path: &Path) -> Result<(), String> {
        let mut output_buf = String::with_capacity(self.absorbance.len() * 40);

        for (x, y) in self.wavenumber.iter().zip(self.absorbance.iter()) {
            write!(output_buf, "{},{}\n", x, y);
        }

        let mut file = File::create(path).err_to_string("could not create file to save to csv")?;
        file.write(output_buf.as_bytes())
            .err_to_string("failed to write csv data to file")?;

        Ok(())
    }
}

// -------------------------------- Tests ------------------------------------

#[cfg(test)]
mod test {
    use std::{path::PathBuf, time::SystemTime};

    use super::*;

    #[test]
    fn test() {
        let path = PathBuf::from_str("test-data.0").unwrap();
        let data = OpusAbsorbanceData::from_path(&path).unwrap();

        let path = PathBuf::from_str("absorbance.dat").unwrap();
        data.to_csv(&path);

        // let bytes = [0x24, 0x7b, 0x00, 0x00];
        // dbg!(bytes_to_int(&bytes));
    }
}
