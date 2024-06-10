fn download_7zr() {
    let out_path =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("../../../7zr.exe");

    if out_path.exists() {
        return;
    }

    let url = "https://www.7-zip.org/a/7zr.exe";
    let response = ureq::get(url).call().unwrap();
    let mut file = std::fs::File::create(&out_path).unwrap();
    std::io::copy(&mut response.into_reader(), &mut file).unwrap();
}

fn main() {
    download_7zr();
}
