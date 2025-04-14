using Microsoft.Win32;
using Newtonsoft.Json.Linq;
using System;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;

namespace ScanKass
{
    internal static class Extensions
    {
        internal static async Task<string> DownloadAsync(this HttpClient http, string url)
        {
            SchedulerInstaller.LogInfo($"Начало скачивания {Path.GetFileName(url)}...");
            try
            {
                var ext = Path.GetExtension(url);
                string filepath;
                do
                {
                    filepath = Path.Combine(Path.GetTempPath(), $"{Path.GetRandomFileName().Replace(".", "")}.{ext}");
                } while (File.Exists(filepath));
                using (var stream = await http.GetStreamAsync(url))
                using (var file = File.Open(filepath, FileMode.CreateNew, FileAccess.Write, FileShare.None))
                    await stream.CopyToAsync(file);
                SchedulerInstaller.LogInfo("Скачивание завершено!");
                return filepath;
            }
            catch (Exception e)
            {
                SchedulerInstaller.LogError("Скачивание было принудительно остановлено!", e);
                return null;
            }
        }

        internal static bool ExistsAppByGuid(this Guid guid, RegistryView view)
        {
            using (var HKLM = RegistryKey.OpenRemoteBaseKey(RegistryHive.LocalMachine, "", view))
            using (var uninstall = HKLM.OpenSubKey($@"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{guid:B}"))
                return !(uninstall is null);
        }

        internal static async Task<string> GetLatestReleaseAsync(this HttpClient http)
        {
            SchedulerInstaller.LogInfo("Получение ссылки на последнюю версию планировщика...");
            try
            {
                var req = new HttpRequestMessage(HttpMethod.Get, Constants.UrlGitLatest);
                req.Headers.Add("accept", "application/vnd.github+json");
                req.Headers.Add("User-Agent", Constants.UserAgent);
                var resp = await http.SendAsync(req);
                var body = await resp.Content.ReadAsStringAsync();
                var json = JArray.Parse(body);
                return json[0]["assets"][0]["browser_download_url"].Value<string>();
            }
            catch (Exception e)
            {
                SchedulerInstaller.LogError("Не удалось получить ссылку на последнюю версию планировщика!", e);
                return null;
            }
        }
    }
}