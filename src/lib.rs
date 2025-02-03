extern crate rand;

use std::fs::File;
use std::env::temp_dir;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use serde_json::json;
use std::process::Command;
use futures_util::StreamExt;
use rand::distr::{Alphanumeric, SampleString};

const ASPNET_URL: &str = "https://download.visualstudio.microsoft.com/download/pr/8cfa7f46-88f2-4521-a2d8-59b827420344/447de18a48115ac0fe6f381f0528e7a5/aspnetcore-runtime-6.0.36-win-x86.exe";
const HOSTING_BUNDLE: &str = "https://download.visualstudio.microsoft.com/download/pr/9b8253ef-554d-4636-b708-e154c0199ce5/f3673dd1f2dc80e5b0505cbd2d4bd5d2/dotnet-hosting-6.0.36-win.exe";
const ENABLE_IIS: &str = "\"Enable-WindowsOptionalFeature -Online -FeatureName IIS-ASPNET, IIS-ManagementConsole -All\"";
const WD86_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/WebDeploy_x86_ru-RU.msi";
const WD64_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/webdeploy_amd64_ru-RU.msi";

pub fn is_installed() -> bool {
    true
}

pub async fn install() {
    //if is_installed() { panic!("Планировщик уже установлен") }

    download_and_execute(ASPNET_URL).await;
    download_and_execute(HOSTING_BUNDLE).await;

    let wd_url = match isize::MAX != i32::MAX as isize{
        true => WD64_URL,
        false => WD86_URL
    };

    download_and_install(wd_url).await;

    _ = Command::new("powershell").args(["-Command", ENABLE_IIS]);

    install_skat_worker().await;
}

async fn download(url: &str, extension: &str) -> String {
    let filename = format!("{0}.{1}", Alphanumeric.sample_string(&mut rand::rng(),16), extension);
    let filepath = format!("{0}{1}", temp_dir().display(), filename);
    let mut file = File::create(filepath.clone()).expect("Не удалось создать файл");

    let resp = reqwest::get(url).await.expect("Не удалось отправить HTTP-запрос");
    let mut stream = resp.bytes_stream();

    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        file.write_all(&chunk).unwrap();
    }

    file.flush().unwrap();

    filepath
}

async fn download_and_execute(url: &str) {
    let path = download(url, "exe").await;
    _ = Command::new(&path).args(["/install","/passive","/norestart"]).output();
    std::fs::remove_file(&path).unwrap();
}

async fn download_and_install(url: &str) {
    let path = download(url, "msi").await;
    _ = Command::new("msiexec").args(["/i", path.as_str(),"/passive","/norestart"]).output();
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

async fn get_latest_release(owner: &str, repo: &str) -> String {
    let url = format!("https://api.github.com/repos/{0}/{1}/releases",owner,repo);
    let resp = reqwest::get(url).await.unwrap();
    let body = json!(resp.text().await.unwrap());
    body[0]["assets"][0]["browser_download_url"].to_string()
}

async fn install_skat_worker(){
    let url = get_latest_release("StarkovVV18", "SkatWorker").await;
    let path = download_and_extract(url.as_str()).await;
    _ = Command::new(format!("{}SkatWorkerAPI.deploy.cmd", path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        tokio::runtime::Runtime::new().unwrap().block_on(install());
        assert_eq!(is_installed(), true);
    }
}
