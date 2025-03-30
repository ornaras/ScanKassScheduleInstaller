use std::fs::{OpenOptions, File};
use std::env::{temp_dir, var};
use std::io::{Cursor, Read, Write};
use std::path::{PathBuf, Path};
use whoami::arch;
use runas::Command;
use rand::distr::{Alphanumeric, SampleString};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use tokio::runtime::Handle;
use serde_json::Value;

const ASPNET_URL: &str = "https://download.visualstudio.microsoft.com/download/pr/8cfa7f46-88f2-4521-a2d8-59b827420344/447de18a48115ac0fe6f381f0528e7a5/aspnetcore-runtime-6.0.36-win-x86.exe"; // {5FEC97CA-FD93-392D-BF36-D9C3492A5698}
const HOSTING_BUNDLE: &str = "https://download.visualstudio.microsoft.com/download/pr/9b8253ef-554d-4636-b708-e154c0199ce5/f3673dd1f2dc80e5b0505cbd2d4bd5d2/dotnet-hosting-6.0.36-win.exe"; // {040F8B83-B3BA-303A-A5BC-FE3E7FC0093B}
const WD86_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/WebDeploy_x86_ru-RU.msi";
const WD64_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/webdeploy_amd64_ru-RU.msi";
const PATH: &str = "C:\\ScanKass\\WORKFLOW";

#[no_mangle]
pub extern "C" fn is_installed() -> bool{
    Handle::current().block_on(is_installed_async())
}

async fn is_installed_async() -> bool {
    
    if !Path::new(format!("{}\\SkatWorkerAPI.exe", PATH).as_str()).exists() {
        let cl = reqwest::Client::new();
        let resp = cl.get("http://localhost/api/Schedule/list").send().await;
        if resp.is_ok() {
            return resp.unwrap().status() == reqwest::StatusCode::OK;
        }
    }
    false
}

fn exists_app(pattern: &str) -> bool{
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let pathes = vec!(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall", 
        "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall");
    for path in pathes{
        let p = format!("{}\\{}", path, pattern);
        let res = hklm.open_subkey(p);
        if res.is_ok(){
            return true;
        }
    }
    false
}


fn enable_features(features: Vec<&str>){
    let mut proc: Command = Command::new("dism");
    proc.arg("/online").arg("/enable-feature");
    for feature in features{
        proc.arg(format!("/featurename:{0}", feature));
    }
    proc.status().unwrap();
}

#[no_mangle]
pub extern "C" fn install() -> i32 {
    adv_install(true)
}

#[no_mangle]
pub extern "C" fn adv_install(is_slient: bool) -> i32 {
    Handle::current().block_on(install_async(is_slient))
}

async fn install_async(is_slient: bool) -> i32 {
    if is_installed() { return 1; }

    let arch = format!("{:?}", arch());

    let wd_url = match arch.as_str(){
        "X64" => WD64_URL,
        "i686" => WD86_URL,
        "i586" => WD86_URL,
        "i386" => WD86_URL,
            _ => return 2
    };

    if !exists_app("{215198BD-8EE1-385D-8194-0D3FF304296D}") {
        download_and_execute(ASPNET_URL, is_slient).await;
    }

    if !exists_app("{040F8B83-B3BA-303A-A5BC-FE3E7FC0093B}") {
        download_and_execute(HOSTING_BUNDLE, is_slient).await;
    }

    download_and_install(wd_url, is_slient).await;

    enable_features(vec!["IIS-WebServerRole", "WAS-WindowsActivationService", "WAS-ProcessModel","WAS-NetFxEnvironment","WAS-ConfigurationAPI"]);
    
    let app_inetsrv_path: String = var("WINDIR").unwrap() + "\\system32\\inetsrv\\APPCMD";
    Command::new(&app_inetsrv_path).arg("stop").arg("site").arg("http://*:80").status().unwrap(); // Выключение всех сайтов на порту 80
    Command::new(&app_inetsrv_path).arg("add").arg("apppool").arg("/name:ScanKass").arg("/processModel.identityType:LocalSystem").status().unwrap(); // Создание отдельного пула
    Command::new(&app_inetsrv_path).arg("add").arg("site").arg("/name:SkatWorkerAPI").arg("/bindings:http/*:80:").arg(format!("/physicalPath:{}",PATH)).status().unwrap(); // Создание сайта
    Command::new(&app_inetsrv_path).arg("set").arg("app").arg("SkatWorkerAPI/").arg("/applicationPool:ScanKass").status().unwrap(); // Присвоение пула
    Command::new(&app_inetsrv_path).arg("start").arg("site").arg("SkatWorkerAPI").status().unwrap(); // Запуск сайта

    install_skat_worker().await;

    configure();

    0
}

