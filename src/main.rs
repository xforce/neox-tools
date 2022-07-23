use byteorder::{LittleEndian, ReadBytesExt};
use clap::{App, Arg};
use log::{debug, info, trace};
use std::io::{BufRead, BufReader, Read, Seek};

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
enum NeoXIndexError {
    // InvalidSize, TODO(alexander): We do want to handle this at some point
    IoError(std::io::Error),
    UnkownCompressType,
    UnknownEncryptType,
    DecompressFailedLZ4,
    DecompressFailedZLib,
}

impl std::fmt::Display for NeoXIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for NeoXIndexError {
    fn from(e: std::io::Error) -> Self {
        NeoXIndexError::IoError(e)
    }
}

#[derive(Debug, Clone, Copy)]
enum NeoXIndex2CompressType {
    None,
    LZ4,
    ZLib,
}

#[derive(Debug, Clone, Copy)]
enum NeoXIndex2EncryptType {
    None,
    RC4,
    Simple,
}

#[derive(Debug)]
struct NeoXIndex1 {
    name_hash: u64,
    offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    compression_type: NeoXIndex2CompressType,
    encrypt_type: NeoXIndex2EncryptType,
    large_file_offset: u8,
}

impl NeoXIndex1 {
    pub fn from_slice(slice: &mut [u8]) -> Result<Self, NeoXIndexError> {
        let mut slice = slice.as_ref();
        let name_hash = slice.read_u32::<LittleEndian>()?;
        let offset = slice.read_u32::<LittleEndian>()?;
        let compressed_size = slice.read_u32::<LittleEndian>()?;
        let uncompressed_size = slice.read_u32::<LittleEndian>()?;
        let _field_14 = slice.read_u64::<LittleEndian>()?;

        let compress_type = slice.read_u16::<LittleEndian>()?;
        let encrypt_type = slice.read_u8()?;
        let large_file_offset = slice.read_u8()?;

        Ok(NeoXIndex1 {
            name_hash: name_hash as u64,
            offset,
            compressed_size,
            uncompressed_size,
            compression_type: match compress_type {
                0 => NeoXIndex2CompressType::None,
                1 => NeoXIndex2CompressType::ZLib,
                2 => NeoXIndex2CompressType::LZ4,
                _ => return Err(NeoXIndexError::UnkownCompressType),
            },
            encrypt_type: match encrypt_type {
                0 => NeoXIndex2EncryptType::None,
                1 => NeoXIndex2EncryptType::Simple,
                2 => NeoXIndex2EncryptType::RC4,
                _ => return Err(NeoXIndexError::UnknownEncryptType),
            },
            large_file_offset,
        })
    }
}

#[derive(Debug)]
struct NeoXIndex1_2 {
    name_hash: u64,
    offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    compression_type: NeoXIndex2CompressType,
    encrypt_type: NeoXIndex2EncryptType,
    large_file_offset: u8,
}

impl NeoXIndex1_2 {
    pub fn from_slice(slice: &mut [u8]) -> Result<Self, NeoXIndexError> {
        let mut slice = slice.as_ref();
        let name_hash = slice.read_u64::<LittleEndian>()?; // +0
        let offset = slice.read_u32::<LittleEndian>()?; // +4
        let compressed_size = slice.read_u32::<LittleEndian>()?; // +C
        let uncompressed_size = slice.read_u32::<LittleEndian>()?; // +10
        let _field_14 = slice.read_u64::<LittleEndian>()?; // +14

        let compress_type = slice.read_u16::<LittleEndian>()?; // +1C
        let encrypt_type = slice.read_u8()?; // +1E
        let large_file_offset = slice.read_u8()?; // +1F

        Ok(NeoXIndex1_2 {
            name_hash,
            offset,
            compressed_size,
            uncompressed_size,
            compression_type: match compress_type {
                0 => NeoXIndex2CompressType::None,
                1 => NeoXIndex2CompressType::ZLib,
                2 => NeoXIndex2CompressType::LZ4,
                _ => return Err(NeoXIndexError::UnkownCompressType),
            },
            encrypt_type: match encrypt_type {
                0 => NeoXIndex2EncryptType::None,
                1 => NeoXIndex2EncryptType::Simple,
                2 => NeoXIndex2EncryptType::RC4,
                _ => return Err(NeoXIndexError::UnknownEncryptType),
            },
            large_file_offset,
        })
    }
}

