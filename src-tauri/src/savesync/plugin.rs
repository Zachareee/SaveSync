use libloading::Library;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{c_char, CStr, CString, OsStr, OsString},
    fs,
    path::Path,
    time::{Duration, SystemTime},
};

use super::config_paths;

type DLLString = *const c_char;
type DLLFileDetails = *const (DLLString, DLLString, u64, DLLString);
type DLLResult<T> = Result<T, String>;

#[derive(Debug)]
pub struct Plugin {
    library: Library,
    filename: OsString,
    credentials: Option<String>,
}

/// Gets file's last modified date
/// Plugin developers can optionally attach the
/// file buffer to reduce API calls where possible
pub struct FileDetails {
    pub tag: String,
    pub folder_name: OsString,
    pub last_modified: SystemTime,
    pub data: Option<Vec<u8>>,
}

impl Plugin {
    unsafe fn free_string(&self, raw_str: DLLString) {
        unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString)>(b"free_string")
                .expect("free_string function not found")(raw_str)
        }
    }

    unsafe fn create_string(&self, raw_str: DLLString) -> Option<&[u8]> {
        if raw_str.is_null() {
            None
        } else {
            let c_str = unsafe { CStr::from_ptr(raw_str) }.to_bytes();
            unsafe {
                self.free_string(raw_str);
            }
            Some(c_str)
        }
    }

    fn credentials(&self) -> String {
        self.credentials.clone().unwrap_or_default()
    }

    pub fn filename(&self) -> OsString {
        self.filename.clone()
    }

    pub unsafe fn new(servicename: &OsStr) -> PluginResult<Plugin> {
        let library = unsafe { Library::new(servicename) }.unwrap();

        Ok(Plugin {
            library,
            filename: servicename.to_owned(),
            credentials: Plugin::read_creds(servicename),
        })
    }

    pub fn info(&self) -> PluginResult<PluginInfo> {
        let ptr = unsafe {
            self.library
                .get::<unsafe extern "C" fn() -> (DLLString, DLLString, DLLString, DLLString)>(
                    b"info",
                )
                .expect("info function not found")()
        };

        let (name, description, author, icon_url) = ptr;

        let url: Option<String> = unsafe { self.create_string(icon_url).into() };

        let info = unsafe {
            PluginInfo {
                name: self.create_string(name).unwrap_or_default().into(),
                description: self.create_string(description).unwrap_or_default().into(),
                author: self.create_string(author).unwrap_or_default().into(),
                icon_url: self.create_string(icon_url).unwrap_or("??".into()).into(),
                filename: self.filename(),
            }
        };

        unsafe {
            self.library
                .get::<unsafe extern "C" fn((DLLString, DLLString, DLLString, DLLString))>(
                    b"free_info",
                )
                .expect("free_info function not found")(ptr)
        };

        Ok(info)
    }

    fn read_creds(filename: &OsStr) -> Option<String> {
        let mut filename = filename.to_owned();
        filename.push(".auth");

        fs::read_to_string(config_paths::creds().join(&filename)).ok()
    }

    fn write_creds(&mut self, credentials: &str) -> std::io::Result<()> {
        self.credentials = Some(credentials.into());
        fs::write(self.filename.to_os_string(), credentials)
    }

    pub fn validate(&self, redirect_uri: &str) -> (Option<String>, Option<String>) {
        let credentials = CString::new(self.credentials()).unwrap_or_default();
        let redirect_uri = CString::new(redirect_uri).unwrap_or_default();

        let (url, msg) = unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString, DLLString) -> (DLLString, DLLString)>(
                    b"validate",
                )
                .expect("validate function not found")(
                credentials.as_ptr(), redirect_uri.as_ptr()
            )
        };

        unsafe { (self.create_string(url).into(), self.create_string(msg)).into() }
    }

    pub fn process_save_credentials(&mut self, url: &str) -> PluginResult<()> {
        let cstring = CString::new(url).unwrap_or_default();

        let (res, possible_err) = unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString) -> (DLLString, DLLString)>(
                    b"extract_credentials",
                )
                .expect("extract_credentials function not found")(cstring.as_ptr())
        };

        match unsafe { self.create_string(possible_err) } {
            None => {
                let _ = self.write_creds(unsafe {
                    &self
                        .create_string(res)
                        .expect("Both ok and error value are empty")
                        .into()
                });
                ()?
            }
            Some(e) => e.into()?,
        }
    }

    pub fn abort(&self) -> PluginResult<()> {
        Ok(())
    }

    pub fn upload(
        &self,
        tag: &[u8],
        folder_name: &[u8],
        date: SystemTime,
        buffer: &[u8],
    ) -> PluginResult<()> {
        let access_token = CString::new(self.credentials()).unwrap_or_default();
        let tagname = CString::new(tag).unwrap_or_default();
        let filename = CString::new(folder_name).unwrap_or_default();

        unsafe {
            let ptr = self
                .library
                .get::<unsafe extern "C" fn(
                    DLLString,
                    DLLString,
                    DLLString,
                    u64,
                    DLLString,
                    u64,
                ) -> DLLString>(b"upload")
                .expect("upload function not found")(
                access_token.as_ptr(),
                tagname.as_ptr(),
                filename.as_ptr(),
                date.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                buffer.as_ptr() as *const i8,
                buffer.len() as u64,
            );
            self.create_string(ptr).map_or(()?, |e| e.into()?)
        }
    }

    pub fn download(&self, tag: &[u8], folder_name: &[u8]) -> PluginResult<Vec<u8>> {
        let access_token = CString::new(self.credentials()).unwrap_or_default();
        let filename = CString::new(folder_name).unwrap_or_default();

        let (ptr, count, possible_err) =
            unsafe {
                self.library
                .get::<unsafe extern "C" fn(DLLString, DLLString) -> (DLLString, u64, DLLString)>(
                    b"download",
                )
                .expect("download function not found")(access_token.as_ptr(), filename.as_ptr())
            };

        if let Some(err) = unsafe { self.create_string(possible_err) } {
            err.into()?
        } else {
            let mut v = Vec::new();
            let u8_ptr = ptr as *const u8;

            for i in 0..count as isize {
                v.push(unsafe { *u8_ptr.offset(i) });
            }

            unsafe { self.free_string(ptr) };

            v?
        }
    }

    pub fn remove(&self, tag: &[u8], folder_name: &[u8]) -> PluginResult<()> {
        let access_token = CString::new(self.credentials()).unwrap_or_default();
        let tagname = CString::new(tag).unwrap_or_default();
        let filename = CString::new(folder_name).unwrap_or_default();

        unsafe {
            let ptr = self
                .library
                .get::<unsafe extern "C" fn(DLLString, DLLString, DLLString) -> DLLString>(
                    b"remove",
                )
                .expect("upload function not found")(
                access_token.as_ptr(),
                tagname.as_ptr(),
                filename.as_ptr(),
            );

            self.create_string(ptr).map_or_else(()?, |e| e.into()?)
        }
    }

    pub fn read_cloud(&self) -> PluginResult<Vec<FileDetails>> {
        let access_token = CString::new(self.credentials()).unwrap_or_default();

        unsafe {
            let (ptr, count, possible_err) =
                self.library
                    .get::<unsafe extern "C" fn(DLLString) -> (DLLFileDetails, u64, DLLString)>(
                        b"read_cloud",
                    )
                    .expect("read_cloud function not found")(access_token.as_ptr());

            match self.create_string(possible_err) {
                Some(err) => err.into()?,
                None => {
                    let mut v: Vec<FileDetails> = Vec::new();

                    for i in 0..count as isize {
                        let detail = *ptr.offset(i);
                        v.push(FileDetails {
                            tag: self.create_string(detail.0).unwrap().into(),
                            folder_name: self.create_string(detail.1).unwrap().into(),
                            last_modified: SystemTime::UNIX_EPOCH + Duration::from_secs(detail.2),
                            data: self.create_string(detail.3).into(),
                        });
                    }

                    self.library
                        .get::<unsafe extern "C" fn(u64, DLLFileDetails)>(b"free_file_details")
                        .expect("free_file_details function not found")(
                        count, ptr
                    );
                    v?
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginInfo {
    name: String,
    description: String,
    author: String,
    icon_url: String,
    filename: OsString,
}

pub type PluginResult<T> = Result<T, String>;
