using System;

namespace ScanKass
{
    internal static class Constants
    {
        public const string GuidHostBundle = "040F8B83-B3BA-303A-A5BC-FE3E7FC0093B";
        public const string GuidWebDeploy = "82FD8C73-C24D-433C-85A9-48AE93570410";
        public const string PathCmd = "C:\\Windows\\system32\\cmd.exe";
        public const string PathDir = "C:\\ScanKass\\WORKFLOW";
        public const string PathDism = "C:\\Windows\\system32\\dism.exe";
        public const string PathDism64 = "C:\\Windows\\SysNative\\dism.exe";
        public const string PathInetcmd = "C:\\Windows\\system32\\inetsrv\\appcmd.exe";
        public const string PathLog = "C:\\ScanKass\\LOG";
        public const string TcpPort = "16160";
        public const string UrlAspNet = "https://download.visualstudio.microsoft.com/download/pr/8cfa7f46-88f2-4521-a2d8-59b827420344/447de18a48115ac0fe6f381f0528e7a5/aspnetcore-runtime-6.0.36-win-x86.exe";
        public const string UrlHostBundle = "https://download.visualstudio.microsoft.com/download/pr/9b8253ef-554d-4636-b708-e154c0199ce5/f3673dd1f2dc80e5b0505cbd2d4bd5d2/dotnet-hosting-6.0.36-win.exe";
        public static string UrlWebDeploy = $"https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/WebDeploy_{(Environment.Is64BitOperatingSystem ? "amd64" : "x86")}_ru-RU.msi";
        public const string UserAgent = "ScanKass";
    }
}