#[derive(Debug)]
struct NeoXIndex1_32 {
    name_hash: u64,
    offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    compression_type: NeoXIndex2CompressType,
    encrypt_type: NeoXIndex2EncryptType,
    large_file_offset: u8,
}

impl NeoXIndex1_32 {
    pub fn from_slice(slice: &mut [u8]) -> Result<Self, NeoXIndexError> {
        let mut slice = slice.as_ref();
        let name_hash = slice.read_u32::<LittleEndian>()?; // +0
        let mut offset = slice.read_u32::<LittleEndian>()?; // +4
        let compressed_size = slice.read_u32::<LittleEndian>()?; // +8
        let uncompressed_size = slice.read_u32::<LittleEndian>()?; // +C
        let _field_10 = slice.read_u32::<LittleEndian>()?; // +10
        let _field_14 = slice.read_u32::<LittleEndian>()?; // +14

        let mut compress_type = slice.read_u32::<LittleEndian>()?; // +18
        let encrypt_type = 0;

        if compress_type >= 62 {
            offset ^= (uncompressed_size + 99) ^ 0x85F6F276;
            compress_type = compress_type - 62;
        }

        Ok(Self {
            name_hash: name_hash as u64,
            offset: offset as u32,
            compressed_size,
            uncompressed_size,
            compression_type: match compress_type {
                0 => NeoXIndex2CompressType::None,
                1 => NeoXIndex2CompressType::ZLib,
                2 => NeoXIndex2CompressType::LZ4,
                _ => return Err(NeoXIndexError::UnkownCompressType),
            },
            encrypt_type: match encrypt_type {
                0 => NeoXIndex2EncryptType::None,
                1 => NeoXIndex2EncryptType::Simple,
                2 => NeoXIndex2EncryptType::RC4,
                _ => return Err(NeoXIndexError::UnknownEncryptType),
            },
            large_file_offset: 0,
        })
    }
}

#[derive(Debug)]
struct NeoXIndex2 {
    name_hash: u64,
    offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    compression_type: NeoXIndex2CompressType,
    encrypt_type: NeoXIndex2EncryptType,
    large_file_offset: u8,
}

impl NeoXIndex2 {
    pub fn name_hash(&self) -> u64 {
        self.name_hash
    }

    pub fn from_slice(slice: &mut [u8]) -> Result<Self, NeoXIndexError> {
        let mut slice = slice.as_ref();
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
        let large_file_offset = slice.read_u8()?;

        Ok(NeoXIndex2 {
            name_hash,
            offset,
            compressed_size,
            uncompressed_size,
            compression_type: match compress_type {
                0 => NeoXIndex2CompressType::None,
                1 => NeoXIndex2CompressType::ZLib,
                2 => NeoXIndex2CompressType::LZ4,
                _ => return Err(NeoXIndexError::UnkownCompressType),
            },
            encrypt_type: match encrypt_type {
                0 => NeoXIndex2EncryptType::None,
                1 => NeoXIndex2EncryptType::Simple,
                2 => NeoXIndex2EncryptType::RC4,
                _ => return Err(NeoXIndexError::UnknownEncryptType),
            },
            large_file_offset,
        })
    }
}

enum NeoXIndex {
    Version1(NeoXIndex1),
    Version1_2(NeoXIndex1_2),
    Version1_32(NeoXIndex1_32),
    Version2(NeoXIndex2),
}

impl NeoXIndex {
    pub fn name_hash(&self) -> u64 {
        match self {
            Self::Version1(index) => index.name_hash,
            Self::Version1_2(index) => index.name_hash,
            Self::Version1_32(index) => index.name_hash,
            Self::Version2(index) => index.name_hash(),
        }
    }

