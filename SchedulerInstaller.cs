using Newtonsoft.Json.Linq;
using System;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace ScanKass
{
    /// <summary>
    /// Установщик планировщика
    /// </summary>
    public static class SchedulerInstaller
    {
        /// <summary>
        /// true, если планировщик установлен и отвечает на запросы, иначе false
        /// </summary>
        public static bool IsInstalled
        {
            get
            {
                LogInfo("Производиться проверка...");
                var path = Path.Combine(Constants.PathDir, "SkatWorkerAPI.exe");
                if (!File.Exists(path))
                {
                    LogWarning($"Планировщик не установлен: не найден файл {path}");
                    return false;
                }
                using (var http = new HttpClient())
                {
                    try
                    {
                        var resp = http.GetAsync($"http://localhost:{Constants.TcpPort}/api/Schedule/list").Result;
                        if (!resp.IsSuccessStatusCode) throw new Exception();
                    }
                    catch
                    {
                        LogWarning($"Планировщик не установлен: не удалось получить список действий");
                        return false;
                    }
                }
                return true;
            }
        }

        /// <summary>
        /// Событие логирования<br/>
        /// Параметры:<br/>
        /// 1) Уровень сообщения (0 - Info, 1 - Warning, 2 - Error)<br/>
        /// 2) Текст сообщения<br/>
        /// 3) Исключение
        /// </summary>
        public static event Action<int, string, Exception> Logging;
        /// <summary>
        /// Событие, выполняющееся после установки планировщика
        /// </summary>
        public static event Action<bool> OnDone;

        internal static void LogError(string text, Exception ex = null) => Logging?.Invoke(2, text, ex);
        internal static void LogWarning(string text, Exception ex = null) => Logging?.Invoke(1, text, ex);
        internal static void LogInfo(string text) => Logging?.Invoke(0, text, null);

        private static Guid guidHostBundle = new Guid(Constants.GuidHostBundle);
        private static Guid guidWebDeploy = new Guid(Constants.GuidWebDeploy);

        /// <summary>
        /// Установка планировщика
        /// </summary>
        public static async Task InstallAsync()
        {
#if DEBUG
            Logging += (l, t, e) => {
                if (!(e is null))
                    File.AppendAllText("C:\\ScanKass\\LOG\\-_-.log", $"{e}\n");
            };
#endif
            string pathLatest = null, pathScript = null, pathASPNet = null, pathHostBundle = null, pathWebDeploy = null;
            ServicePointManager.SecurityProtocol = (SecurityProtocolType)4080;
            LogInfo("Началась установка планировщика...");
            try
            {
                LogInfo("Активация дополнительных компонентов Windows...");
                EnableFeatures("IIS-WebServerRole", "WAS-WindowsActivationService", 
                    "WAS-ProcessModel", "WAS-ConfigurationAPI");

                var http = new HttpClient();

                pathHostBundle = await http.DownloadAsync(Constants.UrlHostBundle);

                LogInfo("Установка Hosting Bundle 6.0.36...");
                Run(pathHostBundle, "/install /quiet /norestart");
                LogInfo("Коррекция Hosting Bundle 6.0.36...");
                Run(pathHostBundle, "/repair /quiet /norestart");

                pathWebDeploy = await http.DownloadAsync(string.Format(Constants.UrlWebDeploy, Environment.Is64BitOperatingSystem ? "amd64" : "x86"));
                LogInfo("Установка Microsoft Web Deploy 4.0...");
                RunMSI(pathWebDeploy);

                LogInfo("Регистрация сайта в IIS...");
                RunAppcmd($"add site /name:SkatWorkerAPI /bindings:http/*:{Constants.TcpPort}: /physicalPath:{Constants.PathDir}");
                LogInfo("Настройка работы сайта...");
                RunAppcmd("add apppool /name:ScanKass /processModel.identityType:LocalSystem");
                RunAppcmd("set app SkatWorkerAPI/ /applicationPool:ScanKass");
                RunAppcmd("stop site SkatWorkerAPI");

                LogInfo("Проверка и корректировка настроек сайта...");
                RunAppcmd($"set site SkatWorkerAPI /bindings:http/*:{Constants.TcpPort}:");

                var urlLatest = await http.GetLatestReleaseAsync();
                pathLatest = await http.DownloadAsync(urlLatest);
                pathScript = Unzip(pathLatest);
                LogInfo("Развертывание планировщика...");
                Run(Path.Combine(pathScript, "SkatWorkerAPI.deploy.cmd"), "/Y");

                LogInfo("Запуск сайта...");
                RunAppcmd("start site SkatWorkerAPI");

                Configure();

                LogInfo("Запуск сайта...");
                RunAppcmd("start site SkatWorkerAPI");

                OnDone?.Invoke(true);
                LogInfo("Установка планировщика завершена!");
            }
            catch(Exception ex)
            {
                OnDone?.Invoke(false);
                LogError("Установка планировщика принудительно завершена!", ex);
            }
            finally
            {
                LogInfo("Очистка временных файлов...");
                RemoveFile(pathASPNet);
                RemoveFile(pathHostBundle);
                RemoveFile(pathWebDeploy);
                RemoveFile(pathLatest);
                if (!(pathScript is null) && Directory.Exists(pathScript))
                    Directory.Delete(pathScript, true);
                LogInfo("Очистка временных файлов завершена!");
            }
        }

        private static void RemoveFile(string filepath)
        {
            if (!(filepath is null) && File.Exists(filepath))
                File.Delete(filepath);
        }

        private static void EnableFeatures(params string[] features)
        {
            var args = new StringBuilder("/online /NoRestart /enable-feature");
            foreach (var feature in features)
                args.Append($" /featurename:{feature}");
            Run(Environment.Is64BitOperatingSystem ? Constants.PathDism64 : Constants.PathDism, args.ToString());
        }

        private static void RunMSI(string path)
        {
            const string nameMutex = "Global\\_MSIExecute";
            do
            {
                if(!Mutex.TryOpenExisting(nameMutex, out _))
                {
                    Run("msiexec", $"/i {path} /quiet /norestart");
                    break;
                }
                Thread.Sleep(1000);
            } while (Mutex.TryOpenExisting(nameMutex, out _));
        }
        private static void RunAppcmd(string args) => Run(Constants.PathInetcmd, args);
        private static string Unzip(string path)
        {
            LogInfo($"Распаковка файла {Path.GetFileName(path)}...");
            string dir;
            do
            {
                dir = Path.Combine(Path.GetTempPath(), Path.GetRandomFileName()).Replace(".", "");
            } while (Directory.Exists(dir));
            Directory.CreateDirectory(dir);
            using (var stream = File.Open(path, FileMode.Open, FileAccess.Read, FileShare.Read))
                new ZipArchive(stream, ZipArchiveMode.Read).ExtractToDirectory(dir);
            return dir;
        }

        private static void Configure()
        {
            LogInfo("Настройка планировщика...");
            var path = Path.Combine(Constants.PathDir, "appsettings.json");
            var config = File.ReadAllText(path);
            var json = JObject.Parse(config);

            json["Settings"]["ConnectionString"] = $"Data Source = {Path.Combine(Constants.PathDir, "db")}";
            json["Settings"]["PathToLog"] = Constants.PathLog;

            File.Delete(path);
            File.WriteAllText(path, json.ToString());
        }

        internal static void Run(string program, string args)
        {
            LogInfo($"Запуск \"{program} {args}\"...");

            var pInfo = new ProcessStartInfo(program, args)
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                Verb = "runas",
#if DEBUG
                RedirectStandardOutput = true,
                RedirectStandardError = true
#endif
                };
            var proc = new Process(){ StartInfo = pInfo };
            proc.Start();

#if DEBUG
            var text = proc.StandardOutput.ReadToEnd();
            if (!string.IsNullOrWhiteSpace(text))
                foreach(var line in text.Split('\n'))
                    LogInfo($"O > {line}");

            text = proc.StandardError.ReadToEnd();
            if (!string.IsNullOrWhiteSpace(text))
                foreach (var line in text.Split('\n'))
                    LogError($"O > {line}");
#endif

            proc.WaitForExit();
        }
    }
}
