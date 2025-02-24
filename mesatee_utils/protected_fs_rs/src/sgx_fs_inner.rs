// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::ffi::{CStr, CString};
use std::io::{self, Error, SeekFrom};
use std::path::Path;

use crate::deps::libc;
use crate::deps::sgx_key_128bit_t;

use crate::sgx_tprotected_fs::{self, SgxFileStream};
pub struct SgxFile(SgxFileStream);

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    update: bool,
    binary: bool,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            update: false,
            binary: false,
        }
    }

    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    pub fn update(&mut self, update: bool) {
        self.update = update;
    }
    pub fn binary(&mut self, binary: bool) {
        self.binary = binary;
    }

    fn get_access_mode(&self) -> io::Result<String> {
        let mut mode = match (self.read, self.write, self.append) {
            (true, false, false) => "r".to_string(),
            (false, true, false) => "w".to_string(),
            (false, false, true) => "a".to_string(),
            _ => return Err(Error::from_raw_os_error(libc::EINVAL)),
        };
        if self.update {
            mode += "+";
        }
        if self.binary {
            mode += "b";
        }
        Ok(mode)
    }
}

impl SgxFile {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<SgxFile> {
        let path = cstr(path)?;
        let mode = opts.get_access_mode()?;
        let opts = CString::new(mode.as_bytes())?;
        SgxFile::open_c(&path, &opts, &sgx_key_128bit_t::default(), true)
    }

    pub fn open_ex(path: &Path, opts: &OpenOptions, key: &sgx_key_128bit_t) -> io::Result<SgxFile> {
        let path = cstr(path)?;
        let mode = opts.get_access_mode()?;
        let opts = CString::new(mode.as_bytes())?;
        SgxFile::open_c(&path, &opts, key, false)
    }

    pub fn open_c(
        path: &CStr,
        opts: &CStr,
        key: &sgx_key_128bit_t,
        auto: bool,
    ) -> io::Result<SgxFile> {
        let file = if auto {
            SgxFileStream::open_auto_key(path, opts)
        } else {
            SgxFileStream::open(path, opts, key)
        };

        file.map(SgxFile).map_err(Error::from_raw_os_error)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf).map_err(Error::from_raw_os_error)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf).map_err(Error::from_raw_os_error)
    }

    pub fn tell(&self) -> io::Result<u64> {
        self.0
            .tell()
            .map_err(Error::from_raw_os_error)
            .map(|offset| offset as u64)
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let (whence, offset) = match pos {
            SeekFrom::Start(off) => (sgx_tprotected_fs::SeekFrom::Start, off as i64),
            SeekFrom::End(off) => (sgx_tprotected_fs::SeekFrom::End, off),
            SeekFrom::Current(off) => (sgx_tprotected_fs::SeekFrom::Current, off),
        };

        self.0
            .seek(offset, whence)
            .map_err(Error::from_raw_os_error)?;

        let offset = self.tell()?;
        Ok(offset as u64)
    }

    pub fn flush(&self) -> io::Result<()> {
        self.0.flush().map_err(Error::from_raw_os_error)
    }

    pub fn is_eof(&self) -> bool {
        self.0.is_eof()
    }

    pub fn clearerr(&self) {
        self.0.clearerr()
    }

    pub fn clear_cache(&self) -> io::Result<()> {
        self.0.clear_cache().map_err(Error::from_raw_os_error)
    }
}

pub fn remove(path: &Path) -> io::Result<()> {
    let path = cstr(path)?;
    sgx_tprotected_fs::remove(&path).map_err(Error::from_raw_os_error)
}

#[cfg(feature = "mesalock_sgx")]
pub fn export_auto_key(path: &Path) -> io::Result<sgx_key_128bit_t> {
    let path = cstr(path)?;
    sgx_tprotected_fs::export_auto_key(&path).map_err(Error::from_raw_os_error)
}

#[cfg(feature = "mesalock_sgx")]
pub fn import_auto_key(path: &Path, key: &sgx_key_128bit_t) -> io::Result<()> {
    let path = cstr(path)?;
    sgx_tprotected_fs::import_auto_key(&path, key).map_err(Error::from_raw_os_error)
}

use std::os::unix::ffi::OsStrExt;
fn cstr(path: &Path) -> io::Result<CString> {
    Ok(CString::new(path.as_os_str().as_bytes())?)
}
