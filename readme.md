# Библиотека для установки SkatWorkerAPI для Windows 7-11

## Методы присутствующие в библиотеке

- **bool IsInstalled** - возвращает `true` если SkatWorkerAPI найден в системе и отвечает на запросы, иначе `false`
- **Task InstallAsync()** - асинхронный метод, скачивающий, устанавливающий и настраивающий SkatWorkerAPI 
- **Action<int, string, Exception> Logging()** - событие логирования

## Зависимости
- [Newtonsoft.Json 13.0.3](https://www.nuget.org/packages/Newtonsoft.Json/13.0.3)

## Пример внедрения в .NET проект

```csharp
using System;
using ScanKass;

public static class Program
{
    static void Main()
    {
		SchedulerInstaller.Logging += Logging;
        if (SchedulerInstaller.IsInstalled())
            Console.WriteLine("Планировщик уже установлен");
        else 
            SchedulerInstaller.InstallAsync();
    }  
	
	static void Logging(int level, string text, Exception ex)
	{
		string _level;
		switch (l)
		{
			case 0:
				_level = "[INF]";
				break;
			case 1:
				_level = "[WRN]";
				break;
			case 2:
				_level = "[ERR]";
				break;
			default:
				_level = "[???]";
				break;
		}
		Console.WriteLine($"{DateTime.Now:yyyy'-'MM'-'dd'T'HH':'mm':'ss} {_level} {t}");
		if(!(e is null))Console.WriteLine($"{e}");
	}
}
```
