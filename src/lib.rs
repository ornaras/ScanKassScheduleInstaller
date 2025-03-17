use std::fs::{OpenOptions, File};
use std::env::{temp_dir, var};
use std::io::{Cursor, Read, Write};
use std::path::{PathBuf, Path};
use runas::Command;
use rand::distr::{Alphanumeric, SampleString};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use tokio::runtime::Runtime;
use serde_json::Value;
use std::io::prelude::*;

const ASPNET_URL: &str = "https://download.visualstudio.microsoft.com/download/pr/8cfa7f46-88f2-4521-a2d8-59b827420344/447de18a48115ac0fe6f381f0528e7a5/aspnetcore-runtime-6.0.36-win-x86.exe"; // {5FEC97CA-FD93-392D-BF36-D9C3492A5698}
const HOSTING_BUNDLE: &str = "https://download.visualstudio.microsoft.com/download/pr/9b8253ef-554d-4636-b708-e154c0199ce5/f3673dd1f2dc80e5b0505cbd2d4bd5d2/dotnet-hosting-6.0.36-win.exe"; // {040F8B83-B3BA-303A-A5BC-FE3E7FC0093B}
const WD86_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/WebDeploy_x86_ru-RU.msi";
const WD64_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/webdeploy_amd64_ru-RU.msi";
const PATH: &str = "C:\\ScanKass\\WORKFLOW";

#[no_mangle]
pub extern "C" fn is_installed() -> bool{
    Path::new(format!("{}\\SkatWorkerAPI.exe", PATH)).exists()
}

fn exists_app(pattern: &str, arch: String) -> bool{
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut soft = hklm.open_subkey("Software").unwrap();
    if arch == "AMD64" {
        soft = soft.open_subkey("WOW6432Node").unwrap();
    }
    let apps = soft.open_subkey("Microsoft\\Windows\\CurrentVersion\\Uninstall").unwrap();
    for i in apps.enum_keys().map(|x| x.unwrap()) {
        if i.starts_with(pattern) {
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
    Runtime::new().unwrap().block_on(install_async(is_slient))
}

async fn install_async(is_slient: bool) -> i32 {
    if is_installed() { return 1; }

    let arch = var("PROCESSOR_ARCHITECTURE");

    let wd_url = match arch {
        Err(_) => return 2,
        Ok(arch) => match arch.as_str(){
            "AMD64" => WD64_URL,
            "x86" => WD86_URL,
            _ => return 2
        }
    };

    if !exists_app("{215198BD-8EE1-385D-8194-0D3FF304296D}", arch) {
        download_and_execute(ASPNET_URL, is_slient).await;
    }

    if !exists_app("{040F8B83-B3BA-303A-A5BC-FE3E7FC0093B}", arch) {
        download_and_execute(HOSTING_BUNDLE, is_slient).await;
    }

    download_and_install(wd_url, is_slient).await;

    enable_features(vec!["IIS-WebServerRole", "WAS-WindowsActivationService", "WAS-ProcessModel","WAS-NetFxEnvironment","WAS-ConfigurationAPI"]);
    
    let app_inetsrv_path: String = var("WINDIR").unwrap() + "\\system32\\inetsrv\\APPCMD";
    Command::new(&app_inetsrv_path).arg("add").arg("apppool").arg("/name:ScanKass").arg("/processModel.identityType:LocalSystem").status().unwrap(); // Создание отдельного пула
    Command::new(&app_inetsrv_path).arg("add").arg("site").arg("/name:SkatWorkerAPI").arg("/bindings:http/*:80:").arg(format!("/physicalPath:{}",PATH)).status().unwrap(); // Создание сайта
    Command::new(&app_inetsrv_path).arg("set").arg("app").arg("SkatWorkerAPI/").arg("/applicationPool:ScanKass").status().unwrap(); // Присвоение пула

    install_skat_worker().await;

    configure();

    0
}

fn configure(){
    let mut file = OpenOptions::new().read(true).write(true).open(format!("{}\\appsettings.json", PATH)).unwarp();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwarp();
    let mut json: Value = serde_json::from_str(contents.as_str()).unwarp();
    json["Settings"]["ConnectionString"] = Value::String(format!("Data Source={}\\db", PATH).to_string());
    json["Settings"]["PathToLog"] = Value::String("C:\\ScanKass\\LOG".to_string());
    contents = serde_json::to_string(&json).unwarp();
    std::fs::remove_file(format!("{}\\appsettings.json", PATH)).unwarp();
    let mut t = File::create(format!("{}\\appsettings.json", PATH)).unwarp();
    File::write_all(&mut t, contents.as_bytes()).unwarp();
}


async fn download(url: &str, extension: &str) -> String {
    let filename = format!("{0}.{1}", Alphanumeric.sample_string(&mut rand::rng(),16), extension);
    let filepath = format!("{0}{1}", temp_dir().display(), filename);
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await.unwarp();
    let mut file = File::create(filepath.clone()).unwarp();

    while let Some(chunk) = response.chunk().await.unwarp() {
        file.write_all(&chunk).unwarp();
    }

    file.flush().unwarp();

    filepath
}

async fn download_and_execute(url: &str, is_slient: bool) {
    let path = download(url, "exe").await.unwarp();
    let mut ui_mode = "/passive";
    if is_slient {
        ui_mode = "/quiet";
    }
    Command::new(&path).arg("/install").arg(ui_mode).arg("/norestart").status().unwarp();
    std::fs::remove_file(&path).unwarp();
}

async fn download_and_install(url: &str, is_slient: bool) {
    let path = download(url, "msi").await.unwarp();
    let mut ui_mode = "/passive";
    if is_slient {
        ui_mode = "/quiet";
    }
    Command::new("msiexec").arg("/i").arg(path.as_str()).arg(ui_mode).arg("/norestart").status().unwarp();
    std::fs::remove_file(&path).unwarp();
}

async fn download_and_extract(url: &str) -> String {
    let path = download(url, "zip").await.unwarp();
    let mut file = File::open(path).unwarp();
    let mut data: Vec<u8> = vec![];
    file.read_to_end(&mut data).unwarp();
    let dir_path = format!("{0}{1}",temp_dir().display(),Alphanumeric.sample_string(&mut rand::rng(),16));
    std::fs::create_dir_all(&dir_path).unwarp();
    zip_extract::extract(Cursor::new(&data), &PathBuf::from(&dir_path), true).unwarp();
    dir_path
}

async fn get_latest_release() -> String {
    let client = reqwest::Client::new();
    let resp = client.get("https://api.github.com/repos/StarkovVV18/SkatWorker/releases")
        .header("accept", "application/vnd.github+json")
        .header("User-Agent", "curl").send().await.unwarp();
    let body = resp.text().await.unwarp();
    let json_value: serde_json::Value = serde_json::from_str(&body).unwarp();
    let res = format!("{}", json_value[0]["assets"][0]["browser_download_url"]);
    res[1..res.len() - 1].to_string()
}

async fn install_skat_worker() {
    let url = get_latest_release().await.unwarp();
    let path = download_and_extract(url.as_str()).await.unwarp();
    Command::new(format!("{}/SkatWorkerAPI.deploy.cmd", path)).arg("/Y").status().unwarp();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let code = install();
        if code != 0 {
            panic!("Тест не пройден (Код: {code})")
        }
        assert_eq!(is_installed(), true);
    }
}
