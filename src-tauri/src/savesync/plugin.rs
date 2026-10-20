use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::{
    ffi::{CStr, CString, OsStr, OsString, c_char},
    fs, slice,
    time::{Duration, SystemTime},
};

use crate::savesync::emitter;

use super::config_paths;

pub struct Plugin {
    library: Library,
    filename: OsString,
    credentials: Option<String>,
    details: Option<Vec<FileDetails>>,
}

/// Gets file's last modified date
/// Plugin developers can optionally attach the
/// file buffer to reduce API calls where possible
#[derive(Clone)]
pub struct FileDetails {
    pub tag: String,
    pub folder_name: OsString,
    pub last_modified: SystemTime,
    pub data: Option<Vec<u8>>,
}

impl Plugin {
    unsafe fn free_string(&self, raw_str: DLLString) {
        unsafe {
            if let Some(f) = self.get_function::<unsafe extern "C" fn(DLLString)>(b"free_string") {
                f(raw_str)
            }
        }
    }

    unsafe fn create_string(&self, raw_str: DLLString) -> Option<String> {
        if raw_str.is_null() {
            None
        } else {
            let c_str = unsafe { CStr::from_ptr(raw_str) }
                .to_owned()
                .into_string()
                .unwrap();
            unsafe {
                self.free_string(raw_str);
            }
            Some(c_str)
        }
    }

    unsafe fn test_error_empty(&self, raw_str: DLLString) -> bool {
        unsafe {
            self.create_string(raw_str)
                .map(|e| self.emit_error(e))
                .is_none()
        }
    }

    fn credentials(&self) -> String {
        self.credentials.clone().unwrap_or_default()
    }

    pub fn filename(&self) -> OsString {
        self.filename.clone()
    }

    fn emit_error(&self, description: String) {
        emitter::plugin_error(&self.filename(), &description);
    }

