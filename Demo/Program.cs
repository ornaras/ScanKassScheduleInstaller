using ScanKass;
using System;
using System.Threading.Tasks;

namespace Demo
{
    internal class Program
    {
        static async Task Main(string[] args)
        {
            SchedulerInstaller.Logging += (l, t, e) =>
            {
                string level;
                switch (l)
                {
                    case 0:
                        level = "[INF]";
                        break;
                    case 1:
                        level = "[WRN]";
                        break;
                    case 2:
                        level = "[ERR]";
                        break;
                    default:
                        level = "[???]";
                        break;
                }
                Console.WriteLine($"{level} {t}");
                if(!(e is null))Console.WriteLine($"{e}");
            };
            await SchedulerInstaller.InstallAsync();
            Console.WriteLine();
            Console.WriteLine(SchedulerInstaller.IsInstalled ? "Проверка пройдена!" : "Нужна доработка!!!");
        }
    }
}
