# Библиотека для установки SkatWorkerAPI для Windows 7-11

## Методы присутствующие в библиотеке

- **bool IsInstalled** - возвращает `true` если SkatWorkerAPI найден в системе и отвечает на запросы, иначе `false`
- **Task InstallAsync()** - асинхронный метод, скачивающий, устанавливающий и настраивающий SkatWorkerAPI 
- **Action<int, string, Exception> Logging** - событие логирования
  - Уровень сообщения (0 - Информация, 1 - Внимание, 2 - Ошибка)
  - Текст сообщения
  - Исключение
- **Action<bool> OnDone** - cобытие, выполняющееся после установки планировщика

## Зависимости
- [Newtonsoft.Json 13.0.3](https://www.nuget.org/packages/Newtonsoft.Json/13.0.3)

## Пример внедрения в .NET проект

```csharp
using System;
using ScanKass;

public static class Program
{
    static async Task Main()
    {
        SchedulerInstaller.Logging += Logging;
        SchedulerInstaller.OnDone += Test;        
        if (SchedulerInstaller.IsInstalled)
            Console.WriteLine("Планировщик уже установлен");
        else 
            await SchedulerInstaller.InstallAsync();
    }  
	
    static void Logging(int level, string text, Exception ex)
    {
        string _level;
        switch (level)
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
        Console.WriteLine($"{DateTime.Now:yyyy'-'MM'-'dd'T'HH':'mm':'ss} {_level} {text}");
        if(!(ex is null)) Console.WriteLine($"{ex}");
    }

    static void Test(bool success)
    {
        if(!success) return;
        try {
            using (var http = new HttpClient())
            {
                var resp = http.GetAsync().Result;
                if(resp.IsSuccessStatusCode)
                    Logging(0, "Тест успешно завершён!")
                Logging(2, "Тест не пройден!")
            }
        }
        catch
        {
            Logging(2, "Тест не пройден!")
        }
    }
}
```
