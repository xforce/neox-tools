use byteorder::{LittleEndian, ReadBytesExt};
use clap::{App, Arg};
use log::{debug, error, info, trace, warn};
use std::io::{BufReader, Read, Seek};

fn is_eof<T>(reader: &mut std::io::BufReader<T>) -> std::io::Result<bool>
where
    T: std::io::Read,
    T: std::io::Seek,
{
    let mut buffer = vec![0; 0x1];
    let eof_check = reader.read_exact(&mut buffer);
    match eof_check {
        Ok(_) => {
            reader.seek(std::io::SeekFrom::Current(-1))?;
            Ok(false)
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Ok(true)
            } else {
                Err(e)
            }
        }
    }
}

#[derive(Debug)]
enum NeoXIndex2Error {
    InvalidSize,
    IoError(std::io::Error),
    UnkownCompressType,
    UnknownEncryptType,
    DecompressFailedLZ4,
}

impl std::fmt::Display for NeoXIndex2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for NeoXIndex2Error {
    fn from(e: std::io::Error) -> Self {
        NeoXIndex2Error::IoError(e)
    }
}

#[derive(Debug)]
enum NeoXIndex2CompressType {
    None,
    LZ4,
    ZLib,
}

#[derive(Debug)]
enum NeoXIndex2EncryptType {
    None,
    RC4,
    Simple,
}

#[derive(Debug)]
struct NeoXIndex2 {
    name_hash: u64,
    offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    compression_type: NeoXIndex2CompressType,
    encrypt_type: NeoXIndex2EncryptType,
    some_flag: u8,
}

impl NeoXIndex2 {
    pub fn name_hash(&self) -> u64 {
        self.name_hash
    }

    pub fn from_slice(slice: &mut [u8]) -> Result<Self, NeoXIndex2Error> {
        let mut slice = slice.as_ref();
        if slice.len() != 0x28 {
            Err(NeoXIndex2Error::InvalidSize)
        } else {
            let name_hash = slice.read_u64::<LittleEndian>()?;
            let offset = slice.read_u32::<LittleEndian>()?;
            let compressed_size = slice.read_u32::<LittleEndian>()?;
            let uncompressed_size = slice.read_u32::<LittleEndian>()?;

            let _field_14 = slice.read_u32::<LittleEndian>()?;
            let _field_18 = slice.read_u64::<LittleEndian>()?;
            let _field_20 = slice.read_i8()?;
            let _field_21 = slice.read_i8()?;
            let _field_22 = slice.read_i8()?;
            let _field_23 = slice.read_i8()?;
            let compress_type = slice.read_u16::<LittleEndian>()?;
            let encrypt_type = slice.read_u8()?;
            let field_27 = slice.read_u8()?;

            Ok(NeoXIndex2 {
                name_hash,
                offset,
                compressed_size,
                uncompressed_size,
                compression_type: match compress_type {
                    0 => NeoXIndex2CompressType::None,
                    1 => NeoXIndex2CompressType::ZLib,
                    2 => NeoXIndex2CompressType::LZ4,
                    _ => return Err(NeoXIndex2Error::UnkownCompressType),
                },
                encrypt_type: match encrypt_type {
                    0 => NeoXIndex2EncryptType::None,
                    1 => NeoXIndex2EncryptType::Simple,
                    2 => NeoXIndex2EncryptType::RC4,
                    _ => return Err(NeoXIndex2Error::UnknownEncryptType),
                },
                some_flag: field_27,
            })
        }
    }

    pub fn read_content_from_buffer<T>(
        &self,
        reader: &mut std::io::BufReader<T>,
    ) -> Result<Vec<u8>, NeoXIndex2Error>
    where
        T: std::io::Read,
        T: std::io::Seek,
    {
        reader.seek(std::io::SeekFrom::Start(
            self.offset as u64 | (self.some_flag as u64) << 20,
        ))?;

        let mut buffer = vec![0; self.compressed_size as usize];
        trace!("Read NeoX source buffer");
        reader.read_exact(&mut buffer)?;

        trace!("Handle Encryption of type {:?}", self.encrypt_type);
        let unencrypted_buffer = match self.encrypt_type {
            NeoXIndex2EncryptType::None => buffer,
            NeoXIndex2EncryptType::RC4 => unimplemented!("RC4 is currently not supported"),
            NeoXIndex2EncryptType::Simple => {
                unimplemented!("Simple encrypt is currenlty not support")
            }
        };

        trace!("Handle Compression of type {:?}", self.compression_type);
        let uncompressed_buffer = match self.compression_type {
            NeoXIndex2CompressType::None => unencrypted_buffer,
            NeoXIndex2CompressType::LZ4 => {
                let mut decompressed = Vec::new();
                let len = compress::lz4::decode_block(&unencrypted_buffer, &mut decompressed);
                if len < 1 {
                    return Err(NeoXIndex2Error::DecompressFailedLZ4);
                }
                decompressed
            }
            NeoXIndex2CompressType::ZLib => unimplemented!("ZLib is currently not supported"),
        };

        Ok(uncompressed_buffer)
    }
}

