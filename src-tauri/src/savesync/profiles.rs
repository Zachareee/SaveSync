use std::{
    ffi::{OsStr, OsString},
    fs,
};

use crate::{app_store, savesync::fs_utils::FolderItems};

const PROFILEEXTENSION: &'static str = "savesyncprofile";

pub fn swap_saves(tag: &str, filename: &OsStr, profile: &OsStr) {
    let mut active_profiles = app_store().get_active_profiles(tag);
    let savepath = app_store().get_mapping(tag).unwrap();

    let current_profile = active_profiles
        .insert(filename.to_owned(), profile.to_owned())
        .unwrap_or("New profile".into());

    // swap out file to profile format
    fs::rename(
        savepath.join(filename),
        savepath
            .join(current_profile)
            .with_extension(PROFILEEXTENSION),
    )
    .unwrap();

    // swap in profile format to file
    fs::rename(
        savepath.join(savepath.join(profile).with_extension(PROFILEEXTENSION)),
        savepath.join(filename),
    )
    .unwrap();
    app_store().set_active_profiles(tag, active_profiles);
}
