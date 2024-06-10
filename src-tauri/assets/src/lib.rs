pub fn sevenzip_path() -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap();
    let exe_dir = exe.parent().unwrap();
    let sevenzip = exe_dir.join("7zr.exe");

    sevenzip
}