#[derive(Debug)]
enum Npk2Error {
    IoError(std::io::Error),
    InvalidHeader,
    IndexError(NeoXIndex2Error),
}

impl std::fmt::Display for Npk2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for Npk2Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<NeoXIndex2Error> for Npk2Error {
    fn from(e: NeoXIndex2Error) -> Self {
        Self::IndexError(e)
    }
}

struct Npk2Header {
    file_count: u32,
    large_file_index_offset: u32,
    index_offset: u32,
    index_size: u32,
    field_28: u32,
}

impl Npk2Header {
    pub fn index_offset(&self) -> u64 {
        self.index_offset as u64 | ((self.large_file_index_offset as u64) << 0x20)
    }

    pub fn size_of_index_entry(&self) -> u64 {
        0x28
    }

    pub fn index_buffer_size(&self) -> usize {
        //
        self.file_count as usize * self.size_of_index_entry() as usize
    }
}

struct Npk2Reader {
    file: std::fs::File,
    header: Npk2Header,
    indices: Vec<NeoXIndex2>,
}

impl Npk2Reader {
    fn read_header<T>(reader: &mut std::io::BufReader<T>) -> Result<Npk2Header, Npk2Error>
    where
        T: std::io::Read,
        T: std::io::Seek,
    {
        //
        let magic = reader.read_u32::<LittleEndian>()?;
        if magic != 0x4B50584E {
            return Err(Npk2Error::InvalidHeader);
        }

        let mut file_count = reader.read_u32::<LittleEndian>()?;

        let large_file_index_offset = reader.read_u32::<LittleEndian>()?;
        let _var2 = reader.read_u32::<LittleEndian>()?; // IGNORED
        let _var3 = reader.read_u32::<LittleEndian>()?; // IGNORED
        let index_offset = reader.read_u32::<LittleEndian>()?;

        let index_size = reader.read_u32::<LittleEndian>()?;
        let mut field_28 = reader.read_u32::<LittleEndian>()?; // DATA END MAYBE?

        // info!("File Size:\t{}", size);
        // info!("NPK Magic:\t{:X}", magic);
        // info!("NPK File Count:\t{}", file_count);
        // info!("NPK Is Large File:\t{}", large_file_index_offset > 0);
        // info!("NPK Index Offset:\t{}", index_offset);

        // TODO(alexander): Figure out what this actually does and why we need it
        // in Eve Echoes NPKs we usually run into the first case, where we change field 28
        let v5 = ((field_28 as i64 - index_size as i64) >> 3) as u64 / 5; // No idea why divide by 5, but it is a thing :)
        if v5 >= file_count as u64 {
            if v5 > file_count as u64 {
                warn!("Handle unknown special case! Changing field 28");
                field_28 = index_size + 0x28 * file_count;
            }
        } else {
            warn!("Unknown limit thing");
            file_count = file_count - v5 as u32;
        }

        //
        //
        Ok(Npk2Header {
            file_count,
            large_file_index_offset,
            index_offset,
            index_size,
            field_28,
        })
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Npk2Error> {
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(&file);
        let header = Self::read_header(&mut reader)?;

        Ok(Npk2Reader {
            file,
            header,
            indices: Vec::new(),
        })
    }

    pub fn open(&mut self) -> Result<(), Npk2Error> {
        let mut reader = BufReader::new(&self.file);

        // info!("File Size:\t{}", size);
        // info!("NPK Magic:\t{:X}", magic);
        // info!("NPK File Count:\t{}", file_count);
        // info!("NPK Is Large File:\t{}", large_file_index_offset > 0);
        // info!("NPK Index Offset:\t{}", index_offset);

        let pos = reader.seek(std::io::SeekFrom::Start(self.header.index_offset()))?;
        assert!(pos == self.header.index_offset());

        // Read the Index Buffer into Memory
        let mut buffer = vec![0; self.header.index_buffer_size()];
        reader.read_exact(&mut buffer)?;

        if !is_eof(&mut reader)? && self.header.index_size != self.header.field_28 {
            unimplemented!("Handle this type of NPK file, embedded file names :)");
        // debug!(
        //     "Reading more stuff...no idea what :) {} != {}",
        //     index_size, field_28
        // );
        // let mut buffer = vec![0; 0x100];
        // trace!("Reading {} bytes", 0x100);
        // reader.read_exact(&mut buffer)?;
        } else {
            // Load all the indices from the NPK File
            let mut buffer_cursor = std::io::Cursor::new(buffer);
            let mut sub_buffer = vec![0; 0x28 as usize];
            let mut index_buffer = buffer_cursor.read_exact(&mut sub_buffer);
            while index_buffer.is_ok() {
                index_buffer = buffer_cursor.read_exact(&mut sub_buffer);
                let index = NeoXIndex2::from_slice(sub_buffer.as_mut_slice())?;
                self.indices.push(index);
            }
        }

        Ok(())
    }

    pub fn indices(&self) -> &Vec<NeoXIndex2> {
        &self.indices
    }

    pub fn read_content_for_index(&self, index: &NeoXIndex2) -> Result<Vec<u8>, Npk2Error> {
        let mut reader = BufReader::new(&self.file);
        Ok(index.read_content_from_buffer(&mut reader)?)
    }
}

fn main() -> Result<(), Npk2Error> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    // let string = "redirect.nxs";
    // let v1 = Hash32::hash_with_seed(string, 0x9747B28C);
    // let v2 = Hash32::hash_with_seed(string, 0xC82B7479);

