fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("src/assets/icons/datapuller_icon.ico");
    match res.compile() {
        Ok(_) => {
            println!("Resource file compiled successfully.");
        }
        Err(e) => {
            eprintln!("Error compiling resource file: {:?}", e);
            std::process::exit(1);
        }
    }
}
