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

        internal static ulong ToUnixTimestamp(this DateTime dt) =>
            (ulong)(dt - Constants.UnixEpoch).TotalSeconds;
    }
}