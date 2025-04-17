fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("CompanyName", "ООО Агрегатор"); // Sets manufacturer
        res.set("ProductName", "AGRG SH-D Utility");
        res.set("FileDescription", "Утилита для настройки ручки AGRG SH-D");
        res.compile().expect("Failed to embed icon");
    }
}