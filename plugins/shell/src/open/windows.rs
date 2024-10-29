use std::path::Path;

use windows::{
    core::{w, HSTRING, PCWSTR},
    Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
        System::Com::CoInitialize,
        UI::{
            Shell::{
                ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems, ShellExecuteExW,
                SHELLEXECUTEINFOW,
            },
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
};

pub fn show_item_in_directory(file: &Path) -> crate::Result<()> {
    let _ = unsafe { CoInitialize(None) };

    let dir = file
        .parent()
        .ok_or_else(|| crate::Error::NoParent(file.to_path_buf()))?;

    let dir = HSTRING::from(dir);
    let dir_item = unsafe { ILCreateFromPathW(PCWSTR::from_raw(dir.as_ptr())) };

    let file_h = HSTRING::from(file);
    let file_item = unsafe { ILCreateFromPathW(PCWSTR::from_raw(file_h.as_ptr())) };

    unsafe {
        if let Err(e) = SHOpenFolderAndSelectItems(dir_item, Some(&[file_item]), 0) {
            if e.code().0 == ERROR_FILE_NOT_FOUND.0 as i32 {
                let is_dir = std::fs::metadata(file).map(|f| f.is_dir()).unwrap_or(false);
                let mut info = SHELLEXECUTEINFOW {
                    cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as _,
                    nShow: SW_SHOWNORMAL.0,
                    lpVerb: if is_dir {
                        w!("explore")
                    } else {
                        PCWSTR::null()
                    },
                    lpClass: if is_dir { w!("folder") } else { PCWSTR::null() },
                    lpFile: PCWSTR(file_h.as_ptr()),
                    ..std::mem::zeroed()
                };

                ShellExecuteExW(&mut info)?;
            }
        }
    }

    unsafe {
        ILFree(Some(dir_item));
        ILFree(Some(file_item));
    }

    Ok(())
}