    fn get_function<T>(&self, symbol: &[u8]) -> Option<Symbol<'_, T>> {
        unsafe {
            self.library
                .get::<T>(symbol)
                .inspect_err(|_| {
                    self.emit_error(format!(
                        "{} function not found",
                        String::from_utf8_lossy(symbol)
                    ))
                })
                .ok()
        }
    }

    pub unsafe fn new(servicename: &OsStr) -> Option<Plugin> {
        let library: Library = unsafe {
            let library =
                libloading::os::windows::Library::new(config_paths::plugin().join(servicename))
                    .inspect_err(|e| emitter::plugin_error(servicename, &e.to_string()))
                    .ok()?;
            library
                .pin()
                .inspect_err(|e| emitter::plugin_error(servicename, &e.to_string()))
                .ok()?;
            library.into()
        };

        Some(Plugin {
            library,
            filename: servicename.to_owned(),
            credentials: Plugin::read_creds(servicename),
            details: None,
        })
    }

    pub fn info(&self) -> Option<PluginInfo> {
        let ptr = unsafe {
            self.get_function::<unsafe extern "C" fn() -> (DLLString, DLLString, DLLString, DLLString)>(b"info")?()
        };

        let (name, description, author, icon_url) = ptr;

        let info = unsafe {
            PluginInfo {
                name: self.create_string(name).unwrap_or_default(),
                description: self.create_string(description).unwrap_or_default(),
                author: self.create_string(author).unwrap_or_default(),
                icon_url: self.create_string(icon_url).unwrap_or_default(),
                filename: self.filename(),
            }
        };

        unsafe {
            self.get_function::<unsafe extern "C" fn((DLLString, DLLString, DLLString, DLLString))>(
                b"free_info",
            )?(ptr)
        };

        Some(info)
    }

    fn read_creds(filename: &OsStr) -> Option<String> {
        fs::read_to_string(config_paths::create_credential_path(filename.to_owned())).ok()
    }

    fn write_creds(&mut self, credentials: &str) {
        self.credentials = Some(credentials.into());

        fs::write(
            config_paths::create_credential_path(self.filename()),
            credentials,
        )
        .unwrap()
    }

    pub fn authenticate(&mut self) -> bool {
        let credentials = CString::new(self.credentials()).unwrap_or_default();

        self.get_function::<AuthFunction>(b"authenticate")
            .map(|f| unsafe { f(credentials.as_ptr()) })
            .is_some_and(|DLLResult { value, error }| unsafe {
                self.create_string(value)
                    .map(|creds| self.write_creds(&creds));
                error.is_null()
            })
    }

    pub fn auth_url(&self, redirect_uri: &str) -> Option<String> {
        let redirect_uri = CString::new(redirect_uri).unwrap_or_default();

        unsafe {
            self.create_string(self.get_function::<AuthUrlFunction>(b"auth_url")?(
                redirect_uri.as_ptr(),
            ))
        }
    }

    pub fn process_save_credentials(&mut self, url: &str) -> bool {
        let cstring = CString::new(url).unwrap_or_default();

        unsafe {
            self.get_function::<ExtractCredentialsFunction>(b"extract_credentials")
                .map(|f| f(cstring.as_ptr()))
                .is_some_and(|DLLResult { value, error }| {
                    let result = self.test_error_empty(error);
                    if result && let Some(credentials) = self.create_string(value) {
                        self.write_creds(&credentials)
                    };
                    result
                })
        }
    }

    pub fn upload(&self, tag: &[u8], folder_name: &[u8], date: SystemTime, buffer: &[u8]) -> bool {
        let access_token = CString::new(self.credentials()).unwrap_or_default();
        let tagname = CString::new(tag).unwrap_or_default();
        let filename = CString::new(folder_name).unwrap_or_default();

        unsafe {
            self.get_function::<UploadFunction>(b"upload")
                .map(|f| {
                    f(
                        access_token.as_ptr(),
                        tagname.as_ptr(),
                        filename.as_ptr(),
                        date.duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        buffer.as_ptr() as *const i8,
                        buffer.len() as u64,
                    )
                })
                .is_some_and(|ptr| self.test_error_empty(ptr))
        }
    }

    fn data_from_ptr(&self, ptr: DLLString, size: u64) -> Option<Vec<u8>> {
        if ptr.is_null() {
            None
        } else {
            let v = unsafe { slice::from_raw_parts(ptr as *const u8, size as usize) }.to_vec();

            unsafe { self.free_string(ptr) };

            Some(v)
        }
    }

    pub fn download(&self, tag: &[u8], folder_name: &[u8]) -> Option<Vec<u8>> {
        let access_token = CString::new(self.credentials()).unwrap_or_default();
        let tagname = CString::new(tag).unwrap_or_default();
        let filename = CString::new(folder_name).unwrap_or_default();

        let DLLFileResult { ptr, size, error } = unsafe {
            self.get_function::<DownloadFunction>(b"download")?(
                access_token.as_ptr(),
                tagname.as_ptr(),
                filename.as_ptr(),
            )
        };

        if unsafe { self.test_error_empty(error) } {
            self.data_from_ptr(ptr, size)
        } else {
            None
        }
    }

    pub fn remove(&self, tag: &[u8], folder_name: &[u8]) -> bool {
        let access_token = CString::new(self.credentials()).unwrap_or_default();
        let tagname = CString::new(tag).unwrap_or_default();
        let filename = CString::new(folder_name).unwrap_or_default();

        unsafe {
            self.get_function::<RemoveFunction>(b"remove")
                .map(|f| f(access_token.as_ptr(), tagname.as_ptr(), filename.as_ptr()))
                .is_some_and(|ptr| self.test_error_empty(ptr))
        }
    }

    pub fn read_cloud(&mut self) -> Option<Vec<FileDetails>> {
        self.details.clone().or_else(|| {
            let access_token = CString::new(self.credentials()).unwrap_or_default();

            unsafe {
                let DLLCloudResult { ptr, count, error } =
                    self.get_function::<ReadCloudFunction>(b"read_cloud")?(access_token.as_ptr());

                if self.test_error_empty(error) {
                    let mut v: Vec<FileDetails> = Vec::new();

                    for i in 0..count as isize {
                        let detail = *ptr.offset(i);
                        v.push(FileDetails {
                            tag: self.create_string(detail.0).unwrap().into(),
                            folder_name: self.create_string(detail.1).unwrap().into(),
                            last_modified: SystemTime::UNIX_EPOCH + Duration::from_secs(detail.2),
                            data: self.data_from_ptr(detail.3, detail.4),
                        });
                    }

                    self.get_function::<unsafe extern "C" fn(u64, DLLFileDetails)>(
                        b"free_file_details",
                    )?(count, ptr);

                    self.details = Some(v);
                    self.details.clone()
                } else {
                    None
                }
            }
        })
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

#[repr(C)]
struct DLLResult {
    pub value: DLLString,
    pub error: DLLString,
}

#[repr(C)]
struct DLLFileResult {
    pub ptr: DLLString,
    pub size: u64,
    pub error: DLLString,
}

#[repr(C)]
struct DLLCloudResult {
    pub ptr: DLLFileDetails,
    pub count: u64,
    pub error: DLLString,
}

type DLLString = *const c_char;
type DLLFileDetails = *const (DLLString, DLLString, u64, DLLString, u64);
type AuthFunction = unsafe extern "C" fn(DLLString) -> DLLResult;
type AuthUrlFunction = unsafe extern "C" fn(DLLString) -> DLLString;
type ExtractCredentialsFunction = unsafe extern "C" fn(DLLString) -> DLLResult;
type UploadFunction =
    unsafe extern "C" fn(DLLString, DLLString, DLLString, u64, DLLString, u64) -> DLLString;
type DownloadFunction = unsafe extern "C" fn(DLLString, DLLString, DLLString) -> DLLFileResult;
type RemoveFunction = unsafe extern "C" fn(DLLString, DLLString, DLLString) -> DLLString;
type ReadCloudFunction = unsafe extern "C" fn(DLLString) -> DLLCloudResult;