    pub fn encrypt_type(&self) -> NeoXIndex2EncryptType {
        match self {
            Self::Version1(index) => index.encrypt_type,
            Self::Version1_2(index) => index.encrypt_type,
            Self::Version1_32(index) => index.encrypt_type,
            Self::Version2(index) => index.encrypt_type,
        }
    }

    pub fn compress_type(&self) -> NeoXIndex2CompressType {
        match self {
            Self::Version1(index) => index.compression_type,
            Self::Version1_2(index) => index.compression_type,
            Self::Version1_32(index) => index.compression_type,
            Self::Version2(index) => index.compression_type,
        }
    }

    pub fn offset(&self) -> u64 {
        match self {
            Self::Version1(index) => index.offset as u64 | (index.large_file_offset as u64) << 20,
            Self::Version1_2(index) => index.offset as u64 | (index.large_file_offset as u64) << 20,
            Self::Version1_32(index) => {
                index.offset as u64 | (index.large_file_offset as u64) << 20
            }
            Self::Version2(index) => index.offset as u64 | (index.large_file_offset as u64) << 20,
        }
    }

    pub fn compressed_size(&self) -> u64 {
        (match self {
            Self::Version1(index) => index.compressed_size,
            Self::Version1_2(index) => index.compressed_size,
            Self::Version1_32(index) => index.compressed_size,
            Self::Version2(index) => index.compressed_size,
        }) as u64
    }

    pub fn read_content_from_buffer<T>(
        &self,
        reader: &mut std::io::BufReader<T>,
    ) -> Result<Vec<u8>, NeoXIndexError>
    where
        T: std::io::Read,
        T: std::io::Seek,
    {
        let offset = self.offset();
        let compress_type = self.compress_type();
        let encrypt_type = self.encrypt_type();

        reader.seek(std::io::SeekFrom::Start(offset))?;

        let mut buffer = vec![0; self.compressed_size() as usize];
        trace!("Read NeoX source buffer");
        reader.read_exact(&mut buffer).unwrap();

        let unencrypted_buffer = match encrypt_type {
            NeoXIndex2EncryptType::None => buffer,
            NeoXIndex2EncryptType::RC4 => unimplemented!("RC4 is currently not supported"),
            NeoXIndex2EncryptType::Simple => {
                unimplemented!("Simple encrypt is currenlty not support")
            }
        };

        let uncompressed_buffer = match compress_type {
            NeoXIndex2CompressType::None => unencrypted_buffer,
            NeoXIndex2CompressType::LZ4 => {
                let mut decompressed = Vec::new();
                let len = compress::lz4::decode_block(&unencrypted_buffer, &mut decompressed);
                if len < 1 {
                    return Err(NeoXIndexError::DecompressFailedLZ4);
                }
                decompressed
            }
            NeoXIndex2CompressType::ZLib => {
                let mut decompressed = Vec::new();
                let res = compress::zlib::Decoder::new(std::io::Cursor::new(&unencrypted_buffer))
                    .read_to_end(&mut decompressed);
                if res.is_err() {
                    return Err(NeoXIndexError::DecompressFailedZLib);
                }
                decompressed
            }
        };

        Ok(uncompressed_buffer)
    }
}