    // println!("{:X}", (v1 as u64 | (v2 as u64) << 0x20) as u64);

    let matches = App::new("NeoX NPK Tool")
        .version("1.0")
        .author("Alexander Guettler <alexander@guettler.io>")
        .arg(
            Arg::with_name("MODE")
                .possible_values(&["x", "p"]) // TODO(alexander): Add some kind of info mode, print file count etc.
                .about("Specifies whether to extract (x) or pack (p) the specified directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("INPUT")
                .about("The NPK file to be operated on")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("DIR")
                .value_name("DIR")
                .about("The directory where this NPK file should be extracted to")
                .default_value("out")
                .index(3)
                .takes_value(true),
        )
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();
    let mut npk_file = Npk2Reader::new(input_file)?;
    npk_file.open()?;

    let mode = matches.value_of("MODE").unwrap();

    match mode {
        "x" => {
            let output_directory = std::path::Path::new(matches.value_of("DIR").unwrap());
            let output_sub_directory = match std::path::Path::new(&input_file).file_stem() {
                Some(v) => v.to_str().unwrap_or(""),
                None => "",
            };
            let output_directory = output_directory.join(output_sub_directory);
            std::fs::create_dir_all(&output_directory)?;

            for index in npk_file.indices() {
                debug!("Reading Index {:?}", index);
                //
                let content = npk_file.read_content_for_index(index)?;
                let result = tree_magic::from_u8(&content);
                let extension = match result.as_str() {
                    "text/plain" => "txt",
                    "application/octet-stream" => {
                        let mut rdr = std::io::Cursor::new(&content);
                        let magic = rdr.read_u32::<LittleEndian>();
                        match magic {
                            Ok(magic) => {
                                // Detect NXS and stuff, which is a NeoX Script File
                                if magic == 0x041D {
                                    "nxs"
                                } else if magic & 0xFFFF == 0x041D {
                                    "nxs"
                                } else {
                                    "dat"
                                }
                            }
                            Err(_) => "dat",
                        }
                    }
                    "application/x-executable" => "exe",
                    "application/x-cpio" => "cpio",
                    "image/ktx" => "ktx",
                    "image/png" => "png",
                    "image/x-dds" => "dds",
                    "image/x-win-bitmap" => "bmp",
                    "application/xml" => "xml",
                    "text/x-matlab" => "mat", // Maybe m instead?
                    "application/x-apple-systemprofiler+xml" => "xml",
                    "text/x-modelica" => "mo",
                    "text/x-csrc" => "c",
                    "font/ttf" => "ttf",
                    "image/bmp" => "bmp",
                    "application/zip" => "zip",
                    "image/jpeg" => "jpg",
                    _ => {
                        error!("Unhandled mime type {}", result);
                        "dat"
                    }
                };

                let out_file =
                    output_directory.join(format!("{:X}.{}", index.name_hash(), extension));
                info!(
                    "Writing {} bytes to {}",
                    bytesize::ByteSize(content.len() as u64),
                    out_file.as_path().to_str().unwrap()
                );
                std::fs::write(out_file, &content)?;
            }
        }
        "p" => unimplemented!("Packing is currently not supported"),
        _ => {}
    }

    info!("Done.");

    Ok(())
}