fn configure(){
    let mut file = OpenOptions::new().read(true).write(true).open(format!("{}\\appsettings.json", PATH)).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut json: Value = serde_json::from_str(contents.as_str()).unwrap();
    json["Settings"]["ConnectionString"] = Value::String(format!("Data Source={}\\db", PATH).to_string());
    json["Settings"]["PathToLog"] = Value::String("C:\\ScanKass\\LOG".to_string());
    contents = serde_json::to_string(&json).unwrap();
    std::fs::remove_file(format!("{}\\appsettings.json", PATH)).unwrap();
    let mut t = File::create(format!("{}\\appsettings.json", PATH)).unwrap();
    File::write_all(&mut t, contents.as_bytes()).unwrap();
}


async fn download(url: &str, extension: &str) -> String {
    let filename = format!("{0}.{1}", Alphanumeric.sample_string(&mut rand::rng(),16), extension);
    let filepath = format!("{0}{1}", temp_dir().display(), filename);
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await.unwrap();
    let mut file = File::create(filepath.clone()).unwrap();

    while let Some(chunk) = response.chunk().await.unwrap() {
        file.write_all(&chunk).unwrap();
    }

    file.flush().unwrap();

    filepath
}

async fn download_and_execute(url: &str, is_slient: bool) {
    let path = download(url, "exe").await;
    let mut ui_mode = "/passive";
    if is_slient {
        ui_mode = "/quiet";
    }
    Command::new(&path).arg("/install").arg(ui_mode).arg("/norestart").status().unwrap();
    std::fs::remove_file(&path).unwrap();
}

async fn download_and_install(url: &str, is_slient: bool) {
    let path = download(url, "msi").await;
    let mut ui_mode = "/passive";
    if is_slient {
        ui_mode = "/quiet";
    }
    Command::new("msiexec").arg("/i").arg(path.as_str()).arg(ui_mode).arg("/norestart").status().unwrap();
    std::fs::remove_file(&path).unwrap();
}

async fn download_and_extract(url: &str) -> String {
    let path = download(url, "zip").await;
    let mut file = File::open(path).unwrap();
    let mut data: Vec<u8> = vec![];
    file.read_to_end(&mut data).unwrap();
    let dir_path = format!("{0}{1}",temp_dir().display(),Alphanumeric.sample_string(&mut rand::rng(),16));
    std::fs::create_dir_all(&dir_path).unwrap();
    zip_extract::extract(Cursor::new(&data), &PathBuf::from(&dir_path), true).unwrap();
    dir_path
}

async fn get_latest_release() -> String {
    let client = reqwest::Client::new();
    let resp = client.get("https://api.github.com/repos/StarkovVV18/SkatWorker/releases")
        .header("accept", "application/vnd.github+json")
        .header("User-Agent", "curl").send().await.unwrap();
    let body = resp.text().await.unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&body).unwrap();
    let res = format!("{}", json_value[0]["assets"][0]["browser_download_url"]);
    res[1..res.len() - 1].to_string()
}

async fn install_skat_worker() {
    let url = get_latest_release().await;
    let path = download_and_extract(url.as_str()).await;
    Command::new(format!("{}/SkatWorkerAPI.deploy.cmd", path)).arg("/Y").status().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        if is_installed(){
            panic!("Тест не пройден (is_installed неправильно работает)")
        }
        let code = install();
        if code != 0 {
            panic!("Тест не пройден (Код: {code})")
        }
        assert_eq!(is_installed(), true);
    }
}