#[derive(Debug)]
enum Npk2Error {
    IoError(std::io::Error),
    InvalidHeader,
    IndexError(NeoXIndexError),
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

impl From<NeoXIndexError> for Npk2Error {
    fn from(e: NeoXIndexError) -> Self {
        Self::IndexError(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NpkVersion {
    Version1,
    Version1_2,
    Version1_32,
    Version2,
}

struct NpkHeader {
    version: NpkVersion,
    file_count: u32,
    large_file_index_offset: u32,
    index_offset: u32,
}

impl NpkHeader {
    pub fn index_offset(&self) -> u64 {
        self.index_offset as u64 | ((self.large_file_index_offset as u64) << 0x20)
    }

    pub fn size_of_index_entry(&self) -> u64 {
        match self.version {
            NpkVersion::Version1 => 0x1C,
            NpkVersion::Version1_32 => 0x1C,
            NpkVersion::Version1_2 => 0x20,
            NpkVersion::Version2 => 0x28,
        }
    }

    pub fn index_buffer_size(&self) -> usize {
        //
        self.file_count as usize * self.size_of_index_entry() as usize
    }
}

struct NpkReader {
    file: std::fs::File,
    header: NpkHeader,
    indices: Vec<NeoXIndex>,
}

impl NpkReader {
    fn read_header<T>(reader: &mut std::io::BufReader<T>) -> Result<NpkHeader, Npk2Error>
    where
        T: std::io::Read,
        T: std::io::Seek,
    {
        //
        let file_size = reader.seek(std::io::SeekFrom::End(0))?;
        reader.seek(std::io::SeekFrom::Start(0))?;
        let magic = reader.read_u32::<LittleEndian>()?; // +0
        if magic != 0x4B50584E {
            return Err(Npk2Error::InvalidHeader);
        }

        let file_count = reader.read_u32::<LittleEndian>()?; // +4

        let large_file_index_offset = reader.read_u32::<LittleEndian>()?;
        // let _var1 = reader.read_u32::<LittleEndian>()?; // IGNORED, hash variant? // +8
        let _var2 = reader.read_u32::<LittleEndian>()?; // IGNORED // +12
        let _var3 = reader.read_u32::<LittleEndian>()?; // IGNORED // +16
        let index_offset = reader.read_u32::<LittleEndian>()?; // +20

        // NOTE(alexander): This is a very very crude way of detecting the format of this
        // TODO(alexander): Try to find a better way to detect what version
        // of NPK this is
        let full_index_offset = index_offset as u64 | ((large_file_index_offset as u64) << 0x20);
        let version = {
            if full_index_offset + file_count as u64 * 0x28 <= file_size {
                NpkVersion::Version2
            } else if full_index_offset + file_count as u64 * 0x20 <= file_size {
                NpkVersion::Version1_2
            } else if full_index_offset + file_count as u64 * 0x1C <= file_size && _var3 == 0 {
                NpkVersion::Version1
            } else if full_index_offset + file_count as u64 * 0x1C <= file_size && _var3 == 2 {
                NpkVersion::Version1_32
            } else {
                unimplemented!("Unsupported NPK Version");
            }
        };
        info!("Detected NPK Version: {:?}", version);

        //
        //
        Ok(NpkHeader {
            version,
            file_count,
            large_file_index_offset,
            index_offset,
        })
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Npk2Error> {
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(&file);
        let header = Self::read_header(&mut reader)?;

        Ok(NpkReader {
            file,
            header,
            indices: Vec::new(),
        })
    }

    pub fn open(&mut self) -> Result<(), Npk2Error> {
        let mut reader = BufReader::new(&self.file);

        let pos = reader.seek(std::io::SeekFrom::Start(self.header.index_offset()))?;
        assert!(pos == self.header.index_offset());

        // Read the Index Buffer into Memory
        let mut buffer = vec![0; self.header.index_buffer_size()];
        reader.read_exact(&mut buffer)?;

        if !is_eof(&mut reader)? {
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
            let mut sub_buffer = vec![0; self.header.size_of_index_entry() as usize];
            while buffer_cursor.read_exact(&mut sub_buffer).is_ok() {
                self.indices.push(match self.header.version {
                    NpkVersion::Version1 => {
                        let index = NeoXIndex1::from_slice(sub_buffer.as_mut_slice())?;
                        NeoXIndex::Version1(index)
                    }
                    NpkVersion::Version1_2 => {
                        let index = NeoXIndex1_2::from_slice(sub_buffer.as_mut_slice())?;
                        NeoXIndex::Version1_2(index)
                    }
                    NpkVersion::Version1_32 => {
                        let index = NeoXIndex1_32::from_slice(sub_buffer.as_mut_slice())?;
                        NeoXIndex::Version1_32(index)
                    }
                    NpkVersion::Version2 => {
                        let index = NeoXIndex2::from_slice(sub_buffer.as_mut_slice())?;
                        NeoXIndex::Version2(index)
                    }
                });
            }
        }

        Ok(())
    }

    pub fn indices(&self) -> &Vec<NeoXIndex> {
        &self.indices
    }

    pub fn read_content_for_index(&self, index: &NeoXIndex) -> Result<Vec<u8>, Npk2Error> {
        let mut reader = BufReader::new(&self.file);
        Ok(index.read_content_from_buffer(&mut reader)?)
    }
}

fn load_file_name_hash_mappings<T>(
    reader: &mut T,
    version: NpkVersion,
) -> std::collections::HashMap<u64, String>
where
    T: std::io::BufRead,
{
    info!("Parsing filelist");
    let mut file_mappings = std::collections::HashMap::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            //
            // <type> <hash>    <unkown>   <size>  0   <filename>
            let r = regex::Regex::new(
                r"(\S+)(?:\s+)(\S+)(?:\s+)(\S+)(?:\s+)(\S+)(?:\s+)(\S+)(?:\s+)(\S.*)",
            )
            .unwrap();
            let caps = r.captures(&line);
            if let Some(caps) = caps {
                let name_hash = caps.get(2).unwrap().as_str().parse::<u64>().unwrap();
                let filename = caps.get(6).unwrap().as_str();
                file_mappings.insert(name_hash, filename.to_string());
            } else {
                use murmur3::murmur3_32;
                use std::io::Cursor;

                if version == NpkVersion::Version1 || version == NpkVersion::Version1_32 {
                    let hash = murmur3_32(
                        &mut Cursor::new(line.clone().replace("/", "\\")),
                        0x9747B28C,
                    )
                    .unwrap();
                    file_mappings.insert(hash as u64, line.to_string());
                } else {
                    let top = murmur3_32(
                        &mut Cursor::new(line.clone().replace("/", "\\")),
                        0x9747B28C,
                    )
                    .unwrap();
                    let bottom = murmur3_32(
                        &mut Cursor::new(line.clone().replace("/", "\\")),
                        0xC82B7479,
                    )
                    .unwrap();
                    let hash = (bottom as i64 | (top as i64) << 0x20) as i64;
                    file_mappings.insert(hash as u64, line.to_string());
                }
            }
        }
    }
    file_mappings
}

fn main() -> Result<(), Npk2Error> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let matches = App::new("NeoX NPK Tool")
        .version("1.0")
        .author("Alexander Guettler <alexander@guettler.io>")
        .subcommand(App::new("x")
        .about("Unpack one or more NPKS")
        .arg(
            Arg::new("INPUT")
                .help("The NPK file(s) to be operated on")
                .required(true)
                .multiple(true)
                .index(1),
        )
        .arg(
            Arg::new("DIR")
                .short('d')
                .long("dir")
                .value_name("DIR")
                .help("The directory where this NPK file should be extracted to")
                .default_value("out")
                .takes_value(true),
        )
        .arg(
            Arg::new("FILELIST")
            .short('f')
            .long("filelist")
            .value_name(
                "FILELIST"
            ).help("Supplies a file list to the npk unpack which will be used to try and reconstruct the original file tree\nWhen INPUT is supplied with a list of all resX.npk files this may be determined and used automatically.")
        ))
        .get_matches();

    match matches.subcommand() {
        Some(("x", sub_m)) => {
            let input_files: Vec<&str> = sub_m.values_of("INPUT").unwrap().collect();

            let mut npk_readers = Vec::new();
            for input_file in input_files {
                let mut npk_file = NpkReader::new(input_file)?;
                npk_file.open()?;
                npk_readers.push(npk_file);
            }

            let file_list = match sub_m.value_of("FILELIST") {
                Some(path) => {
                    let file = std::fs::File::open(path)?;
                    load_file_name_hash_mappings(&mut BufReader::new(file), NpkVersion::Version1)
                }
                None => {
                    match npk_readers
                        .iter()
                        .map(|x| x.indices().iter().map(|i| (x, i)).collect::<Vec<_>>())
                        .collect::<Vec<Vec<_>>>()
                        .into_iter()
                        .flatten()
                        .find(|(_, x)| {
                            x.name_hash() == 0xD4A17339F75381FD
                                || x.name_hash() == 0xE581738CE3FD567E
                                || x.name_hash() == 0x4176DE2A
                        }) {
                        Some((npk_file, index)) => {
                            let content = npk_file.read_content_for_index(index)?;
                            let mut decompressed = Vec::new();
                            match compress::zlib::Decoder::new(std::io::Cursor::new(&content))
                                .read_to_end(&mut decompressed)
                            {
                                Ok(_) => load_file_name_hash_mappings(
                                    &mut std::io::Cursor::new(&decompressed),
                                    npk_file.header.version,
                                ),
                                Err(_) => load_file_name_hash_mappings(
                                    &mut std::io::Cursor::new(&content),
                                    npk_file.header.version,
                                ),
                            }
                        }
                        None => std::collections::HashMap::new(),
                    }
                }
            };

            let output_directory = std::path::Path::new(sub_m.value_of("DIR").unwrap());
            std::fs::create_dir_all(&output_directory)?;

            use indicatif::{ProgressBar, ProgressStyle};
            let pb = ProgressBar::new(
                npk_readers
                    .iter()
                    .fold(0 as usize, |x, y| x + y.indices().len()) as u64,
            );
            pb.set_style(ProgressStyle::default_bar().template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, {msg}, ETA {eta})",
            ));
            pb.enable_steady_tick(100);
            for npk_file in npk_readers {
                for index in npk_file.indices() {
                    let content = npk_file.read_content_for_index(index)?;
                    let mut decompressed = Vec::new();
                    match compress::zlib::Decoder::new(std::io::Cursor::new(&content))
                        .read_to_end(&mut decompressed)
                    {
                        Ok(_) => {
                            println!("Possible file hash {}", index.name_hash());
                            println!("Possible file hash {}", index.name_hash());
                            println!("Possible file hash {}", index.name_hash());
                        }
                        Err(_) => {}
                    }
                    let file_name = match file_list.get(&index.name_hash()) {
                        Some(file_name) => file_name.clone(),
                        None => {
                            // This is a massive hack, but oh well
                            // We know the hash, and we know what the file is
                            // possibly zlib compressed data
                            // Or in-case of some other npk files there is a different
                            // plain text file with a list of files
                            if index.name_hash() == 0xD4A17339F75381FD
                                || index.name_hash() == 0xE581738CE3FD567E
                                || index.name_hash() == 0x4176DE2A
                            {
                                "filelist.txt".to_string()
                            } else {
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
                                                    if magic == 0x58544BAB {
                                                        "ktx"
                                                    } else {
                                                        "dat"
                                                    }
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
                                    "image/vnd.zbrush.pcx" => "pcx",
                                    "audio/mpeg" => "mp3",
                                    "audio/x-wav" => "wav",
                                    "application/x-java-jce-keystore" => ".pem",
                                    "application/x-font-ttf" => ".ttf",
                                    _ => {
                                        pb.println(format!("Unhandled mime type {}", result));
                                        // error!("Unhandled mime type {}", result);
                                        "dat"
                                    }
                                };
                                format!("unknown_file_name/{:X}.{}", index.name_hash(), extension)
                            }
                        }
                    };
                    //
                    pb.set_message(format!("{}", file_name));
                    pb.inc(1);

                    let out_file = output_directory.join(file_name);
                    if let Some(dir_path) = std::path::Path::new(&out_file).parent() {
                        std::fs::create_dir_all(dir_path)?;
                    }
                    debug!(
                        "Writing {} to {}",
                        bytesize::ByteSize(content.len() as u64),
                        out_file.as_path().to_str().unwrap()
                    );
                    std::fs::write(out_file, &content)?;
                }
            }
        }
        Some(("p", _)) => unimplemented!("Packing is currently not supported"),
        _ => {}
    }

    info!("Done.");

    Ok(())
}
